use nom_supreme::error::GenericErrorTree;
use std::borrow::Cow;
use std::fmt::Display;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    #[error("utils error: {0}")]
    UtilsError(#[from] crate::utils::UtilsError),
    #[error("parsing error: `{0}`")]
    ParseError(Cow<'static, str>),
    #[error("invalid state: `{0}`")]
    InvalidState(Cow<'static, str>),
    #[error("initialization error: `{0}`")]
    InitError(Cow<'static, str>),
    #[error("unsolvable error: `{0}`")]
    Unsolvable(Cow<'static, str>),
    #[error("nom error: {0}")]
    NomParseError(String),
}

pub type NomError<'a, T = &'a str> = GenericErrorTree<T, &'static str, &'static str, Error>;

impl<T: Display> From<NomError<'_, T>> for Error {
    fn from(value: NomError<T>) -> Self {
        Self::NomParseError(value.to_string())
    }
}
