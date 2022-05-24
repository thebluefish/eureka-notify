use std::collections::HashMap;
use std::env;
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::{Duration, Utc};
use chrono_humanize::HumanTime;
use derive_more::{Deref, DerefMut};
use lazy_static::lazy_static;
use notify_rust::Notification;
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use serenity::async_trait;
use serenity::framework::standard::{
    Args,
    CommandGroup,
    CommandOptions,
    CommandResult,
    DispatchError,
    help_commands,
    HelpOptions,
    Reason,
    StandardFramework,
};
use serenity::framework::standard::buckets::{LimitedFor, RevertBucket};
use serenity::framework::standard::macros::{check, command, group, help, hook};
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::{content_safe, ContentSafeOptions};
use tokio::time::sleep;
use tracing::*;
use tracing_subscriber;

use eureka_notify::prelude::*;

// Globals loaded from environment vars
lazy_static! {
    pub static ref DISCORD_TOKEN: String = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in env");
    pub static ref DB: Mutex<PickleDb> = Mutex::new(PickleDb::load("data.db", PickleDbDumpPolicy::DumpUponRequest, SerializationMethod::Json).unwrap_or_else(|_| PickleDb::new("data.db", PickleDbDumpPolicy::DumpUponRequest, SerializationMethod::Json)));
    // pub static ref NOTIFICATION_ROLE_ID: u64 = env::var("NOTIFICATION_ROLE_ID").expect("Expected NOTIFICATION_ROLE_ID in env").parse().expect("Invalid NOTIFICATION_ROLE_ID");
    // pub static ref CHANNEL_ID: u64 = env::var("CHANNEL_ID").expect("Expected CHANNEL_ID in env").parse().expect("Invalid CHANNEL_ID");
}

#[derive(Default, Deref, DerefMut)]
struct Handler(AtomicBool);

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct GuildItem {
    ping_id: Option<u64>,
    posts: Vec<(u64, i64)>,
}

enum TimeSleep {
    OneCycle,
    FiveMinutes,
    ThirtySeconds,
    Now,
}

impl Display for TimeSleep {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeSleep::OneCycle => f.write_str("in 23 minutes"),
            TimeSleep::FiveMinutes => f.write_str("in 5 minutes"),
            TimeSleep::ThirtySeconds => f.write_str("in 30 seconds"),
            TimeSleep::Now => f.write_str("now"),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _state: Ready) {
        // Trigger the main loop to start on first load
        if !self.load(Ordering::Relaxed) {
            self.swap(true, Ordering::Relaxed);

            tokio::spawn(run_main_loop(ctx));
        }
    }
}

#[group]
// This requires us to call commands in this group
#[prefixes("ross", "br")]
#[only_in(guilds)]
#[summary = "Ross commands"]
// Sets a command that will be executed if only a group-prefix was passed.
#[commands(cat, dog)]
struct Ross;

#[command]
#[description = "Sets the ID to ping on near-future events"]
#[bucket = "ross"]
#[sub_commands(sub)]
#[required_permissions("ADMINISTRATOR")]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.react("white_check_mark");
    Ok(())
}

#[command("set")]
#[description("Sets the ID to ping")]
async fn ping_set(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "This is a sub function!").await?;

    Ok(())
}

#[command("stop")]
#[aliases("clear", "remove")]
#[description("Clears the ping ID, stopping the bot from pinging anyone")]
async fn ping_clear(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "This is a sub function!").await?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    // Optionally load the .env file
    if let Err(e) = dotenv::dotenv() {
        error!("failed to load .env file: {e}");
    }

    let mut client = Client::builder(&*DISCORD_TOKEN, GatewayIntents::GUILD_MESSAGES)
        .event_handler(Handler(false.into()))
        .await
        .expect("Failed to create client")
        ;

    info!("Initializing discord client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
    Ok(())
}

