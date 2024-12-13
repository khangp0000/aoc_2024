use nom_supreme::error::GenericErrorTree;
use std::borrow::Cow;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    #[error(transparent)]
    UtilsError(#[from] crate::utils::UtilsError),
    #[error("parsing error: `{0}`")]
    ParseError(Cow<'static, str>),
    #[error("invalid state: `{0}`")]
    InvalidState(Cow<'static, str>),
    #[error("initialization error: `{0}`")]
    InitError(Cow<'static, str>),
    #[error("overflow error: `{0}`")]
    Unsolvable(Cow<'static, str>),
    #[error("{0}")]
    NomParseError(String),
}

pub type NomError<'a> = GenericErrorTree<&'a str, &'static str, &'static str, Error>;

impl<'a> From<nom::Err<NomError<'a>>> for Error {
    fn from(value: nom::Err<NomError<'a>>) -> Self {
        Self::NomParseError(value.to_string())
    }
}
