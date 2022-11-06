//! `bincode-json` is a wrapper around `bincode` to encode/decode JSON-like objects.
//!
//! ## Features
//!  - `preserve-order`: use `indexmap` instead of HashMap to preserve object fields' order.
//!  - `json`: enables converting from/to `serde_json::Value`.

pub mod de;
pub mod error;
pub mod ser;
pub mod value;

pub use error::{Error, Result};
pub use value::Value;

use serde::{de::DeserializeOwned, Serialize};

/// Interpret a [Value] as an instance of type `T`.
pub fn from_value<T: DeserializeOwned>(val: Value) -> Result<T> {
    T::deserialize(de::Deserializer::from(val))
}

/// Convert a `T` into [Value].
pub fn to_value<T: Serialize>(val: &T) -> Result<Value> {
    val.serialize(ser::Serializer::new())
}

/// Serialize the given data structure as a byte vector.
pub fn to_vec<T: Serialize>(val: &T) -> Result<Vec<u8>> {
    let value = to_value(val)?;
    Ok(bincode::encode_to_vec(value, bincode::config::standard())?)
}

/// Deserialize an instance of type `T` from bytes of Bincode JSON.
pub fn from_slice<T: DeserializeOwned>(val: &[u8]) -> Result<T> {
    let (value, _) = bincode::decode_from_slice(val, bincode::config::standard())?;
    from_value(value)
}
