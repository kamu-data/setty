pub mod combine;
mod config;
pub mod format;
mod markdown;
mod merge_with_defaults;
pub mod types;

/////////////////////////////////////////////////////////////////////////////////////////

pub use config::*;

/////////////////////////////////////////////////////////////////////////////////////////

pub use setty_derive::{__erase, Config, Default, derive};

pub use figment2::value::Value;

#[cfg(feature = "test")]
pub mod test {
    pub use figment2::Jail;
}

/////////////////////////////////////////////////////////////////////////////////////////

pub mod __internal {
    pub use figment2;

    #[cfg(feature = "derive-jsonschema")]
    pub use schemars;

    #[cfg(any(feature = "derive-deserialize", feature = "derive-serialize"))]
    pub use serde;

    pub use serde_json;

    #[cfg(feature = "derive-serialize")]
    pub use serde_with;
}
