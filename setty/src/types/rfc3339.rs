#![cfg(feature = "types-chrono")]

/////////////////////////////////////////////////////////////////////////////////////////

/// Wrapper type for [`chrono::DateTime`] that serializes to/from RFC3339 format like `1996-12-19T16:39:57-08:00` or `1996-12-19T16:39:57Z`
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DateTime(chrono::DateTime<chrono::FixedOffset>);

/////////////////////////////////////////////////////////////////////////////////////////

impl DateTime {
    pub fn new(dt: chrono::DateTime<chrono::FixedOffset>) -> Self {
        Self(dt)
    }

    pub fn as_chrono(&self) -> &chrono::DateTime<chrono::FixedOffset> {
        &self.0
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

impl std::fmt::Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true)
        )
    }
}

impl std::fmt::Debug for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

impl std::convert::TryFrom<&str> for DateTime {
    type Error = chrono::ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let dt = chrono::DateTime::parse_from_rfc3339(s)?;
        Ok(Self::new(dt))
    }
}

impl std::str::FromStr for DateTime {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

impl std::ops::Deref for DateTime {
    type Target = chrono::DateTime<chrono::FixedOffset>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<DateTime> for chrono::DateTime<chrono::FixedOffset> {
    fn from(value: DateTime) -> Self {
        value.0
    }
}

impl From<chrono::DateTime<chrono::FixedOffset>> for DateTime {
    fn from(value: chrono::DateTime<chrono::FixedOffset>) -> Self {
        Self::new(value)
    }
}

impl From<DateTime> for String {
    fn from(value: DateTime) -> Self {
        value.to_string()
    }
}

impl From<chrono::DateTime<chrono::Utc>> for DateTime {
    fn from(value: chrono::DateTime<chrono::Utc>) -> Self {
        Self::new(value.into())
    }
}

impl From<DateTime> for chrono::DateTime<chrono::Utc> {
    fn from(value: DateTime) -> Self {
        value.0.into()
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
// Serde
/////////////////////////////////////////////////////////////////////////////////////////

impl serde::Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

impl<'de> serde::de::Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse()
            .map_err(|e: chrono::ParseError| serde::de::Error::custom(e.to_string()))
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
// JsonSchema
/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "derive-jsonschema")]
impl schemars::JsonSchema for DateTime {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "DateTime".into()
    }

    fn json_schema(_gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "format": "date-time-rfc3339",
        })
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

impl crate::combine::Combine for DateTime {}

/////////////////////////////////////////////////////////////////////////////////////////
