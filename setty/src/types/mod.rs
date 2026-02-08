/////////////////////////////////////////////////////////////////////////////////////////

pub mod duration_string;
pub mod rfc3339;

#[cfg(feature = "types-duration-string")]
pub use duration_string::*;

#[cfg(feature = "types-chrono")]
pub use rfc3339::*;

/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "types-bigdecimal")]
impl crate::combine::Combine for bigdecimal::BigDecimal {}

#[cfg(feature = "types-url")]
impl crate::combine::Combine for url::Url {}
