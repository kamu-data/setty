/////////////////////////////////////////////////////////////////////////////////////////

pub mod duration_string;
pub mod rfc3339;

pub use duration_string::*;
pub use rfc3339::*;

/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "type-bigdecimal04")]
impl crate::combine::Combine for bigdecimal::BigDecimal {}

#[cfg(feature = "type-url2")]
impl crate::combine::Combine for url::Url {}
