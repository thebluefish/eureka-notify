use crate::time::EorzeaDateTime;
use crate::weather::{EorzeaMap, EorzeaWeather};
use chrono::Duration;

pub fn crab_status(mut now: EorzeaDateTime) -> EorzeaDateTime {
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

pub fn cassie_status(mut now: EorzeaDateTime) -> EorzeaDateTime {
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

pub fn skoll_status(mut now: EorzeaDateTime) -> EorzeaDateTime {
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


fn is_hotbox_weather(weather: EorzeaWeather) -> bool {
    weather.name == "Snow" || weather.name == "Blizzards" || weather.name == "Umbral Wind"
}

pub fn hotbox_status(mut now: EorzeaDateTime) -> (EorzeaDateTime, usize) {
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

pub fn offensive_status(mut now: EorzeaDateTime) -> (EorzeaDateTime, usize) {
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