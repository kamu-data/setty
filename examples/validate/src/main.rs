//! Example demonstrating the #[config(validate(...))] attribute
//!
//! The validation constraints are delegated to the `validator` crate.
//! When the `derive-validate` feature is enabled, the `Config` derive
//! macro will generate `Validate` derive for your struct and pass through
//! validation attributes.

#[derive(setty::Config)]
struct AppConfig {
    /// Username must be between 3 and 32 characters
    #[config(validate(length(min = 3, max = 32)))]
    username: String,

    /// Port must be between 1 and 65535
    #[config(validate(range(min = 1, max = 65535)))]
    port: u16,

    /// Email must be a valid email format
    #[config(validate(email))]
    email: String,

    /// Optional API key that if provided must be at least 10 characters
    #[config(validate(length(min = 10)))]
    api_key: Option<String>,
}

fn main() -> color_eyre::Result<()> {
    // Valid config
    setty::Config::<AppConfig>::new()
        .with_source(serde_json::json!({
            "username": "johndoe",
            "port": 8080,
            "email": "john@example.com",
            "api_key": "secret1234567890",
        }))
        .extract()?;

    println!("✓ Valid config passed validation");

    // Invalid config
    let e = setty::Config::<AppConfig>::new()
        .with_source(serde_json::json!({
            "username": "jo", // too short
            "port": 8080,
            "email": "john@example.com",
            "api_key": "secret1234567890",
        }))
        .extract()
        .expect_err("Validation shoud fail");

    println!("✗ Validation correctly failed:\n  {}", e);

    // Invalid config
    let e = setty::Config::<AppConfig>::new()
        .with_source(serde_json::json!({
            "username": "johndoe",
            "port": 8080,
            "email": "not-an-email", // invalid email
            "api_key": "secret1234567890",
        }))
        .extract()
        .expect_err("Validation shoud fail");

    println!("✗ Validation correctly failed:\n  {}", e);

    Ok(())
}
