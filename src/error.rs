use thiserror::Error;

#[derive(Error, Debug)]
pub enum LinkError {
    #[error("Value not provided for {value:?}.")]
    UserBuildError { value: Vec<String> },
    #[error("HTTP request error.")]
    HttpError(#[from] reqwest::Error),
}
