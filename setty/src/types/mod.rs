/////////////////////////////////////////////////////////////////////////////////////////

pub mod duration_string;
pub mod rfc3339;
pub mod url_or_path;

#[cfg(feature = "types-duration-string")]
pub use duration_string::*;

#[cfg(feature = "types-chrono")]
pub use rfc3339::*;

#[cfg(feature = "types-url")]
pub use url_or_path::*;

/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "types-bigdecimal")]
impl crate::combine::Combine for bigdecimal::BigDecimal {}

#[cfg(feature = "types-secrecy")]
impl crate::combine::Combine for secrecy::SecretString {}

#[cfg(feature = "types-url")]
impl crate::combine::Combine for url::Url {}