async fn run_main_loop(ctx: Context) {
    info!("Starting main loop");

    // Start either where we left off previously or now
    let mut skip_first_tick = false;
    let mut now = EorzeaDateTime::now().truncated(Duration::hours(8));
    let catch_up = now;
    if let Some(db_now) = DB.lock().await.get("now") {
        if db_now < now.timestamp() {
            now = EorzeaDateTime::from_timestamp(db_now) + Duration::hours(8);
        } else { // if we already ran the current frame, sleep until the next one
            skip_first_tick = true;
        }
    }

    let mut guilds: HashMap<u64, GuildItem> = DB.lock().await.get("posts").unwrap_or(HashMap::new());

    loop {
        let future = now + Duration::hours(8);

        let crab = crab_status(now);
        let crabb = if crab == future { Some(crab) } else { None };

        let cassie = cassie_status(now);
        let cassiee = if cassie == future { Some(cassie) } else { None };

        // let (hotbox, hotbox_count) = hotbox_farm(now);
        // let hotboxx = if hotbox == future { Some((hotbox, hotbox_count)) } else { None };
        //
        // let (offensive, offensive_count) = offensive_farm(now);
        // let offensivee = if offensive == future { Some((offensive, offensive_count)) } else { None };

        if skip_first_tick {
            info!("skipping same tick");
            skip_first_tick = false;
        } else {
            for (guild_id, GuildItem { ping_id, mut posts }) in guilds {

                // Post updates
                let post_id = post_discord(&ctx, now).await;

                // Send notifications for up to 1 weather cycle
                if future > catch_up && (crabb.is_some() || cassiee.is_some()/* || hotboxx.is_some() || offensivee.is_some()*/) {
                    notify_os(TimeSleep::OneCycle, crabb, cassiee/*, hotboxx, offensivee*/);
                }

                // Clean up historical posts
                for (id, timestamp) in posts.drain(..) {
                    edit_post(&ctx, id, EorzeaDateTime::from_timestamp(timestamp)).await;
                }

                // Push this post to history
                if let Some(id) = post_id {
                    posts.push((id, now.timestamp()));
                }
            }

            let mut db = DB.lock().await;
            db.set("posts", &past_posts).unwrap();
            db.set("now", &(now.timestamp())).unwrap();
            db.dump().expect("failed to save db");

            info!("Completed tick for {}", now.to_utc());
        }

        // Wait until 5 minutes before the next cycle to send another notification
        if future > catch_up && (crabb.is_some() || cassiee.is_some()/* || hotboxx.is_some() || offensivee.is_some()*/) {
            if let Ok(duration) = ((future.to_utc() - Duration::minutes(5)) - Utc::now()).to_std() {
                info!("sleep for {:?} to 5-minute notification", duration);
                sleep(duration).await;

                notify_os(TimeSleep::FiveMinutes, crabb, cassiee/*, hotboxx, offensivee*/);
            }
        }

        // Wait until 30 seconds before the next cycle to send another notification
        if future > catch_up && (crabb.is_some() || cassiee.is_some()/* || hotboxx.is_some() || offensivee.is_some()*/) {
            if let Ok(duration) = ((future.to_utc() - Duration::seconds(30)) - Utc::now()).to_std() {
                info!("sleep for {:?} to 30-second notification", duration);
                sleep(duration).await;

                notify_os(TimeSleep::ThirtySeconds, crabb, cassiee/*, hotboxx, offensivee*/);
            }
        }

        if let Ok(duration) = (future.to_utc() - Utc::now()).to_std() {
            info!("sleep for {:?} to next weather", duration);
            sleep(duration).await;
        }

        // if crabb.is_some() || cassiee.is_some()/* || hotboxx.is_some() || offensivee.is_some()*/ {
        //     notify_os(TimeSleep::Now, crabb, cassiee/*, hotboxx, offensivee*/);
        // }

        now = future;
    }
}

pub async fn edit_post(ctx: &Context, id: u64, now: EorzeaDateTime) {
    let pagos = EorzeaMap::from_name("Eureka Pagos").expect("Could not find map");
    let pyros = EorzeaMap::from_name("Eureka Pyros").expect("Could not find map");
    let hydatos = EorzeaMap::from_name("Eureka Hydatos").expect("Could not find map");
    let past = now - Duration::hours(8);
    let future = now + Duration::hours(8);

    // We will post the next crab timer when this is crab weather *or* crab weather is next
    let crab_time = crab_status(now);
    let crab = if crab_status(past) == now || crab_time == future { Some(crab_time) } else { None };

    let cassie_time = cassie_status(now);
    let cassie = if cassie_status(past) == now || cassie_time == future { Some(cassie_time) } else { None };

    let past_crab = crab_status(past) == now;
    let past_cassie = cassie_status(past) == now;

    let result = ChannelId(CHANNEL_ID.clone()).edit_message(&ctx, id, |m| {
        m.content("");
        if past_crab || past_cassie {
            m.add_embed(|e| {
                e
                    .field(format!("Crab <t:{}:R>", crab_time.to_utc().timestamp()), format!("<t:{}>", crab_time.to_utc().timestamp()), true)
                    .field(format!("Cassie <t:{}:R>", cassie_time.to_utc().timestamp()), format!("<t:{}>", cassie_time.to_utc().timestamp()), true)
            });
        } else if crab.is_some() || cassie.is_some() {
            m.add_embed(|e| {
                if let Some(crab) = crab {
                    e.field(format!("Crab <t:{}:R>", crab.to_utc().timestamp()), format!("<t:{}>", crab.to_utc().timestamp()), true);
                }
                if let Some(cassie) = cassie {
                    e.field(format!("Cassie <t:{}:R>", cassie.to_utc().timestamp()), format!("<t:{}>", cassie.to_utc().timestamp()), true);
                }
                e
            });
        }
        m.add_embed(|e| {
            e
                .field(format!("Pagos: {}", pagos.weather(now)), format!("Next: {}", pagos.weather(future)), true)
                .field(format!("Pyros: {}", pyros.weather(now)), format!("Next: {}", pyros.weather(future)), true)
                .field(format!("Hydatos: {}", hydatos.weather(now)), format!("Next: {}", hydatos.weather(future)), true)
                .field(format!("<t:{}:R>", now.to_utc().timestamp()), format!("<t:{}>", now.to_utc().timestamp()), false)
        });
        m
    }).await;
    if let Err(err) = result {
        eprintln!("Error editing discord post: {err:?}");
    };
}

