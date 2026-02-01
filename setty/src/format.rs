/////////////////////////////////////////////////////////////////////////////////////////

pub trait Format {
    type Error: std::error::Error;

    fn deserialize<T: serde::de::DeserializeOwned>(string: &str) -> Result<T, Self::Error>;

    fn serialize<T: serde::ser::Serialize>(value: &T) -> Result<String, Self::Error>;
}

/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "fmt-json")]
pub use figment2::providers::Json;

#[cfg(feature = "fmt-json")]
impl Format for figment2::providers::Json {
    type Error = serde_json::Error;

    fn deserialize<T: serde::de::DeserializeOwned>(string: &str) -> Result<T, Self::Error> {
        serde_json::from_str(string)
    }

    fn serialize<T: serde::ser::Serialize>(value: &T) -> Result<String, Self::Error> {
        serde_json::to_string(value)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "fmt-yaml")]
pub use figment2::providers::Yaml;

#[cfg(feature = "fmt-yaml")]
impl Format for figment2::providers::Yaml {
    type Error = serde_yaml::Error;

    fn deserialize<T: serde::de::DeserializeOwned>(string: &str) -> Result<T, Self::Error> {
        serde_yaml::from_str(string)
    }

    fn serialize<T: serde::ser::Serialize>(value: &T) -> Result<String, Self::Error> {
        serde_yaml::to_string(value)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
