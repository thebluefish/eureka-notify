use std::collections::HashMap;
use std::env;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use eureka_notify::{prelude::*, discord::*};
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

// Globals loaded from environment vars
lazy_static! {
    pub static ref DISCORD_TOKEN: String = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in env");
    pub static ref DB: Arc<Mutex<PickleDb>> = Arc::new(Mutex::new(PickleDb::load("data.db", PickleDbDumpPolicy::DumpUponRequest, SerializationMethod::Json).unwrap_or_else(|_| PickleDb::new("data.db", PickleDbDumpPolicy::DumpUponRequest, SerializationMethod::Json))));
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct GuildItem {
    pub channel_id: Option<u64>,
    pub role_id: Option<u64>,
    pub posts: Vec<(u64, i64)>,
    pub notifications: Vec<u64>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    // Optionally load the .env file
    if let Err(e) = dotenv::dotenv() {
        error!("failed to load .env file: {e}");
    }

    let framework = StandardFramework::new()
        .configure(|c| c
            .with_whitespace(true)
            .prefix("^")
            .delimiters(vec![", ", ","])
        )
        // Set a function that's called whenever a command's execution didn't complete for one
        // reason or another. For example, when a user has exceeded a rate-limit or a command
        // can only be performed by the bot owner.
        .before(before)
        .on_dispatch_error(dispatch_error)
        .bucket("ross", |b| b.delay(2)).await
        .group(&ROSS_GROUP);

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_MESSAGE_REACTIONS | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&*DISCORD_TOKEN, intents)
        .event_handler(DiscordHandler(false.into()))
        .framework(framework)
        .type_map_insert::<DataStore>(DataStore(DB.clone()))
        .await
        .expect("Failed to create client")
        ;

    info!("Initializing discord client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
    Ok(())
}

#[group]
// This requires us to call commands in this group
#[prefixes("ross", "br")]
#[only_in(guilds)]
#[summary = "Ross commands"]
// Sets a command that will be executed if only a group-prefix was passed.
#[commands(notify, ping)]
pub struct Ross;

#[derive(Default, Deref, DerefMut)]
struct DiscordHandler(AtomicBool);

#[async_trait]
impl EventHandler for DiscordHandler {
    async fn ready(&self, ctx: Context, _state: Ready) {
        // Trigger the main loop to start on first load
        if !self.load(Ordering::Relaxed) {
            self.swap(true, Ordering::Relaxed);

            tokio::spawn(run_main_loop(ctx));
        }
    }
}

async fn run_main_loop(ctx: Context) {
    info!("Starting main loop");

    // Start either where we left off previously or now
    let mut skip_first_tick = false;
    let mut now = DateTimeEorzea::now().truncated(Duration::hours(8));

    if let Some(db_now) = DB.lock().await.get("now") {
        if db_now < now.timestamp() {
            now = DateTimeEorzea::from_timestamp(db_now) + Duration::hours(8);
        } else { // if we already ran the current frame, sleep until the next one
            skip_first_tick = true;
        }
    }

    loop {
        let future = now + Duration::hours(8);

        let crab = crab_status(now, Direction::Future);
        let crab = if crab == future { Some(crab) } else { None };

        let cassie = cassie_status(now, Direction::Future);
        let cassie = if cassie == future { Some(cassie) } else { None };

        let skoll = skoll_status(now, Direction::Future);
        let skoll = if skoll == future { Some(skoll) } else { None };

        let do_notify = crab.is_some() || cassie.is_some() || skoll.is_some();

        if skip_first_tick {
            info!("skipping same tick");
            skip_first_tick = false;
        } else {
            let mut db = DB.lock().await;
            let mut guilds: HashMap<u64, GuildItem> = db.get("guilds").unwrap_or(HashMap::new());

            for (_, guild) in guilds.iter_mut() {
                if let Some(channel_id) = guild.channel_id {
                    // Post updates
                    let post_id = post_discord(&ctx, channel_id, guild.role_id, now).await;

                    // Clean up historical posts
                    for (id, timestamp) in guild.posts.drain(..) {
                        edit_post(&ctx, channel_id, id, DateTimeEorzea::from_timestamp(timestamp)).await;
                    }

                    // Clean up historical notifications
                    for id in guild.notifications.drain(..) {
                        edit_notification(&ctx, channel_id, id, now).await;
                    }

                    // Push this post to history
                    if let Some(id) = post_id {
                        guild.posts.push((id, now.timestamp()));
                    }
                }
            }

            db.set("guilds", &guilds).unwrap();
            db.set("now", &(now.timestamp())).unwrap();
            db.dump().expect("failed to save db");

            info!("Completed tick for {}", now.to_utc());
        }

        // Wait until 5 minutes before the next cycle to send a notification
        if do_notify {
            if let Ok(duration) = ((future.to_utc() - Duration::minutes(5)) - Utc::now()).to_std() {
                info!("sleep for {:?} to 5-minute notification", duration);
                sleep(duration).await;

                let mut db = DB.lock().await;
                let mut guilds: HashMap<u64, GuildItem> = db.get("guilds").unwrap_or(HashMap::new());

                for (_, guild) in guilds.iter_mut() {
                    if let Some(channel_id) = guild.channel_id {
                        // Push this post to history
                        if let Some(id) = notify_discord(&ctx, channel_id, guild.role_id, now).await {
                            guild.notifications.push(id);
                        }
                    }
                }
            }
        }

        // Wait until next cycle
        if let Ok(duration) = (future.to_utc() - Utc::now()).to_std() {
            info!("sleep for {:?} to next weather", duration);
            sleep(duration).await;
        }

        now = future;
    }
}

#[hook]
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    msg.react(ctx, '✅').await.ok();

    true
}

