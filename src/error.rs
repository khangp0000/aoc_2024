use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum Error {
    #[error(transparent)]
    UtilsError(#[from] crate::utils::UtilsError),
    #[error("parsing error: `{0}`")]
    ParseError(String),
    #[error("invalid state: `{0}`")]
    InvalidState(String),
    #[error("initialization error: `{0}`")]
    InitError(String),
}
