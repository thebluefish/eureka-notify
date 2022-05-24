use derive_more::{Deref, DerefMut, Display};
use crate::data::*;
use tracing::{info, debug};

/// Zone-independent weather value
/// Use to index into a weather rate table
#[derive(Deref, DerefMut, Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct EorzeaWeatherRate(pub usize);

/// Zone-aware weather value
#[derive(Display, Clone, Eq, PartialEq, Debug, Hash)]
#[display(fmt = "{}", name)]
pub struct EorzeaWeather {
    pub name: String,
    pub id: usize,
}

impl EorzeaWeather {
    /// Find a weather from its English name
    pub fn from_name(name: &str) -> Option<Self> {
        if let Some((id, name)) = WEATHER_NAMES.iter().find(|(_, wname) | wname.en == name) {
            Some(EorzeaWeather {
                name: name.en.clone(),
                id: *id,
            })
        }
        else {
            None
        }
    }
}

/// Map & zone info
#[derive(Display, Clone, Eq, PartialEq, Debug)]
#[display(fmt = "{}", name)]
pub struct EorzeaMap {
    name: String,
    id: usize,
    pub weathers: Vec<WeatherRate>,
}

impl EorzeaMap {
    pub fn from_name(name: &str) -> Option<Self> {
        let map = MAP_INFO.iter().find(|&info| info.name == name);
        if let Some(map) = map {
            debug!("got map {:?}", map);
            let weathers = if let Some(rates) = WEATHER_RATES.get(&map.weather_rate) {
                rates.clone()
            }
            else {
                info!("failed to get rate map for {}", name);
                vec![]
            };
            Some(EorzeaMap {
                name: map.name.clone(),
                id: map.id,
                weathers,
            })
        }
        else {
            None
        }
    }

    /// Get this zone's weather
    pub fn weather<W: Into<EorzeaWeatherRate>>(&self, rate: W) -> EorzeaWeather {
        let rate = rate.into().0;
        let weather = self.weathers.iter().find(|&o| rate < o.rate).unwrap();
        EorzeaWeather {
            name: WEATHER_NAMES.get(&weather.weather_id).expect("Missing weather from map").en.clone(),
            id: weather.weather_id,
        }
    }
}