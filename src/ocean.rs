use crate::time::*;
use chrono::{DateTime, Duration, DurationRound, Utc};
use tracing::info;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Route {
    BloodbrineDay,
    RothlytDay,
    MerlthorDay,
    RhotanoDay,
    BloodbrineSunset,
    RothlytSunset,
    MerlthorSunset,
    RhotanoSunset,
    BloodbrineNight,
    RothlytNight,
    MerlthorNight,
    RhotanoNight,
}

impl Route {
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        let duration = Duration::hours(2);

        // Calculate index based off magic offset
        let floored = dt.clone().truncated(duration);
        let voyage = OFFSET + (floored.timestamp() / duration.num_seconds()) as usize;

        // Find in table
        PATTERN[voyage % 144]
    }

    pub fn to_name(self) -> &'static str {
        match self {
            Route::BloodbrineDay => "Crab / Seafaring Toad",
            Route::RothlytDay => "Fugu / Mantas",
            Route::MerlthorDay => "Sothis / Elasmosaurus",
            Route::RhotanoDay => "Shark / Coral Manta",
            Route::BloodbrineSunset => "Hafgufa / Elasmosaurus",
            Route::RothlytSunset => "Hafgufa / Placodus",
            Route::MerlthorSunset => "Seadragons / Coral Manta",
            Route::RhotanoSunset => "Sothis / Stonescale",
            Route::BloodbrineNight => "Mantas",
            Route::RothlytNight => "Fugu / Stonescale",
            Route::MerlthorNight => "Octopodes",
            Route::RhotanoNight => "Jellyfish",
        }
    }
}

pub enum TimePeriod {
    Day,
    Sunset,
    Night,
}

pub const BEST_ROUTES: [Route; 4] = [Route::BloodbrineDay, Route::RothlytDay, Route::RhotanoDay, Route::RhotanoNight];
pub const VERY_GOOD_ROUTES: [Route; 4] = [Route::RothlytSunset, Route::MerlthorSunset, Route::RothlytNight, Route::MerlthorNight];
pub const GOOD_ROUTES: [Route; 1] = [Route::BloodbrineNight];
pub const BAD_ROUTES: [Route; 3] = [Route::MerlthorDay, Route::BloodbrineSunset, Route::RhotanoSunset];


const OFFSET: usize = 88;
const PATTERN: [Route; 144] = [
    Route::BloodbrineDay, Route::RothlytDay, Route::MerlthorDay, Route::RhotanoDay, Route::BloodbrineSunset, Route::RothlytSunset, Route::MerlthorSunset, Route::RhotanoSunset, Route::BloodbrineNight, Route::RothlytNight, Route::MerlthorNight, Route::RhotanoNight,
    Route::RothlytDay, Route::MerlthorDay, Route::RhotanoDay, Route::BloodbrineSunset, Route::RothlytSunset, Route::MerlthorSunset, Route::RhotanoSunset, Route::BloodbrineNight, Route::RothlytNight, Route::MerlthorNight, Route::RhotanoNight, Route::BloodbrineDay,
    Route::MerlthorDay, Route::RhotanoDay, Route::BloodbrineSunset, Route::RothlytSunset, Route::MerlthorSunset, Route::RhotanoSunset, Route::BloodbrineNight, Route::RothlytNight, Route::MerlthorNight, Route::RhotanoNight, Route::BloodbrineDay, Route::RothlytDay,
    Route::RhotanoDay, Route::BloodbrineSunset, Route::RothlytSunset, Route::MerlthorSunset, Route::RhotanoSunset, Route::BloodbrineNight, Route::RothlytNight, Route::MerlthorNight, Route::RhotanoNight, Route::BloodbrineDay, Route::RothlytDay, Route::MerlthorDay,
    Route::BloodbrineSunset, Route::RothlytSunset, Route::MerlthorSunset, Route::RhotanoSunset, Route::BloodbrineNight, Route::RothlytNight, Route::MerlthorNight, Route::RhotanoNight, Route::BloodbrineDay, Route::RothlytDay, Route::MerlthorDay, Route::RhotanoDay,
    Route::RothlytSunset, Route::MerlthorSunset, Route::RhotanoSunset, Route::BloodbrineNight, Route::RothlytNight, Route::MerlthorNight, Route::RhotanoNight, Route::BloodbrineDay, Route::RothlytDay, Route::MerlthorDay, Route::RhotanoDay, Route::BloodbrineSunset,
    Route::MerlthorSunset, Route::RhotanoSunset, Route::BloodbrineNight, Route::RothlytNight, Route::MerlthorNight, Route::RhotanoNight, Route::BloodbrineDay, Route::RothlytDay, Route::MerlthorDay, Route::RhotanoDay, Route::BloodbrineSunset, Route::RothlytSunset,
    Route::RhotanoSunset, Route::BloodbrineNight, Route::RothlytNight, Route::MerlthorNight, Route::RhotanoNight, Route::BloodbrineDay, Route::RothlytDay, Route::MerlthorDay, Route::RhotanoDay, Route::BloodbrineSunset, Route::RothlytSunset, Route::MerlthorSunset,
    Route::BloodbrineNight, Route::RothlytNight, Route::MerlthorNight, Route::RhotanoNight, Route::BloodbrineDay, Route::RothlytDay, Route::MerlthorDay, Route::RhotanoDay, Route::BloodbrineSunset, Route::RothlytSunset, Route::MerlthorSunset, Route::RhotanoSunset,
    Route::RothlytNight, Route::MerlthorNight, Route::RhotanoNight, Route::BloodbrineDay, Route::RothlytDay, Route::MerlthorDay, Route::RhotanoDay, Route::BloodbrineSunset, Route::RothlytSunset, Route::MerlthorSunset, Route::RhotanoSunset, Route::BloodbrineNight,
    Route::MerlthorNight, Route::RhotanoNight, Route::BloodbrineDay, Route::RothlytDay, Route::MerlthorDay, Route::RhotanoDay, Route::BloodbrineSunset, Route::RothlytSunset, Route::MerlthorSunset, Route::RhotanoSunset, Route::BloodbrineNight, Route::RothlytNight,
    Route::RhotanoNight, Route::BloodbrineDay, Route::RothlytDay, Route::MerlthorDay, Route::RhotanoDay, Route::BloodbrineSunset, Route::RothlytSunset, Route::MerlthorSunset, Route::RhotanoSunset, Route::BloodbrineNight, Route::RothlytNight, Route::MerlthorNight
];