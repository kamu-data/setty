#![cfg(feature = "types-bytesize")]

use bytesize as bs;

/////////////////////////////////////////////////////////////////////////////////////////

/// Wrapper type for [`bs::ByteSize`] is needed to be able to derive missing traits like [`schemars::JsonSchema`]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ByteSize(bs::ByteSize);

/////////////////////////////////////////////////////////////////////////////////////////

impl ByteSize {
    pub fn new(v: bs::ByteSize) -> Self {
        Self(v)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

impl std::convert::TryFrom<&str> for ByteSize {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let v: bs::ByteSize = s.parse()?;
        Ok(Self::new(v))
    }
}

impl std::str::FromStr for ByteSize {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

impl std::ops::Deref for ByteSize {
    type Target = bs::ByteSize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<ByteSize> for u64 {
    fn from(value: ByteSize) -> Self {
        value.0.as_u64()
    }
}

impl From<u64> for ByteSize {
    fn from(value: u64) -> Self {
        Self::new(bs::ByteSize::b(value))
    }
}

impl From<ByteSize> for String {
    fn from(value: ByteSize) -> Self {
        value.to_string()
    }
}

impl From<bs::ByteSize> for ByteSize {
    fn from(value: bs::ByteSize) -> Self {
        Self::new(value)
    }
}

impl From<ByteSize> for bs::ByteSize {
    fn from(value: ByteSize) -> Self {
        value.0
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

impl std::fmt::Display for ByteSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Debug for ByteSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
// Serde
/////////////////////////////////////////////////////////////////////////////////////////

impl serde::Serialize for ByteSize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> serde::de::Deserialize<'de> for ByteSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let v = bs::ByteSize::deserialize(deserializer)?;
        Ok(Self::new(v))
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
// JsonSchema
/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "derive-jsonschema")]
impl schemars::JsonSchema for ByteSize {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "ByteSize".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        String::json_schema(generator)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

impl crate::combine::Combine for ByteSize {}

/////////////////////////////////////////////////////////////////////////////////////////
