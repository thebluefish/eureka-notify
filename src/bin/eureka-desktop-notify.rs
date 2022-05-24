use eureka_notify::prelude::*;
use chrono::{Duration, Utc};
use tokio::time::sleep;
use tracing::*;
use tracing_subscriber;
use notify_rust::Notification;
use chrono_humanize::HumanTime;

enum TimeSleep {
    OneCycle,
    FiveMinutes,
    ThirtySeconds,
    Now,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting main loop");

    let mut now = EorzeaDateTime::now().truncated(Duration::hours(8));

    loop {
        let future = now + Duration::hours(8);

        let crab = crab_status(now);
        let crab = if crab == future { Some(crab) } else { None };

        let cassie = cassie_status(now);
        let cassie = if cassie == future { Some(cassie) } else { None };

        let skoll = skoll_status(now);
        let skoll = if skoll == future { Some(skoll) } else { None };

        let do_notify = crab.is_some() || cassie.is_some() || skoll.is_some();

        if do_notify {
            notify_os(TimeSleep::OneCycle, crab, cassie, skoll);

            // Wait until 5 minutes before the next cycle to send another notification
            if let Ok(duration) = ((future.to_utc() - Duration::minutes(5)) - Utc::now()).to_std() {
                info!("sleep for {:?} to 5-minute notification", duration);
                sleep(duration).await;
                notify_os(TimeSleep::FiveMinutes, crab, cassie, skoll);
            }

            // Wait until 30 seconds before the next cycle to send another notification
            if let Ok(duration) = ((future.to_utc() - Duration::seconds(30)) - Utc::now()).to_std() {
                info!("sleep for {:?} to 30-second notification", duration);
                sleep(duration).await;
                notify_os(TimeSleep::ThirtySeconds, crab, cassie, skoll);
            }
        }

        if let Ok(duration) = (future.to_utc() - Utc::now()).to_std() {
            info!("sleep for {:?} to next weather", duration);
            sleep(duration).await;
        }

        if do_notify {
            notify_os(TimeSleep::Now, crab, cassie, skoll);
        }

        now = future;
    }

    Ok(())
}

fn notify_os(timesleep: TimeSleep, crab: Option<EorzeaDateTime>, cassie: Option<EorzeaDateTime>, skoll: Option<EorzeaDateTime>) {
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

    if let Some(dt) = skoll {
        let length = match timesleep {
            TimeSleep::OneCycle => format!("{}", HumanTime::from(dt.to_utc())),
            TimeSleep::FiveMinutes => "in 5 minutes".into(),
            TimeSleep::ThirtySeconds => "in 30 seconds".into(),
            TimeSleep::Now => "now".into(),
        };
        Notification::new()
            .app_id("com.squirrel.XIVLauncher.XIVLauncher")
            .summary("Skoll")
            .body(&format!("{length}"))
            .sound_name("Default")
            .show()
            .expect("failed to open OS notification");
    }
}