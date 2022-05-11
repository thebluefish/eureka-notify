use eureka_notify::prelude::*;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use chrono::{Duration, Utc};
use derive_more::{Deref, DerefMut};
use tokio::time::sleep;
use tracing::*;
use tracing_subscriber;
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use lazy_static::lazy_static;

// Globals loaded from environment vars
lazy_static! {
    pub static ref DISCORD_TOKEN: String = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in env");
    pub static ref CHANNEL_ID: u64 = env::var("CHANNEL_ID").expect("Expected CHANNEL_ID in env").parse().expect("Invalid CHANNEL_ID");
}

#[derive(Default, Deref, DerefMut)]
struct Handler(AtomicBool);

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
    tracing_subscriber::fmt::init();

    // Optionally load the .env file
    if let Err(e) = dotenv::dotenv() {
        error!("failed to load .env file: {e}");
    }


    let mut client = Client::builder(&*DISCORD_TOKEN, GatewayIntents::GUILD_MESSAGES)
        .event_handler(Handler(false.into()))
        .await
        .expect("Failed to create client")
        ;

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
    Ok(())
}

async fn run_main_loop(ctx: Context) {
    let pagos = EorzeaMap::from_name("Eureka Pagos").expect("Could not find map");
    let pyros = EorzeaMap::from_name("Eureka Pyros").expect("Could not find map");
    let hydatos = EorzeaMap::from_name("Eureka Hydatos").expect("Could not find map");

    let mut now = EorzeaDateTime::now().truncated(Duration::hours(8));
    loop {
        let future = now + Duration::hours(8);
        let future1 = future + Duration::hours(8);
        let future2 = future1 + Duration::hours(8);

        let speed_belt = speed_belt(now).to_utc();
        let cassie_earring = cassie_earring(now).to_utc();
        let skoll_claw = skoll_claw(now).to_utc();
        let (hotbox, hotbox_count) = hotbox_farm(now);
        let hotbox = hotbox.to_utc();
        let (offensive, offensive_count) = offensive_farm(now);
        let offensive = offensive.to_utc();

        let message = ChannelId(CHANNEL_ID.clone())
            .send_message(&ctx, |m| {
                m
                    .add_embed(|e| {
                        e
                            .field(format!("Current weather started <t:{}:R>", now.to_utc().timestamp()), format!("<t:{}>", now.to_utc().timestamp()), true)
                    })
                    .add_embed(|e| {
                        e.title(format!("Eureka Pagos: {}", pagos.weather(now)))
                            .field(format!("Speed belt <t:{}:R>", speed_belt.timestamp()), format!("<t:{}>", speed_belt.timestamp()), false)
                            .field(format!("Cassie's earring <t:{}:R>", cassie_earring.timestamp()), format!("<t:{}>", cassie_earring.timestamp()), false)

                            .field(format!("{} <t:{}:R>", pagos.weather(future), future.to_utc().timestamp()), format!("<t:{}>", future.to_utc().timestamp()), true)
                            .field(format!("{} <t:{}:R>", pagos.weather(future1), future1.to_utc().timestamp()), format!("<t:{}>", future1.to_utc().timestamp()), true)
                            .field(format!("{} <t:{}:R>", pagos.weather(future2), future2.to_utc().timestamp()), format!("<t:{}>\u{200B}", future2.to_utc().timestamp()), true)

                    })
                    .add_embed(|e| {
                        e.title(format!("Eureka Pyros: {}", pyros.weather(now)))
                            .field(format!("Skoll's Claw <t:{}:R>", skoll_claw.timestamp()), format!("<t:{}>", skoll_claw.timestamp()), false)
                            .field(format!("Hotbox x{} <t:{}:R>", hotbox_count, hotbox.timestamp()), format!("<t:{}>", hotbox.timestamp()), false)

                            .field(format!("{} <t:{}:R>", pyros.weather(future), future.to_utc().timestamp()), format!("<t:{}>", future.to_utc().timestamp()), true)
                            .field(format!("{} <t:{}:R>", pyros.weather(future1), future1.to_utc().timestamp()), format!("<t:{}>", future1.to_utc().timestamp()), true)
                            .field(format!("{} <t:{}:R>", pyros.weather(future2), future2.to_utc().timestamp()), format!("<t:{}>", future2.to_utc().timestamp()), true)

                    })
                    .add_embed(|e| {
                        e.title(format!("Eureka Hydatos: {}", hydatos.weather(now)))
                            .field(format!("Offensive x{} <t:{}:R>", offensive_count, offensive.timestamp()), format!("<t:{}>", offensive.timestamp()), false)

                            .field(format!("{} <t:{}:R>", hydatos.weather(future), future.to_utc().timestamp()), format!("<t:{}>", future.to_utc().timestamp()), true)
                            .field(format!("{} <t:{}:R>", hydatos.weather(future1), future1.to_utc().timestamp()), format!("<t:{}>", future1.to_utc().timestamp()), true)
                            .field(format!("{} <t:{}:R>", hydatos.weather(future2), future2.to_utc().timestamp()), format!("<t:{}>", future2.to_utc().timestamp()), true)
                    })
            })
            .await;
        if let Err(why) = message {
            eprintln!("Error sending message: {:?}", why);
        };

        if let Ok(duration) = (future.to_utc() - Utc::now()).to_std() {
            sleep(duration).await;
        }
        now = future;
    }
}

pub fn is_hotbox_weather(weather: EorzeaWeather) -> bool {
    weather.name == "Snow" || weather.name == "Blizzards" || weather.name == "Umbral Wind"
}

pub fn hotbox_farm(mut now: EorzeaDateTime) -> (EorzeaDateTime, usize) {
    let zone = EorzeaMap::from_name("Eureka Pyros").expect("Could not find map");

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

pub fn offensive_farm(mut now: EorzeaDateTime) -> (EorzeaDateTime, usize) {
    let zone = EorzeaMap::from_name("Eureka Hydatos").expect("Could not find map");

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

pub fn speed_belt(mut now: EorzeaDateTime) -> EorzeaDateTime {
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

pub fn cassie_earring(mut now: EorzeaDateTime) -> EorzeaDateTime {
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

pub fn skoll_claw(mut now: EorzeaDateTime) -> EorzeaDateTime {
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