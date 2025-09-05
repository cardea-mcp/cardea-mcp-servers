use serde::{Deserialize, Serialize};

/// Represents the complete weather response from the OpenWeatherMap API.
/// This struct contains all the weather data for a specific location.
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct WeatherResponse {
    /// Geographic coordinates of the location.
    pub(crate) coord: Coord,
    /// List of weather conditions (e.g., rain, clear sky).
    pub(crate) weather: Vec<Weather>,
    /// Internal parameter indicating the data source.
    pub(crate) base: String,
    /// Main weather parameters like temperature and pressure.
    pub(crate) main: Main,
    /// Visibility in meters (maximum 10 km).
    pub(crate) visibility: u32,
    /// Wind information including speed and direction.
    pub(crate) wind: Wind,
    /// Precipitation information for rain (if available).
    pub(crate) rain: Option<Rain>,
    /// Precipitation information for snow (if available).
    pub(crate) snow: Option<Snow>,
    /// Cloudiness percentage.
    pub(crate) clouds: Clouds,
    /// Time of data calculation in Unix UTC timestamp.
    pub(crate) dt: u64,
    /// System information including country and sunrise/sunset times.
    pub(crate) sys: Sys,
    /// Shift in seconds from UTC for the location's timezone.
    pub(crate) timezone: i32,
    /// City ID for internal use.
    pub(crate) id: u64,
    /// City name.
    pub(crate) name: String,
    /// Internal parameter indicating response code.
    pub(crate) cod: u32,
}

/// Geographic coordinates of the location.
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Coord {
    /// Longitude of the location.
    pub(crate) lon: f64,
    /// Latitude of the location.
    pub(crate) lat: f64,
}

/// Weather condition details.
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Weather {
    /// Weather condition ID.
    pub(crate) id: u32,
    /// Group of weather parameters (e.g., Rain, Snow, Clouds).
    pub(crate) main: String,
    /// Description of the weather condition.
    pub(crate) description: String,
    /// Weather icon ID.
    pub(crate) icon: String,
}

/// Main weather parameters including temperature and atmospheric data.
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Main {
    /// Temperature in Celsius (or Kelvin/Fahrenheit based on API units).
    pub(crate) temp: f64,
    /// Temperature accounting for human perception in Celsius.
    pub(crate) feels_like: f64,
    /// Minimum temperature observed in Celsius.
    pub(crate) temp_min: f64,
    /// Maximum temperature observed in Celsius.
    pub(crate) temp_max: f64,
    /// Atmospheric pressure on the ground level in hPa.
    pub(crate) pressure: u32,
    /// Humidity percentage.
    pub(crate) humidity: u32,
    /// Atmospheric pressure on the sea level in hPa.
    pub(crate) sea_level: u32,
    /// Atmospheric pressure on the ground level in hPa.
    pub(crate) grnd_level: u32,
}

/// Wind information.
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Wind {
    /// Wind speed in meter/sec.
    pub(crate) speed: f64,
    /// Wind direction in degrees (meteorological).
    pub(crate) deg: u32,
    /// Wind gust in meter/sec.
    pub(crate) gust: f64,
}

/// Rain precipitation information.
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Rain {
    /// Precipitation volume for the last 1 hour in mm/h.
    #[serde(rename = "1h")]
    pub(crate) one_hour: f64,
}

/// Snow precipitation information.
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Snow {
    /// Precipitation volume for the last 1 hour in mm/h.
    #[serde(rename = "1h")]
    pub(crate) one_hour: f64,
}

/// Cloudiness information.
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Clouds {
    /// Cloudiness percentage.
    pub(crate) all: u32,
}

/// System information related to the location.
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Sys {
    /// Internal parameter (type of the system).
    #[serde(rename = "type")]
    pub(crate) sys_type: Option<u32>,
    /// Internal parameter (system ID).
    pub(crate) id: Option<u32>,
    /// Country code (e.g., GB, JP, CN).
    pub(crate) country: String,
    /// Sunrise time in Unix UTC timestamp.
    pub(crate) sunrise: u64,
    /// Sunset time in Unix UTC timestamp.
    pub(crate) sunset: u64,
}
