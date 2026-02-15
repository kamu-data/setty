/////////////////////////////////////////////////////////////////////////////////////////

/// Error returned when reading a [`crate::source::Source`].
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum ReadError {
    // TODO: Expand this to provide more error kinds
    /// Boxed deserialization error
    Serde(Box<dyn std::error::Error + Send + Sync>),

    /// Failed validation
    #[cfg(feature = "derive-validate")]
    Validation(#[from] validator::ValidationErrors),

    /// IO error when reading from disk
    Io(#[from] std::io::Error),
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Error returned when saving configuration
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum WriteError {
    Read(#[from] ReadError),
    Io(#[from] std::io::Error),
}

/////////////////////////////////////////////////////////////////////////////////////////
