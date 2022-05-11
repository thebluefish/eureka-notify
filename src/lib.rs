pub mod data;
pub mod time;
pub mod weather;

pub mod prelude {
    pub use crate::{
        data::{MAP_INFO, WEATHER_NAMES, WEATHER_RATES},
        time::*,
        weather::*,
    };
}