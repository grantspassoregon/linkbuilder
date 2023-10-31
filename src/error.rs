//! Error handling module for the library.
//!
//! Creates a custom Error type `LinkError`, and an alias
//! for Result using the `LinkError`, called `LinkResult`.

/// The LinkError enum takes failure types from different libraries and converts them to a variant
/// of LinkError using the framework from the thiserror library.
#[derive(thiserror::Error, Debug)]
pub enum LinkError {
    /// Error returned from local builder pattern [see [`crate::authorize::UserBuilder::build`]].
    #[error("Value not provided for {value:?}.")]
    UserBuildError {
        /// Vector of fields passed into UserBuilder that triggered the error.
        value: Vec<String>,
    },
    /// Produced by calling build on a builder with incomplete fields.
    #[error("Required values not provided.")]
    BuildError,
    /// Error returned by the reqwest library.
    #[error("HTTP request error.")]
    HttpError(#[from] reqwest::Error),
    /// Error returned by the std::io module.
    #[error("Input/output error from std.")]
    Io(#[from] std::io::Error),
    /// Error returned by the std::env module.
    #[error("Could not read environmental variables from .env.")]
    EnvError(#[from] std::env::VarError),
    /// Local error returned by the authorize module.  See [`crate::authorize::AuthorizeInfo::authorize`].
    #[error("Authorization failed.")]
    AuthError,
    // #[error("Bad file name {0:?}.")]
    // FileNameError(std::ffi::OsString),
    /// Error returned by the serde_json library.  See [`crate::document::Document::update`].
    #[error("Conversion to JSON failed.")]
    JsonError(#[from] serde_json::Error),
    /// Error returned by the byte_unit library. See [`crate::report::ReportItem::new()']
    #[error("Byte conversion failed.")]
    ByteError(#[from] byte_unit::ByteError),
}

/// Alias for the Result type using the local Error type.
pub type LinkResult<T> = Result<T, LinkError>;
