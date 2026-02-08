/////////////////////////////////////////////////////////////////////////////////////////

pub trait Format {
    type ErrorDe: std::error::Error + Send + Sync + 'static;
    type ErrorSer: std::error::Error + Send + Sync + 'static;

    fn name() -> std::borrow::Cow<'static, str>;

    fn deserialize<T: serde::de::DeserializeOwned>(string: &str) -> Result<T, Self::ErrorDe>;

    fn serialize<T: serde::ser::Serialize>(value: &T) -> Result<String, Self::ErrorSer>;
}

/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "fmt-json")]
pub struct Json;

#[cfg(feature = "fmt-json")]
impl Format for Json {
    type ErrorDe = serde_json::Error;
    type ErrorSer = serde_json::Error;

    fn name() -> std::borrow::Cow<'static, str> {
        "json".into()
    }

    fn deserialize<T: serde::de::DeserializeOwned>(string: &str) -> Result<T, Self::ErrorDe> {
        serde_json::from_str(string)
    }

    fn serialize<T: serde::ser::Serialize>(value: &T) -> Result<String, Self::ErrorSer> {
        serde_json::to_string(value)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "fmt-json")]
pub struct JsonPretty;

#[cfg(feature = "fmt-json")]
impl Format for JsonPretty {
    type ErrorDe = serde_json::Error;
    type ErrorSer = serde_json::Error;

    fn name() -> std::borrow::Cow<'static, str> {
        "json".into()
    }

    fn deserialize<T: serde::de::DeserializeOwned>(string: &str) -> Result<T, Self::ErrorDe> {
        serde_json::from_str(string)
    }

    fn serialize<T: serde::ser::Serialize>(value: &T) -> Result<String, Self::ErrorSer> {
        serde_json::to_string_pretty(value)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "fmt-yaml")]
pub struct Yaml;

#[cfg(feature = "fmt-yaml")]
impl Format for Yaml {
    type ErrorDe = serde_yaml::Error;
    type ErrorSer = serde_yaml::Error;

    fn name() -> std::borrow::Cow<'static, str> {
        "yaml".into()
    }

    fn deserialize<T: serde::de::DeserializeOwned>(string: &str) -> Result<T, Self::ErrorDe> {
        serde_yaml::from_str(string)
    }

    #[cfg(not(feature = "fmt-yaml-arbitrary-precision-hack"))]
    fn serialize<T: serde::ser::Serialize>(value: &T) -> Result<String, Self::ErrorSer> {
        serde_yaml::to_string(value)
    }

    #[cfg(feature = "fmt-yaml-arbitrary-precision-hack")]
    fn serialize<T: serde::ser::Serialize>(value: &T) -> Result<String, Self::ErrorSer> {
        // To avoid outputting `$serde_json::private::Number` tag we resort to ugly transcoding
        // See: https://github.com/acatton/serde-yaml-ng/issues/31
        let s = serde_json::to_string(value).unwrap();
        let value: serde_yaml::Value = serde_yaml::from_str(&s).unwrap();
        serde_yaml::to_string(&value)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "fmt-toml")]
pub struct Toml;

#[cfg(feature = "fmt-toml")]
impl Format for Toml {
    type ErrorDe = toml::de::Error;
    type ErrorSer = toml::ser::Error;

    fn name() -> std::borrow::Cow<'static, str> {
        "toml".into()
    }

    fn deserialize<T: serde::de::DeserializeOwned>(string: &str) -> Result<T, Self::ErrorDe> {
        toml::from_str(string)
    }

    fn serialize<T: serde::ser::Serialize>(value: &T) -> Result<String, Self::ErrorSer> {
        toml::to_string(value)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