#[command]
#[description = "Explains the current notification configuration"]
#[bucket = "ross"]
#[sub_commands(notify_set, notify_clear)]
#[required_permissions("ADMINISTRATOR")]
pub async fn notify(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let db = data.get_mut::<DataStore>().unwrap().lock().await;
    let guilds = db.get::<HashMap<u64, GuildItem>>("guilds").unwrap_or(HashMap::new());
    let guild = guilds.get(&msg.guild_id.unwrap().0);

    let mut success = false;
    if let Some(guild) = guild {
        if let Some(channel_id) = guild.channel_id {
            if let Ok(channels) = msg.guild_id.unwrap().channels(&ctx).await {
                if let Some(channel) = channels.get(&ChannelId(channel_id)) {
                    msg.reply(&ctx.http, format!("Posting in {channel}")).await?;
                    success = true;
                }
            }
        }
    }

    if !success {
        msg.reply(&ctx.http, format!("Not currently set to post updates.\nUse `^ross notify set` to set the current channel")).await?;
    }

    Ok(())
}

#[command("set")]
#[description("Sets the channel to post updates")]
pub async fn notify_set(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let mut db = data.get_mut::<DataStore>().unwrap().lock().await;
    let mut guilds = db.get::<HashMap<u64, GuildItem>>("guilds").unwrap_or(HashMap::new());
    let mut guild = guilds.entry(msg.guild_id.unwrap().0).or_insert(Default::default());

    guild.channel_id = Some(msg.channel_id.0);

    db.set("guilds", &guilds).unwrap();
    db.dump().expect("failed to save db");

    msg.reply(&ctx.http, format!("Will now post updates in this channel")).await?;

    Ok(())
}

#[command("stop")]
#[aliases("clear", "remove")]
#[description("Clears the notification ID, stopping the bot from posting updates")]
pub async fn notify_clear(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let mut db = data.get_mut::<DataStore>().unwrap().lock().await;
    let mut guilds = db.get::<HashMap<u64, GuildItem>>("guilds").unwrap_or(HashMap::new());
    let mut guild = guilds.entry(msg.guild_id.unwrap().0).or_insert(Default::default());

    guild.channel_id = None;

    db.set("guilds", &guilds).unwrap();
    db.dump().expect("failed to save db");

    msg.reply(&ctx.http, format!("No longer sending updates")).await?;

    Ok(())
}

#[command]
#[description = "Explains the current mention configuration"]
#[bucket = "ross"]
#[sub_commands(ping_set, ping_clear)]
#[required_permissions("ADMINISTRATOR")]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let db = data.get_mut::<DataStore>().unwrap().lock().await;
    let guilds = db.get::<HashMap<u64, GuildItem>>("guilds").unwrap_or(HashMap::new());
    let guild = guilds.get(&msg.guild_id.unwrap().0);

    let mut success = false;
    if let Some(guild) = guild {
        if let Some(role_id) = guild.role_id {
            if let Ok(roles) = msg.guild_id.unwrap().roles(&ctx).await {
                if let Some(role) = roles.get(&RoleId(role_id)) {
                    msg.reply(&ctx.http, format!("Will ping `@{}`", role.name)).await?;
                    success = true;
                }
            }
        }
    }

    if !success {
        msg.reply(&ctx.http, format!("Not currently set to ping.\nUse `^ross ping set <id>` to set a role")).await?;
    }

    Ok(())
}

#[command("set")]
#[description("Sets the ID to ping")]
pub async fn ping_set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let role_id = args.single::<u64>()?;

    let mut data = ctx.data.write().await;
    let mut db = data.get_mut::<DataStore>().unwrap().lock().await;
    let mut guilds = db.get::<HashMap<u64, GuildItem>>("guilds").unwrap_or(HashMap::new());
    let mut guild = guilds.entry(msg.guild_id.unwrap().0).or_insert(Default::default());

    let mut role = None;
    if let Ok(roles) = msg.guild_id.unwrap().roles(&ctx).await {
        if let Some(r) = roles.get(&RoleId(role_id)) {
            role = Some(r.clone());
        }
    }

    guild.role_id = Some(role_id);

    db.set("guilds", &guilds).unwrap();
    db.dump().expect("failed to save db");

    match role {
        Some(role) => msg.reply(&ctx.http, format!("Will now ping {}", role)).await?,
        None => msg.reply(&ctx.http, format!("Will try to ping <@{}>", role_id)).await?,
    };

    Ok(())
}

#[command("stop")]
#[aliases("clear", "remove")]
#[description("Clears the ping ID, stopping the bot from pinging anyone")]
pub async fn ping_clear(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let mut db = data.get_mut::<DataStore>().unwrap().lock().await;
    let mut guilds = db.get::<HashMap<u64, GuildItem>>("guilds").unwrap_or(HashMap::new());
    let guild = guilds.get_mut(&msg.guild_id.unwrap().0);

    let mut message = None;
    if let Some(guild) = guild {
        if let Some(role_id) = guild.role_id {
            let mut worked = true;
            if let Ok(roles) = msg.guild_id.unwrap().roles(&ctx).await {
                if let Some(role) = roles.get(&RoleId(role_id)) {
                    message = Some(format!("No longer pinging `{}`", role.name));
                    worked = true;
                }
            }
            if !worked {
                message = Some(format!("No longer pinging `{}`", role_id));
            }
        }
        guild.role_id = None;
    }

    db.set("guilds", &guilds).unwrap();
    db.dump().expect("failed to save db");

    if let Some(message) = message {
        msg.reply(&ctx.http, message).await?;
    }

    Ok(())
}