//! The Value enum, a loosely typed way of representing any valid `bincode-json` value.

use serde::{de, ser};

/// Represents a `bincode-json` key/value type.
pub type Map<K, V> = std::collections::HashMap<K, V>;

macro_rules! value_from_int {
    ($x:tt) => {
        impl From<$x> for Value {
            fn from(i: $x) -> Self {
                Self::Integer(i as _)
            }
        }
    };
}
macro_rules! value_is {
    ($x:tt, $v:ident) => {
        pub fn $x(&self) -> bool {
            matches!(self, Self::$v(_))
        }
    }
}
macro_rules! value_as {
    ($x:tt, $v:ident, $t:ty) => {
        pub fn $x(&self) -> Option<&$t> {
            match self {
                Value::$v(v) => Some(v),
                _ => None,
            }
        }
    }
}

/// Represents any valid `bincode-json` value.
#[derive(Debug, Clone, bincode::Encode, bincode::Decode)]
pub enum Value {
    /// Represents a `bincode-json` null value.
    Null,

    /// Represents a `bincode-json` bool value.
    Boolean(bool),

    /// Represents a `bincode-json` blob value.
    Blob(Vec<u8>),

    /// Represents a `bincode-json` array value.
    Array(Vec<Value>),

    /// Represents a `bincode-json` integer value.
    Integer(i64),

    /// Represents a `bincode-json` float value.
    Float(f64),

    /// Represents a `bincode-json` object value.
    Object(Map<String, Value>),

    /// Represents a `bincode-json` string value.
    String(String),
}
impl<'a> From<&'a str> for Value {
    fn from(s: &'a str) -> Self {
        Self::String(s.into())
    }
}
value_from_int!(i8);
value_from_int!(u8);
value_from_int!(i16);
value_from_int!(u16);
value_from_int!(i32);
value_from_int!(u32);
value_from_int!(i64);
value_from_int!(u64);

impl Value {
    /// Gets the `bincode-json` type of the value.
    pub(crate) fn error_description(&self) -> &'static str {
        match self {
            Self::Null => "type null",
            Self::Blob(_) => "type blob",
            Self::Boolean(_) => "type boolean",
            Self::Integer(_) => "type integer",
            Self::Float(_) => "type float",
            Self::Object(_) => "type object",
            Self::String(_) => "type string",
            Self::Array(_) => "type array",
        }
    }

    #[cfg(feature = "json")]
    /// Converts a [Value] to a [serde_json::Value].
    pub fn to_json(self) -> serde_json::Value {
        match self {
            Self::Null => serde_json::Value::Null,
            Self::Blob(blob) => serde_json::Value::String(base64::encode(blob)),
            Self::Boolean(b) => serde_json::Value::Bool(b),
            Self::Integer(i) => serde_json::Value::Number(i.into()),
            Self::Float(f) => match serde_json::Number::from_f64(f) {
                Some(n) => serde_json::Value::Number(n),
                None => serde_json::Value::String(f.to_string()),
            },
            Self::Object(o) => {
                let mut map = serde_json::Map::with_capacity(o.len());
                for (k, v) in o {
                    map.insert(k, v.to_json());
                }
                serde_json::Value::Object(map)
            }
            Self::String(s) => serde_json::Value::String(s),
            Self::Array(a) => {
                let mut arr = Vec::with_capacity(a.len());
                for v in a {
                    arr.push(v.to_json());
                }
                serde_json::Value::Array(arr)
            }
        }
    }

    /// Returns `true` if this value is `Null`.
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    value_is!(is_str, String);
    value_as!(as_str, String, str);

    value_is!(is_integer, Integer);
    value_as!(as_integer, Integer, i64);

    value_is!(is_float, Float);
    value_as!(as_float, Float, f64);

    value_is!(is_blob, Blob);
    value_as!(as_blob, Blob, [u8]);

    value_is!(is_bool, Boolean);
    value_as!(as_bool, Boolean, bool);
}
impl<'de> de::Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(Visitor)
    }
}
impl ser::Serialize for Value {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Null => serializer.serialize_none(),
            Value::Boolean(b) => serializer.serialize_bool(*b),
            Value::Blob(b) => serializer.serialize_bytes(b),
            Value::Integer(n) => serializer.serialize_i64(*n),
            Value::Float(f) => serializer.serialize_f64(*f),
            Value::String(s) => serializer.serialize_str(s),
            Value::Array(v) => v.serialize(serializer),
            Value::Object(m) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Value> for Value {
    fn from(v: serde_json::Value) -> Self {
        match v {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(b) => Self::Boolean(b),
            serde_json::Value::Array(array) => {
                let mut arr: Vec<Value> = Vec::with_capacity(array.len());
                for i in array {
                    arr.push(i.into());
                }
                Self::Array(arr)
            }
            serde_json::Value::Number(n) => {
                if n.is_i64() {
                    Self::Integer(n.as_i64().unwrap())
                } else if n.is_u64() {
                    Self::Integer(n.as_u64().unwrap() as _)
                } else if n.is_f64() {
                    Self::Float(n.as_f64().unwrap())
                } else {
                    unreachable!()
                }
            }
            serde_json::Value::Object(o) => {
                let mut map = Map::with_capacity(o.len());
                for (k, v) in o {
                    map.insert(k, v.into());
                }
                Self::Object(map)
            }
            serde_json::Value::String(s) => Self::String(s),
        }
    }
}

struct Visitor;
impl<'de> de::Visitor<'de> for Visitor {
    type Value = Value;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("a Bincode JSON value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Boolean(value))
    }

    fn visit_i8<E>(self, value: i8) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Integer(value as _))
    }

    fn visit_i16<E>(self, value: i16) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Integer(value as _))
    }

    fn visit_i32<E>(self, value: i32) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Integer(value as _))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Integer(value))
    }

    fn visit_u8<E>(self, value: u8) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Integer(value as _))
    }

    fn visit_u16<E>(self, value: u16) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Integer(value as _))
    }

    fn visit_u32<E>(self, value: u32) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Integer(value as _))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Integer(value as _))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Value, E> {
        Ok(Value::Float(value))
    }

    fn visit_str<E>(self, value: &str) -> Result<Value, E>
    where
        E: de::Error,
    {
        self.visit_string(String::from(value))
    }

    fn visit_string<E>(self, value: String) -> Result<Value, E> {
        Ok(Value::String(value))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Blob(v.into()))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Blob(v))
    }

    fn visit_none<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Array(vec![]))
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<Value, V::Error>
    where
        V: de::SeqAccess<'de>,
    {
        let mut values = Vec::new();

        while let Some(elem) = visitor.next_element()? {
            values.push(elem);
        }

        Ok(Value::Array(values))
    }

    fn visit_map<V>(self, mut visitor: V) -> Result<Value, V::Error>
    where
        V: de::MapAccess<'de>,
    {
        let mut map: Map<String, Value> = Map::new();

        while let Some((k, v)) = visitor.next_entry()? {
            map.insert(k, v);
        }

        Ok(Value::Object(map))
    }
}
