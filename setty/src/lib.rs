pub mod combine;
mod config;
pub mod format;
mod markdown;
mod merge_with_defaults;
pub mod source;
pub mod types;

/////////////////////////////////////////////////////////////////////////////////////////

pub use config::*;

pub use serde_json::Value;

/////////////////////////////////////////////////////////////////////////////////////////

pub use setty_derive::{__erase, Config, Default, derive};

/////////////////////////////////////////////////////////////////////////////////////////

pub mod __internal {

    #[cfg(feature = "derive-jsonschema")]
    pub use schemars;

    #[cfg(any(feature = "derive-deserialize", feature = "derive-serialize"))]
    pub use serde;

    pub use serde_json;

    #[cfg(feature = "derive-serialize")]
    pub use serde_with;
}
