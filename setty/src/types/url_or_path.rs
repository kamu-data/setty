#![cfg(feature = "types-url")]

use std::path::PathBuf;

/////////////////////////////////////////////////////////////////////////////////////////

/// Wrapper type for [`url::Url`] that upon failed URL parsing
/// falls back to interpreting value as a local fs path
/// and converting it into a canonical URL with file:/// scheme.
#[derive(Clone, PartialEq, Eq)]
pub struct UrlOrPath(url::Url);

/////////////////////////////////////////////////////////////////////////////////////////

impl UrlOrPath {
    pub fn new(url: url::Url) -> Self {
        Self(url)
    }

    pub fn as_url(&self) -> &url::Url {
        &self.0
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

impl std::fmt::Display for UrlOrPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Debug for UrlOrPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

impl std::convert::TryFrom<&str> for UrlOrPath {
    type Error = url::ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match url::Url::parse(s) {
            Ok(url) => Ok(Self(url)),
            Err(err) => match PathBuf::from(s).canonicalize() {
                Ok(path) => Ok(Self(url::Url::from_directory_path(path).unwrap())),
                Err(_) => Err(err),
            },
        }
    }
}

impl std::str::FromStr for UrlOrPath {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

impl std::ops::Deref for UrlOrPath {
    type Target = url::Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<UrlOrPath> for url::Url {
    fn from(value: UrlOrPath) -> Self {
        value.0
    }
}

impl From<url::Url> for UrlOrPath {
    fn from(value: url::Url) -> Self {
        Self::new(value)
    }
}

impl From<UrlOrPath> for String {
    fn from(value: UrlOrPath) -> Self {
        value.to_string()
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
// Serde
/////////////////////////////////////////////////////////////////////////////////////////

impl serde::Serialize for UrlOrPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

impl<'de> serde::de::Deserialize<'de> for UrlOrPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse()
            .map_err(|e: url::ParseError| serde::de::Error::custom(e.to_string()))
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
// JsonSchema
/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "derive-jsonschema")]
impl schemars::JsonSchema for UrlOrPath {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "UrlOrPath".into()
    }

    fn json_schema(_gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "format": "url-or-path",
        })
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

impl crate::combine::Combine for UrlOrPath {}

/////////////////////////////////////////////////////////////////////////////////////////