/// Create the discord log for this weather cycle
pub async fn post_discord(ctx: &Context, now: EorzeaDateTime) -> Option<u64> {
    let pagos = EorzeaMap::from_name("Eureka Pagos").expect("Could not find map");
    let pyros = EorzeaMap::from_name("Eureka Pyros").expect("Could not find map");
    let hydatos = EorzeaMap::from_name("Eureka Hydatos").expect("Could not find map");
    let past = now - Duration::hours(8);
    let future = now + Duration::hours(8);

    // We will post the next crab timer when this is crab weather *or* crab weather is next
    let crab_time = crab_status(now);
    let crab = if crab_status(past) == now || crab_time == future { Some(crab_time) } else { None };

    let cassie_time = cassie_status(now);
    let cassie = if cassie_status(past) == now || cassie_time == future { Some(cassie_time) } else { None };

    let past_crab = crab_status(past) == now;
    let past_cassie = cassie_status(past) == now;

    let message = ChannelId(CHANNEL_ID.clone())
        .send_message(&ctx, |m| {
            // Notify when futures are near
            if crab_time == future || cassie_time == future {
                m.content(RoleId(NOTIFICATION_ROLE_ID.clone()).mention());
            }
            if past_crab || past_cassie {
                m.add_embed(|e| {
                    e
                        .field(format!("Crab <t:{}:R>", crab_time.to_utc().timestamp()), format!("<t:{}>", crab_time.to_utc().timestamp()), true)
                        .field(format!("Cassie <t:{}:R>", cassie_time.to_utc().timestamp()), format!("<t:{}>", cassie_time.to_utc().timestamp()), true)
                });
            } else if crab.is_some() || cassie.is_some() {
                m.add_embed(|e| {
                    if let Some(crab) = crab {
                        e.field(format!("Crab <t:{}:R>", crab.to_utc().timestamp()), format!("<t:{}>", crab.to_utc().timestamp()), true);
                    }
                    if let Some(cassie) = cassie {
                        e.field(format!("Cassie <t:{}:R>", cassie.to_utc().timestamp()), format!("<t:{}>", cassie.to_utc().timestamp()), true);
                    }
                    e
                });
            }
            m.add_embed(|e| {
                e
                    .field(format!("Pagos: {}", pagos.weather(now)), format!("Next: {}", pagos.weather(future)), true)
                    .field(format!("Pyros: {}", pyros.weather(now)), format!("Next: {}", pyros.weather(future)), true)
                    .field(format!("Hydatos: {}", hydatos.weather(now)), format!("Next: {}", hydatos.weather(future)), true)
                    .field(format!("Next <t:{}:R>", future.to_utc().timestamp()), format!("Started <t:{}:R>", now.to_utc().timestamp()), false)
            });
            m
        })
        .await;

    match message {
        Ok(msg) => Some(msg.id.0),
        Err(err) => {
            error!("Error sending message: {err:?}");
            None
        }
    }
}

pub async fn notify_discord(ctx: &Context, crab: Option<EorzeaDateTime>, cassie: Option<EorzeaDateTime>,
                            /*hotbox: Option<(EorzeaDateTime, usize)>, offensive: Option<(EorzeaDateTime, usize)>*/) {
    let message = ChannelId(CHANNEL_ID.clone())
        .send_message(&ctx, |m| {
            m.content(RoleId(NOTIFICATION_ROLE_ID.clone()).mention());
            m.add_embed(|e| {
                if let Some(crab) = crab {
                    e.field(format!("Crab <t:{}:R>", crab.to_utc().timestamp()), format!("<t:{}>", crab.to_utc().timestamp()), false);
                };
                if let Some(cassie) = cassie {
                    e.field(format!("Cassie <t:{}:R>", cassie.to_utc().timestamp()), format!("<t:{}>", cassie.to_utc().timestamp()), false);
                }
                // if let Some((hotbox, hotbox_count)) = hotbox {
                //     e.field(format!("Hotbox x{} <t:{}:R>", hotbox_count, hotbox.to_utc().timestamp()), format!("<t:{}>", hotbox.to_utc().timestamp()), false);
                // }
                // if let Some((offensive, offensive_count)) = offensive {
                //     e.field(format!("Offensive x{} <t:{}:R>", offensive_count, offensive.to_utc().timestamp()), format!("<t:{}>", offensive.to_utc().timestamp()), false);
                // }
                e
            });
            m
        })
        .await;

    if let Err(why) = message {
        eprintln!("Error sending discord notification: {:?}", why);
    };
}
