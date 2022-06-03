use super::APP_ID;
use eureka_notify::prelude::*;
use chrono::{Duration, Utc};
use tokio::time::sleep;
use tracing::*;
use notify_rust::Notification;
use chrono_humanize::HumanTime;

enum TimeSleep {
    OneCycle,
    FiveMinutes,
    OneMinute,
    Now,
}

pub async fn run_loop() {
    info!("Starting eureka loop");

    let mut now = DateTimeEorzea::now().truncated(Duration::hours(8));

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

            // Wait to send another notification
            if let Ok(duration) = ((future.to_utc() - Duration::minutes(1)) - Utc::now()).to_std() {
                info!("sleep for {:?} to 1-min notification", duration);
                sleep(duration).await;
                notify_os(TimeSleep::OneMinute, crab, cassie, skoll);
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
}

fn notify_os(timesleep: TimeSleep, crab: Option<DateTimeEorzea>, cassie: Option<DateTimeEorzea>, skoll: Option<DateTimeEorzea>) {
    if let Some(dt) = crab {
        let length = match timesleep {
            TimeSleep::OneCycle => format!("{}", HumanTime::from(dt.to_utc())),
            TimeSleep::FiveMinutes => "in 5 minutes".into(),
            TimeSleep::OneMinute => "in 1 minute".into(),
            TimeSleep::Now => "now".into(),
        };
        Notification::new()
            .app_id(APP_ID)
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
            TimeSleep::OneMinute => "in 1 minute".into(),
            TimeSleep::Now => "now".into(),
        };
        Notification::new()
            .app_id(APP_ID)
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
            TimeSleep::OneMinute => "in 30 seconds".into(),
            TimeSleep::Now => "now".into(),
        };
        Notification::new()
            .app_id(APP_ID)
            .summary("Skoll")
            .body(&format!("{length}"))
            .sound_name("Default")
            .show()
            .expect("failed to open OS notification");
    }
}