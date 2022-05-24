use eureka_notify::prelude::*;
use std::env;
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicBool, Ordering};
use chrono::{Duration, Utc};
use chrono_humanize::HumanTime;
use derive_more::{Deref, DerefMut};
use tokio::time::sleep;
use tracing::*;
use tracing_subscriber;
use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;

use lazy_static::lazy_static;
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use notify_rust::Notification;

// Globals loaded from environment vars
lazy_static! {
    pub static ref DISCORD_TOKEN: String = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in env");
    pub static ref CHANNEL_ID: u64 = env::var("CHANNEL_ID").expect("Expected CHANNEL_ID in env").parse().expect("Invalid CHANNEL_ID");
    pub static ref NOTIFICATION_ROLE_ID: u64 = env::var("NOTIFICATION_ROLE_ID").expect("Expected NOTIFICATION_ROLE_ID in env").parse().expect("Invalid NOTIFICATION_ROLE_ID");
}

#[derive(Default, Deref, DerefMut)]
struct Handler(AtomicBool);

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

    let mut db = PickleDb::load("data.db", PickleDbDumpPolicy::DumpUponRequest, SerializationMethod::Json).unwrap_or_else(|_| PickleDb::new("data.db", PickleDbDumpPolicy::DumpUponRequest, SerializationMethod::Json));

    // Start either where we left off previously or now
    let mut skip_first_tick = false;
    let mut now = EorzeaDateTime::now().truncated(Duration::hours(8));
    let catch_up = now;
    if let Some(db_now) = db.get("now") {
        if db_now < now.timestamp() {
            now = EorzeaDateTime::from_timestamp(db_now) + Duration::hours(8);
        }
        else { // if we already ran the current frame, sleep until the next one
            skip_first_tick = true;
        }
    }

    let mut past_posts = db.get("posts").unwrap_or(Vec::<(u64, i64)>::new());

    loop {
        let future = now + Duration::hours(8);

        let crab = speed_belt(now);
        let crabb = if crab == future { Some(crab) } else { None };

        let cassie = cassie_earring(now);
        let cassiee = if cassie == future { Some(cassie) } else { None };

        // let (hotbox, hotbox_count) = hotbox_farm(now);
        // let hotboxx = if hotbox == future { Some((hotbox, hotbox_count)) } else { None };
        //
        // let (offensive, offensive_count) = offensive_farm(now);
        // let offensivee = if offensive == future { Some((offensive, offensive_count)) } else { None };

        if skip_first_tick {
            info!("skipping same tick");
            skip_first_tick = false;
        }
        else {
            // Post updates
            let post_id = post_discord(&ctx, now).await;

            // Send notifications for up to 1 weather cycle
            if future > catch_up && (crabb.is_some() || cassiee.is_some()/* || hotboxx.is_some() || offensivee.is_some()*/) {
                notify_os(TimeSleep::OneCycle, crabb, cassiee/*, hotboxx, offensivee*/);
            }

            // Clean up historical posts
            for (id, timestamp) in past_posts.drain(..) {
                edit_post(&ctx, id, EorzeaDateTime::from_timestamp(timestamp)).await;
            }

            // Push this post to history
            if let Some(id) = post_id {
                past_posts.push((id, now.timestamp()));
            }

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
    let crab_time = speed_belt(now);
    let crab = if speed_belt(past) == now || crab_time == future { Some(crab_time) } else { None };

    let cassie_time = cassie_earring(now);
    let cassie = if cassie_earring(past) == now || cassie_time == future { Some(cassie_time) } else { None };

    let past_crab = speed_belt(past) == now;
    let past_cassie = cassie_earring(past) == now;

    let result = ChannelId(CHANNEL_ID.clone()).edit_message(&ctx, id, |m| {
        m.content("");
        if past_crab || past_cassie {
            m.add_embed(|e| {
                e
                    .field(format!("Crab <t:{}:R>", crab_time.to_utc().timestamp()), format!("<t:{}>", crab_time.to_utc().timestamp()), true)
                    .field(format!("Cassie <t:{}:R>", cassie_time.to_utc().timestamp()), format!("<t:{}>", cassie_time.to_utc().timestamp()), true)
            });
        }
        else if crab.is_some() || cassie.is_some() {
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
    let crab_time = speed_belt(now);
    let crab = if speed_belt(past) == now || crab_time == future { Some(crab_time) } else { None };

    let cassie_time = cassie_earring(now);
    let cassie = if cassie_earring(past) == now || cassie_time == future { Some(cassie_time) } else { None };

    let past_crab = speed_belt(past) == now;
    let past_cassie = cassie_earring(past) == now;

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
            }
            else if crab.is_some() || cassie.is_some() {
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

fn notify_os(timesleep: TimeSleep, crab: Option<EorzeaDateTime>, cassie: Option<EorzeaDateTime>) {

    if let Some(dt) = crab {
        let length = match timesleep {
            TimeSleep::OneCycle => format!("{}", HumanTime::from(dt.to_utc())),
            TimeSleep::FiveMinutes => "in 5 minutes".into(),
            TimeSleep::ThirtySeconds => "in 30 seconds".into(),
            TimeSleep::Now => "now".into(),
        };
        Notification::new()
            .app_id("com.squirrel.XIVLauncher.XIVLauncher")
            .summary("Crab")
            .body(&format!("{length}"))
            .sound_name("Default")
            .show()
            .expect("failed to open OS notification");
    }

    if let Some(dt) = cassie {
        let length = match timesleep {
            TimeSleep::OneCycle => format!("{}", HumanTime::from(dt.to_utc())),
            TimeSleep::FiveMinutes => "in 5 minutes".into(),
            TimeSleep::ThirtySeconds => "in 30 seconds".into(),
            TimeSleep::Now => "now".into(),
        };
        Notification::new()
            .app_id("com.squirrel.XIVLauncher.XIVLauncher")
            .summary("Cassie")
            .body(&format!("{length}"))
            .sound_name("Default")
            .show()
            .expect("failed to open OS notification");
    }
}

fn is_hotbox_weather(weather: EorzeaWeather) -> bool {
    weather.name == "Snow" || weather.name == "Blizzards" || weather.name == "Umbral Wind"
}

fn hotbox_farm(mut now: EorzeaDateTime) -> (EorzeaDateTime, usize) {
    let zone = EorzeaMap::from_name("Eureka Pyros").expect("Could not find map");

    // Start search next weather
    now += Duration::hours(8);

    let mut count;
    loop {
        // Find next hotbox weather
        while !is_hotbox_weather(zone.weather(now)) {
            now += Duration::hours(8);
        }

        count = 1;
        let mut future = now + Duration::hours(8);

        // Count total back-to-back hotbox weathers
        while is_hotbox_weather(zone.weather(future)) {
            count += 1;
            future += Duration::hours(8);
        }

        // Break once we have multiple good weathers
        if count > 1 {
            break;
        }

        now += Duration::hours(8);
    }
    (now, count)
}

fn offensive_farm(mut now: EorzeaDateTime) -> (EorzeaDateTime, usize) {
    let zone = EorzeaMap::from_name("Eureka Hydatos").expect("Could not find map");

    // Start search next weather
    now += Duration::hours(8);

    let mut count;
    loop {
        // Find next hotbox weather
        while zone.weather(now).name != "Snow" {
            now += Duration::hours(8);
        }

        count = 1;
        let mut future = now + Duration::hours(8);

        // Count total back-to-back hotbox weathers
        while zone.weather(future).name == "Snow" {
            count += 1;
            future += Duration::hours(8);
        }

        // Break once we have multiple good weathers
        if count > 1 {
            break;
        }

        now += Duration::hours(8);
    }
    (now, count)
}

fn speed_belt(mut now: EorzeaDateTime) -> EorzeaDateTime {
    let zone = EorzeaMap::from_name("Eureka Pagos").expect("Could not find map");

    // Find next non-Fog
    while zone.weather(now).name == "Fog" {
        now += Duration::hours(8);
    }
    let mut future = now + Duration::hours(8);

    // Fog after is our target
    while zone.weather(future).name != "Fog" {
        future += Duration::hours(8);
    }
    future
}

fn cassie_earring(mut now: EorzeaDateTime) -> EorzeaDateTime {
    let zone = EorzeaMap::from_name("Eureka Pagos").expect("Could not find map");

    // Find next non-Blizzards
    while zone.weather(now).name == "Blizzards" {
        now += Duration::hours(8);
    }
    let mut future = now + Duration::hours(8);

    // Blizzards after is our target
    while zone.weather(future).name != "Blizzards" {
        future += Duration::hours(8);
    }
    future
}

fn skoll_claw(mut now: EorzeaDateTime) -> EorzeaDateTime {
    let zone = EorzeaMap::from_name("Eureka Pyros").expect("Could not find map");

    // Find next non-Blizzards
    while zone.weather(now).name == "Blizzards" {
        now += Duration::hours(8);
    }
    let mut future = now + Duration::hours(8);

    // Blizzards after is our target
    while zone.weather(future).name != "Blizzards" {
        future += Duration::hours(8);
    }
    future
}