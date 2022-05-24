pub mod data;
pub mod discord;
pub mod status;
pub mod time;
pub mod weather;

pub mod prelude {
    pub use crate::{
        data::{MAP_INFO, WEATHER_NAMES, WEATHER_RATES},
        discord,
        status::*,
        time::*,
        weather::*,
    };
}