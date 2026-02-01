#![cfg(feature = "type-duration-string05")]

use duration_string as ds;

/////////////////////////////////////////////////////////////////////////////////////////

/// Wrapper type for [`ds::DurationString`] is needed to be able to derive missing traits like [`schemars::JsonSchema`]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DurationString(std::time::Duration);

/////////////////////////////////////////////////////////////////////////////////////////

impl DurationString {
    pub fn new(d: std::time::Duration) -> Self {
        Self(d)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

impl std::convert::TryFrom<&str> for DurationString {
    type Error = ds::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let d: ds::DurationString = s.parse()?;
        Ok(Self::new(d.into()))
    }
}

impl std::str::FromStr for DurationString {
    type Err = duration_string::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

impl std::ops::Deref for DurationString {
    type Target = std::time::Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<DurationString> for std::time::Duration {
    fn from(value: DurationString) -> Self {
        value.0
    }
}

impl From<std::time::Duration> for DurationString {
    fn from(value: std::time::Duration) -> Self {
        Self::new(value)
    }
}

impl From<DurationString> for String {
    fn from(value: DurationString) -> Self {
        let ds = ds::DurationString::new(value.0);
        ds.to_string()
    }
}

impl From<ds::DurationString> for DurationString {
    fn from(value: ds::DurationString) -> Self {
        Self::new(value.into())
    }
}

impl From<DurationString> for ds::DurationString {
    fn from(value: DurationString) -> Self {
        Self::new(value.into())
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

impl std::fmt::Display for DurationString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ds::DurationString::new(self.0).fmt(f)
    }
}

impl std::fmt::Debug for DurationString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
// Serde
/////////////////////////////////////////////////////////////////////////////////////////

impl serde::Serialize for DurationString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        ds::DurationString::new(self.0).serialize(serializer)
    }
}

impl<'de> serde::de::Deserialize<'de> for DurationString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let ds = ds::DurationString::deserialize(deserializer)?;
        Ok(ds.into())
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
// JsonSchema
/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "derive-jsonschema")]
impl schemars::JsonSchema for DurationString {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "DurationString".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        String::json_schema(generator)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
