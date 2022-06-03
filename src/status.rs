use crate::datetime_eorzea::DateTimeEorzea;
use crate::weather::{EorzeaMap, EorzeaWeather};
use chrono::Duration;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Direction {
    Future,
    Past,
}

impl Direction {
    pub fn next(self, dt: DateTimeEorzea) -> DateTimeEorzea {
        match self {
            Direction::Future => dt + Duration::hours(8),
            Direction::Past => dt - Duration::hours(8),
        }
    }
}

pub fn crab_status(mut now: DateTimeEorzea, direction: Direction) -> DateTimeEorzea {
    let zone = EorzeaMap::from_name("Eureka Pagos").expect("Could not find map");

    // Find next non-Fog
    if direction == Direction::Future {
        while zone.weather(now).name == "Fog" {
            now = direction.next(now);
        }
    }

    // Fog after is our target
    while zone.weather(now).name != "Fog" {
        now = direction.next(now);
    }

    now
}

pub fn cassie_status(mut now: DateTimeEorzea, direction: Direction) -> DateTimeEorzea {
    let zone = EorzeaMap::from_name("Eureka Pagos").expect("Could not find map");

    // Find next non-Blizzards
    if direction == Direction::Future {
        while zone.weather(now).name == "Blizzards" {
            now = direction.next(now);
        }
    }

    // Blizzards after is our target
    while zone.weather(now).name != "Blizzards" {
        now = direction.next(now);
    }

    now
}

pub fn skoll_status(mut now: DateTimeEorzea, direction: Direction) -> DateTimeEorzea {
    let zone = EorzeaMap::from_name("Eureka Pyros").expect("Could not find map");

    // Find next non-Blizzards
    if direction == Direction::Future {
        while zone.weather(now).name == "Blizzards" {
            now = direction.next(now);
        }
    }

    // Blizzards after is our target
    while zone.weather(now).name != "Blizzards" {
        now = direction.next(now);
    }

    now
}

fn is_hotbox_weather(weather: EorzeaWeather) -> bool {
    weather.name == "Snow" || weather.name == "Blizzards" || weather.name == "Umbral Wind"
}

pub fn hotbox_status(mut now: DateTimeEorzea) -> (DateTimeEorzea, usize) {
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
        let mut next = now + Duration::hours(8);

        // Count total back-to-back hotbox weathers
        while is_hotbox_weather(zone.weather(next)) {
            count += 1;
            next += Duration::hours(8);
        }

        // Break once we have multiple good weathers
        if count > 1 {
            break;
        }

        now += Duration::hours(8);
    }
    (now, count)
}

pub fn offensive_status(mut now: DateTimeEorzea) -> (DateTimeEorzea, usize) {
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
        let mut next = now + Duration::hours(8);

        // Count total back-to-back hotbox weathers
        while zone.weather(next).name == "Snow" {
            count += 1;
            next += Duration::hours(8);
        }

        // Break once we have multiple good weathers
        if count > 1 {
            break;
        }

        now += Duration::hours(8);
    }
    (now, count)
}