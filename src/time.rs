use core::ops::{Add, Sub};
use chrono::{Duration, DateTime, TimeZone};
use chrono::prelude::*;

pub trait TruncateDateTime: Sized {
    /// Rounds time down to the nearest duration
    fn truncate(&mut self, duration: Duration);

    /// Rounds time down to the nearest duration, builder-style
    fn truncated(mut self, duration: Duration) -> Self {
        self.truncate(duration);
        self
    }
}

impl<Tz: TimeZone> TruncateDateTime for DateTime<Tz> {
    fn truncate(&mut self, duration: Duration) {
        *self = self.clone().sub(Duration::seconds(self.timestamp() % duration.num_seconds()));
    }
}