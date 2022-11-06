//! When serializing or deserializing `bincode-json` goes wrong.

use serde::de::{Expected, Unexpected};
use std::fmt::Display;

/// This type represents all possible errors that can occur when serializing or
/// deserializing `bincode-json` data.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bincode error: {0}")]
    Bincode(BincodeError),

    #[error("custom error: {0}")]
    Custom(String),

    #[error("expected {0}, found {1}")]
    Expected(String, String),

    #[error("field {0} was duplicated")]
    Duplicated(String),

    #[error("field {0} was missing")]
    Missing(String),

    #[error("field or variant {0} was unknown")]
    Unknown(String),

    #[error("unexpected eof")]
    Eof,
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
#[derive(Debug, thiserror::Error)]
pub enum BincodeError {
    #[error("encode: {0}")]
    Encode(bincode::error::EncodeError),

    #[error("decode: {0}")]
    Decode(bincode::error::DecodeError),
}
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
