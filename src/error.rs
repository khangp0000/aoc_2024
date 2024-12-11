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
}
