use std::convert::Infallible;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResolveError {
    #[error("parameter not specified")]
    Missing,
    #[error("paremeter is unresolvable")]
    Unresolvable,
    #[error("parameter has invalid type")]
    InvalidType,
    #[error("integer out of range")]
    IntegerRangeError,
    #[error("number out of range")]
    NumberRangeError,
    #[error("invalid string length")]
    StringLengthError,
    #[error("no partial member data available")]
    NoPartialMemberData,
    #[error("invalid choice")]
    InvalidChoice,
    #[error("invalid channel type")]
    InvalidChannelType,
}

impl From<Infallible> for ResolveError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

pub type ResolveResult<T> = anyhow::Result<T>;
