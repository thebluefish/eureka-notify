mod commands;

use chrono::Duration;
pub use commands::*;

use crate::status::*;
use crate::datetime_eorzea::DateTimeEorzea;
use crate::weather::EorzeaMap;
use tracing::*;
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

#[hook]
pub async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError, _command_name: &str) {
    if let DispatchError::Ratelimited(info) = error {
        // We notify them only once
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
                .await;
        }
    }
    else {
        error!("uncaught dispatch error: {:?}", error);
    }
}

/// Attempts to delete an existing message without checking if it worked
pub async fn delete_post(ctx: &Context, channel_id: u64, id: u64) {
    ChannelId(channel_id).delete_message(&ctx, MessageId(id)).await.ok();
}

pub async fn edit_post(ctx: &Context, channel_id: u64, id: u64, now: DateTimeEorzea) {
    let pagos = EorzeaMap::from_name("Eureka Pagos").expect("Could not find map");
    let pyros = EorzeaMap::from_name("Eureka Pyros").expect("Could not find map");
    let hydatos = EorzeaMap::from_name("Eureka Hydatos").expect("Could not find map");
    let future = now + Duration::hours(8);

    // We will post the next crab timer when this is crab weather *or* crab weather is next
    let crab = crab_status(now, Direction::Future);
    let past_crab = crab_status(now, Direction::Past);

    let cassie = cassie_status(now, Direction::Future);
    let past_cassie = cassie_status(now, Direction::Past);

    let skoll = skoll_status(now, Direction::Future);
    let past_skoll = skoll_status(now, Direction::Past);

    let result = ChannelId(channel_id).edit_message(&ctx, id, |m| {
        m.content("");
        if past_crab == now || past_cassie == now || past_skoll == now {
            m.add_embed(|e| {
                e
                    .field(format!("Crab <t:{}:R>", crab.to_utc().timestamp()), format!("Prev <t:{}:R>", past_crab.to_utc().timestamp()), true)
                    .field(format!("Cassie <t:{}:R>", cassie.to_utc().timestamp()), format!("Prev <t:{}:R>", past_cassie.to_utc().timestamp()), true)
                    .field(format!("Skoll <t:{}:R>", skoll.to_utc().timestamp()), format!("Prev <t:{}:R>", past_skoll.to_utc().timestamp()), true)
            });
        } else if crab == future || cassie == future || skoll == future {
            m.add_embed(|e| {
                if crab == future {
                    e.field(format!("Crab <t:{}:R>", crab.to_utc().timestamp()), format!("Prev <t:{}:R>", past_crab.to_utc().timestamp()), true);
                }
                if cassie == future {
                    e.field(format!("Cassie <t:{}:R>", cassie.to_utc().timestamp()), format!("Prev <t:{}:R>", past_cassie.to_utc().timestamp()), true);
                }
                if skoll == future {
                    e.field(format!("Skoll <t:{}:R>", skoll.to_utc().timestamp()), format!("Prev <t:{}:R>", past_skoll.to_utc().timestamp()), true);
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
pub async fn post_discord(ctx: &Context, channel_id: u64, role_id: Option<u64>, now: DateTimeEorzea) -> Option<u64> {
    let pagos = EorzeaMap::from_name("Eureka Pagos").expect("Could not find map");
    let pyros = EorzeaMap::from_name("Eureka Pyros").expect("Could not find map");
    let hydatos = EorzeaMap::from_name("Eureka Hydatos").expect("Could not find map");
    let future = now + Duration::hours(8);

    // We will post the next crab timer when this is crab weather *or* crab weather is next
    let crab = crab_status(now, Direction::Future);
    let past_crab = crab_status(now, Direction::Past);

    let cassie = cassie_status(now, Direction::Future);
    let past_cassie = cassie_status(now, Direction::Past);

    let skoll = skoll_status(now, Direction::Future);
    let past_skoll = skoll_status(now, Direction::Past);

    let message = ChannelId(channel_id)
        .send_message(&ctx, |m| {
            // Notify when futures are near
            if crab == future || cassie == future || skoll == future {
                if let Some(role_id) = role_id {
                    m.content(RoleId(role_id.clone()).mention());
                }
            }
            if past_crab == now || past_cassie == now || past_skoll == now {
                m.add_embed(|e| {
                    e
                        .field(format!("Crab <t:{}:R>", crab.to_utc().timestamp()), format!("Prev <t:{}:R>", past_crab.to_utc().timestamp()), true)
                        .field(format!("Cassie <t:{}:R>", cassie.to_utc().timestamp()), format!("Prev <t:{}:R>", past_cassie.to_utc().timestamp()), true)
                        .field(format!("Skoll <t:{}:R>", skoll.to_utc().timestamp()), format!("Prev <t:{}:R>", past_skoll.to_utc().timestamp()), true)
                });
            } else if crab == future || cassie == future || skoll == future {
                m.add_embed(|e| {
                    if crab == future {
                        e.field(format!("Crab <t:{}:R>", crab.to_utc().timestamp()), format!("Prev <t:{}:R>", past_crab.to_utc().timestamp()), true);
                    }
                    if cassie == future {
                        e.field(format!("Cassie <t:{}:R>", cassie.to_utc().timestamp()), format!("Prev <t:{}:R>", past_cassie.to_utc().timestamp()), true);
                    }
                    if skoll == future {
                        e.field(format!("Skoll <t:{}:R>", skoll.to_utc().timestamp()), format!("Prev <t:{}:R>", past_skoll.to_utc().timestamp()), true);
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

pub async fn edit_notification(ctx: &Context, channel_id: u64, id: u64, now: DateTimeEorzea) {
    let crab = crab_status(now, Direction::Future);
    let past_crab = crab_status(now, Direction::Past);

    let cassie = cassie_status(now, Direction::Future);
    let past_cassie = cassie_status(now, Direction::Past);

    let skoll = skoll_status(now, Direction::Future);
    let past_skoll = skoll_status(now, Direction::Past);

    let result = ChannelId(channel_id).edit_message(&ctx, id, |m| {
        m.content("")
         .add_embed(|e| {
            if crab == now {
                e.field(format!("Crab <t:{}:R>", crab.to_utc().timestamp()), format!("Prev <t:{}:R>", past_crab.to_utc().timestamp()), false);
            }
            if cassie == now {
                e.field(format!("Cassie <t:{}:R>", cassie.to_utc().timestamp()), format!("Prev <t:{}:R>", past_cassie.to_utc().timestamp()), false);
            }
            if skoll == now {
                e.field(format!("Skoll <t:{}:R>", skoll.to_utc().timestamp()), format!("Prev <t:{}:R>", past_skoll.to_utc().timestamp()), false);
            }
            e
        })
    }).await;
    if let Err(err) = result {
        eprintln!("Error editing discord notification: {err:?}");
    };
}

// We will post the next crab timer when this is crab weather *or* crab weather is next
pub async fn notify_discord(ctx: &Context, channel_id: u64, role_id: Option<u64>, now: DateTimeEorzea) -> Option<u64> {
    let crab = crab_status(now, Direction::Future);
    let past_crab = crab_status(now, Direction::Past);

    let cassie = cassie_status(now, Direction::Future);
    let past_cassie = cassie_status(now, Direction::Past);

    let skoll = skoll_status(now, Direction::Future);
    let past_skoll = skoll_status(now, Direction::Past);

    let message = ChannelId(channel_id)
        .send_message(&ctx, |m| {
            if let Some(role_id) = role_id {
                m.content(RoleId(role_id.clone()).mention());
            }
            m.add_embed(|e| {
                if crab == now {
                    e.field(format!("Crab <t:{}:R>", crab.to_utc().timestamp()), format!("Prev <t:{}:R>", past_crab.to_utc().timestamp()), false);
                }
                if cassie == now {
                    e.field(format!("Cassie <t:{}:R>", cassie.to_utc().timestamp()), format!("Prev <t:{}:R>", past_cassie.to_utc().timestamp()), false);
                }
                if skoll == now {
                    e.field(format!("Skoll <t:{}:R>", skoll.to_utc().timestamp()), format!("Prev <t:{}:R>", past_skoll.to_utc().timestamp()), false);
                }
                e
            });
            m
        })
        .await;

    match message {
        Ok(msg) => Some(msg.id.0),
        Err(err) => {
            error!("Error sending notification: {err:?}");
            None
        }
    }
}
