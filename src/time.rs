use chrono::Duration;
use chrono::prelude::*;
use derive_more::{Deref, DerefMut, Display};
use newtype_ops::newtype_ops;
use crate::weather::EorzeaWeatherRate;

const EORZEA_TIME_RATIO: f64 = 3600. / 175.;
const HOUR: f64 = 60. * 60.;
const DAY: f64 = HOUR * 24.;

/// A DateTime aware of Eorzean time scale
/// Provides utility for conversion with other timezones
#[derive(Display, Deref, DerefMut, Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct EorzeaDateTime(pub NaiveDateTime);

// Auto-forwards ops for wrapped type
newtype_ops! { [EorzeaDateTime] {add sub} {:=} {^&}Self ^Duration }

impl EorzeaDateTime {
    /// Get the current Eorzean date & time
    pub fn now() -> Self {
        EorzeaDateTime(NaiveDateTime::from_timestamp((Utc::now().timestamp() as f64 * EORZEA_TIME_RATIO)  as i64, 0))
    }

    /// Converts to UTC time
    pub fn to_utc(self) -> DateTime<Utc> {
        Utc.from_utc_datetime(&NaiveDateTime::from_timestamp((self.timestamp() as f64 / EORZEA_TIME_RATIO)  as i64, 0))
    }

    /// Finds the current weather
    pub fn to_weather_rate(self) -> EorzeaWeatherRate {
        self.into()
    }

    /// Rounds time down to the nearest duration
    pub fn truncate(&mut self, duration: Duration) {
        self.0 -= Duration::seconds(self.timestamp() % duration.num_seconds());
    }

    /// Rounds time down to the nearest duration, builder-style
    pub fn truncated(mut self, duration: Duration) -> Self {
        self.truncate(duration);
        self
    }
}

impl From<EorzeaDateTime> for EorzeaWeatherRate {
    fn from(dt: EorzeaDateTime) -> Self {
        let days = (dt.timestamp() as f64 / DAY).floor() as u32;
        let hours = (dt.timestamp() as f64 / HOUR).floor() as u32;

        // Magic offset aligned to 8 hour increments
        let offset = (hours + 8 - (hours % 8)) % 24;

        // XorShift RNG
        // Seed is a base-10 number in the format DDDDHH
        let mut calc = days * 100 + offset;
        calc ^= calc << 11;
        calc ^= calc >> 8;
        calc %= 100;

        EorzeaWeatherRate(calc as usize)
    }
}