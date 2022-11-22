//! When serializing or deserializing `bincode-json` goes wrong.

use serde::de::{Expected, Unexpected};
use std::fmt::{self, Display};

/// This type represents all possible errors that can occur when serializing or
/// deserializing `bincode-json` data.
#[derive(Debug)]
pub enum Error {
    Bincode(BincodeError),
    Custom(String),
    Expected(String, String),
    Duplicated(String),
    Missing(String),
    Unknown(String),
    Eof,
}
impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bincode(e) => write!(formatter, "bincode error: {}", e),
            Self::Custom(s) => write!(formatter, "custom error: {}", s),
            Self::Expected(e, f) => write!(formatter, "expected {}, found {}", e, f),
            Self::Duplicated(x) => write!(formatter, "field {} was duplicated", x),
            Self::Missing(x) => write!(formatter, "field {} was missing", x),
            Self::Unknown(x) => write!(formatter, "field or variant {} was unknown", x),
            Self::Eof => write!(formatter, "unexpected eof"),
        }
    }
}
impl From<bincode::error::EncodeError> for Error {
    fn from(value: bincode::error::EncodeError) -> Self {
        Self::Bincode(value.into())
    }
}
impl From<bincode::error::DecodeError> for Error {
    fn from(value: bincode::error::DecodeError) -> Self {
        Self::Bincode(value.into())
    }
}
impl std::error::Error for Error {}
impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Custom(msg.to_string())
    }
}
impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Custom(msg.to_string())
    }
    fn invalid_type(unexp: Unexpected, exp: &dyn Expected) -> Self {
        Self::Expected(exp.to_string(), unexp.to_string())
    }
    fn invalid_value(unexp: Unexpected, exp: &dyn Expected) -> Self {
        Self::Expected(exp.to_string(), unexp.to_string())
    }
    fn invalid_length(len: usize, exp: &dyn Expected) -> Self {
        Self::Expected(format!("length {}", len), exp.to_string())
    }
    fn unknown_variant(variant: &str, _: &'static [&'static str]) -> Self {
        Self::Unknown(variant.into())
    }
    fn unknown_field(field: &str, _: &'static [&'static str]) -> Self {
        Self::Unknown(field.into())
    }
    fn missing_field(field: &'static str) -> Self {
        Self::Missing(field.into())
    }
    fn duplicate_field(field: &'static str) -> Self {
        Self::Duplicated(field.into())
    }
}

/// A common type of [bincode::error::EncodeError] and [bincode::error::DecodeError].
#[derive(Debug)]
pub enum BincodeError {
    Encode(bincode::error::EncodeError),
    Decode(bincode::error::DecodeError),
}
impl Display for BincodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encode(e) => write!(formatter, "encode: {}", e),
            Self::Decode(d) => write!(formatter, "decode: {}", d),
        }
    }
}
impl std::error::Error for BincodeError {}
impl From<bincode::error::EncodeError> for BincodeError {
    fn from(value: bincode::error::EncodeError) -> Self {
        Self::Encode(value)
    }
}
impl From<bincode::error::DecodeError> for BincodeError {
    fn from(value: bincode::error::DecodeError) -> Self {
        Self::Decode(value)
    }
}

/// Alias for a Result with the error type [Error].
pub type Result<T> = std::result::Result<T, Error>;
