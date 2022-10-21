use std::{convert::Infallible, fmt, num::TryFromIntError};

use crate::AnyError;

#[derive(Debug)]
pub enum ResolveError {
    Missing,
    Unresolvable,
    InvalidType,
    IntegerRangeError,
    NumberRangeError,
    StringLengthError,
    NoPartialMemberData,
    InvalidChoice,
    InvalidChannelType,
    TryFromIntError(TryFromIntError),
    Serenity(serenity::Error),
    Other(AnyError),
}

impl From<Infallible> for ResolveError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl From<TryFromIntError> for ResolveError {
    fn from(error: TryFromIntError) -> Self {
        Self::TryFromIntError(error)
    }
}

impl From<serenity::Error> for ResolveError {
    fn from(error: serenity::Error) -> Self {
        Self::Serenity(error)
    }
}

pub type ResolveResult<T> = Result<T, ResolveError>;

impl std::error::Error for ResolveError {}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolveError::Missing => write!(f, "parameter not specified"),
            ResolveError::Unresolvable => write!(f, "paremeter is unresolvable"),
            ResolveError::InvalidType => write!(f, "parameter has invalid type"),
            ResolveError::IntegerRangeError => write!(f, "integer out of range"),
            ResolveError::NumberRangeError => write!(f, "number out of range"),
            ResolveError::StringLengthError => write!(f, "invalid string length"),
            ResolveError::NoPartialMemberData => write!(f, "no partial member data available"),
            ResolveError::InvalidChoice => write!(f, "invalid choice"),
            ResolveError::InvalidChannelType => write!(f, "invalid channel type"),
            ResolveError::TryFromIntError(error) => error.fmt(f),
            ResolveError::Serenity(error) => error.fmt(f),
            ResolveError::Other(error) => error.fmt(f),
        }
    }
}
