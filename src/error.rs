use thiserror::Error;

#[derive(Error, Debug)]
pub enum LinkError {
    #[error("Value not provided for {value:?}.")]
    UserBuildError { value: Vec<String> },
    #[error("HTTP request error.")]
    HttpError(#[from] reqwest::Error),
    #[error("Input/output error from std.")]
    Io(#[from] std::io::Error),
    #[error("Could not read environmental variables from .env.")]
    EnvError(#[from] std::env::VarError),
    #[error("Authorization failed.")]
    AuthError,
    #[error("Bad file name {0:?}.")]
    FileNameError(std::ffi::OsString),
    #[error("Conversion to JSON failed.")]
    JsonError(#[from] serde_json::Error),
    #[error("Link missing for document.")]
    MissingLink,
    #[error("Byte conversion failed.")]
    ByteError(#[from] byte_unit::ByteError),
}
