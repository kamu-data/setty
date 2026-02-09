#![cfg(feature = "derive-jsonschema")]

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq)]
#[repr(transparent)]
pub struct Schema(schemars::Schema);

impl Schema {
    pub fn new(value: schemars::Schema) -> Self {
        Self(value)
    }

    pub fn to_string_pretty(&self) -> String {
        serde_json::to_string_pretty(self.0.as_value()).unwrap()
    }

    pub fn to_value(self) -> serde_json::Value {
        self.0.to_value()
    }
}

impl From<schemars::Schema> for Schema {
    fn from(value: schemars::Schema) -> Self {
        Self::new(value)
    }
}

impl From<Schema> for schemars::Schema {
    fn from(value: Schema) -> Self {
        value.0
    }
}

impl std::ops::Deref for Schema {
    type Target = schemars::Schema;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
