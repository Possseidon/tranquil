use std::fmt;

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
    Other(AnyError),
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
            ResolveError::Other(error) => error.fmt(f),
        }
    }
}
