pub mod data;
pub mod datetime_eorzea;
pub mod discord;
pub mod ocean;
pub mod status;
pub mod store;
pub mod time;
pub mod weather;

pub mod prelude {
    pub use crate::{
        data::{MAP_INFO, WEATHER_NAMES, WEATHER_RATES},
        discord,
        status::*,
        store::*,
        datetime_eorzea::*,
        time::*,
        weather::*,
    };
}