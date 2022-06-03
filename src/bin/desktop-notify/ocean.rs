use super::APP_ID;
use eureka_notify::{prelude::*, ocean::*};
use chrono::{DateTime, Duration, TimeZone, Utc};
use tokio::time::sleep;
use tracing::*;
use notify_rust::Notification;
use chrono_humanize::HumanTime;

enum TimeSleep {
    OneCycle,
    FifteenMinutes,
    Now,
    LastCall,
}

pub async fn run_loop() {
    info!("Starting ocean loop");

    let mut now = Utc::now().truncated(Duration::hours(2));

    // Handle special case where we start between XX:00 and XX:15
    if let Ok(duration) = ((now + Duration::minutes(15)) - Utc::now()).to_std() {
        let route = Route::from_datetime(now);
        let do_notify = BEST_ROUTES.contains(&route) || GOOD_ROUTES.contains(&route);

        if do_notify {
            info!("sleep for {:?} to last-call notification", duration);
            sleep(duration).await;
            notify_os(TimeSleep::LastCall, route, now);
        }
    }

    loop {
        let future = now + Duration::hours(2);

        let route = Route::from_datetime(future);
        let do_notify = BEST_ROUTES.contains(&route) || GOOD_ROUTES.contains(&route);

        if do_notify {
            notify_os(TimeSleep::OneCycle, route, future);

            // Wait until 15 minutes before the route opens
            if let Ok(duration) = ((future - Duration::minutes(15)) - Utc::now()).to_std() {
                info!("sleep for {:?} to 15-min notification", duration);
                sleep(duration).await;
                notify_os(TimeSleep::FifteenMinutes, route, future);
            }
        }

        if let Ok(duration) = (future - Utc::now()).to_std() {
            info!("sleep for {:?} to next ocean trip", duration);
            sleep(duration).await;
        }

        if do_notify {
            notify_os(TimeSleep::Now, route, future);
        }

        // Wait until 2 minutes before the ship closes
        if let Ok(duration) = ((future + Duration::minutes(13)) - Utc::now()).to_std() {
            info!("sleep for {:?} to last-call notification", duration);
            sleep(duration).await;
            notify_os(TimeSleep::LastCall, route, future);
        }

        now = future;
    }
}

fn notify_os<Tz: TimeZone>(timesleep: TimeSleep, route: Route, dt: DateTime<Tz>) {
    info!("sending notification for {route:?}");
    let length = match timesleep {
        TimeSleep::OneCycle => format!("{}", HumanTime::from(dt)),
        TimeSleep::FifteenMinutes => "in 15 minutes".into(),
        TimeSleep::Now => "now".into(),
        TimeSleep::LastCall => "closes in 2 minutes".into(),
    };
    Notification::new()
        .app_id(APP_ID)
        .summary(route.to_name())
        .body(&format!("{length}"))
        .sound_name("Default")
        .show()
        .expect("failed to open OS notification");
}