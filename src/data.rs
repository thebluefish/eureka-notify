use std::collections::HashMap;
use std::path::Path;
use derive_more::{Deref, DerefMut, Display};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use lazy_static::lazy_static;

// Global file references for loading in data
lazy_static! {
    pub static ref WEATHER_NAMES: WeatherNameMap = WeatherNameMap::from_file("data/weathers.json");
    pub static ref WEATHER_RATES: WeatherRateMap = WeatherRateMap::from_file("data/weather-index.json");
    pub static ref MAP_INFO: MapInfoMap = MapInfoMap::from_file("data/map-ids.json");
}

/// Maps weather rate patterns, corresponding to each map's `weather_rate` field
#[derive(serde::Deserialize, Deref, DerefMut, Clone, Eq, PartialEq, Debug)]
pub struct WeatherRateMap(pub HashMap<usize, Vec<WeatherRate>>);

#[derive(serde::Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct WeatherRate {
    pub rate: usize,
    #[serde(rename = "weatherId")]
    #[serde(deserialize_with = "deserialize_null_default")]
    pub weather_id: usize,
}

/// Various information about maps and zones
#[derive(serde::Deserialize, Deref, DerefMut, Clone, Eq, PartialEq, Debug)]
pub struct MapInfoMap(pub Vec<MapInfo>);

#[derive(serde::Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct MapInfo {
    #[serde(deserialize_with = "deserialize_null_default")]
    pub name: String,
    pub id: usize,
    pub zone: usize,
    pub territory: usize,
    pub scale: usize,
    #[serde(rename = "weatherRate")]
    #[serde(deserialize_with = "deserialize_null_default")]
    pub weather_rate: usize,
}

/// Maps weather ID to string names
#[derive(serde::Deserialize, Deref, DerefMut, Clone, Eq, PartialEq, Debug)]
pub struct WeatherNameMap(pub HashMap<usize, WeatherName>);

#[derive(serde::Deserialize, Display, Deref, DerefMut, Clone, Eq, PartialEq, Debug)]
#[display(fmt = "{}", name)]
pub struct WeatherName {
    pub name: WeatherNameInner,
}

#[derive(serde::Deserialize, Display, Clone, Eq, PartialEq, Debug)]
#[display(fmt = "{}", en)]
pub struct WeatherNameInner {
    pub en: String,
    pub ja: String,
    pub de: String,
    pub fr: String,
}

/// Provides support for loading a Deserializable object from file
pub trait DataMap: Sized + DeserializeOwned {
    fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let data = std::fs::read_to_string(path).expect("Unable to read file");
        serde_json::from_str(&data).expect("Unable to parse map")
    }
}

impl DataMap for MapInfoMap {}
impl DataMap for WeatherNameMap {}
impl DataMap for WeatherRateMap {}

/// Auto-converts null values to Default values
fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        T: Default + Deserialize<'de>,
        D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}