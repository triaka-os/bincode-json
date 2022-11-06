//! Serializes Rust data into `bincode-json` data.

use crate::{
    error::{Error, Result},
    value::{Map, Value},
};
use serde::{ser, Serialize};

/// A `bincode-json` serializer.
pub struct Serializer;
impl Default for Serializer {
    fn default() -> Self {
        Self
    }
}
impl Serializer {
    /// Constructs a new [Serializer] with default configuration.
    pub fn new() -> Self {
        Self::default()
    }
}
impl ser::Serializer for Serializer {
    type Ok = Value;
    type Error = Error;
    type SerializeSeq = SeqSerializer;
    type SerializeTuple = SeqSerializer;
    type SerializeTupleStruct = SeqSerializer;
    type SerializeTupleVariant = SeqVariantSerializer;
    type SerializeMap = MapSerializer;
    type SerializeStruct = StructSerializer;
    type SerializeStructVariant = StructVariantSerializer;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        Ok(Value::Boolean(v))
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.serialize_i64(v as _)
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.serialize_i64(v as _)
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.serialize_i64(v as _)
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        Ok(Value::Integer(v))
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.serialize_i64(v as _)
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.serialize_i64(v as _)
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.serialize_i64(v as _)
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.serialize_i64(v as _)
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.serialize_f64(v as _)
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        Ok(Value::Float(v))
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.serialize_str(&String::from(v)[..])
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        Ok(Value::String(v.to_owned()))
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        Ok(Value::Blob(v.to_owned()))
    }
    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(Value::Null)
    }
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(Value::Array(vec![]))
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }
    fn serialize_newtype_struct<T: ?Sized>(self, _: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        let mut map: Map<String, Value> = Map::with_capacity(1);
        map.insert(variant.to_string(), value.serialize(self)?);
        Ok(Value::Object(map))
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SeqSerializer {
            inner: Vec::with_capacity(len.unwrap_or(0)),
        })
    }
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_tuple(len)
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(SeqVariantSerializer {
            variant,
            inner: Vec::with_capacity(len),
        })
    }
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(MapSerializer {
            inner: Map::with_capacity(len.unwrap_or(0)),
            next_key: None,
        })
    }
    fn serialize_struct(self, _: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        Ok(Self::SerializeStruct {
            inner: Map::with_capacity(len),
        })
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(StructVariantSerializer {
            variant,
            inner: Map::with_capacity(len),
        })
    }
    fn is_human_readable(&self) -> bool {
        false
    }
}

pub struct SeqSerializer {
    inner: Vec<Value>,
}
impl ser::SerializeSeq for SeqSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.inner.push(value.serialize(Serializer::new())?);
        Ok(())
    }
    fn end(self) -> Result<Self::Ok> {
        Ok(Value::Array(self.inner))
    }
}
impl ser::SerializeTuple for SeqSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.inner.push(value.serialize(Serializer::new())?);
        Ok(())
    }
    fn end(self) -> Result<Self::Ok> {
        Ok(Value::Array(self.inner))
    }
}
impl ser::SerializeTupleStruct for SeqSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.inner.push(value.serialize(Serializer::new())?);
        Ok(())
    }
    fn end(self) -> Result<Self::Ok> {
        Ok(Value::Array(self.inner))
    }
}

pub struct SeqVariantSerializer {
    variant: &'static str,
    inner: Vec<Value>,
}
impl ser::SerializeTupleVariant for SeqVariantSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.inner.push(value.serialize(Serializer::new())?);
        Ok(())
    }
    fn end(self) -> Result<Self::Ok> {
        let mut map: Map<String, Value> = Map::with_capacity(1);
        map.insert(self.variant.to_owned(), Value::Array(self.inner));
        Ok(Value::Object(map))
    }
}

pub struct MapSerializer {
    inner: Map<String, Value>,
    next_key: Option<String>,
}
impl ser::SerializeMap for MapSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<()> {
        self.next_key = match key.serialize(Serializer::new())? {
            Value::String(s) => Some(s),
            other => {
                return Err(Error::Expected(
                    "type str".into(),
                    other.error_description().into(),
                ))
            }
        };
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        let key = self.next_key.take().unwrap_or_default();
        self.inner.insert(key, value.serialize(Serializer::new())?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Value::Object(self.inner))
    }
}

pub struct StructSerializer {
    inner: Map<String, Value>,
}
impl ser::SerializeStruct for StructSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.inner
            .insert(key.to_owned(), value.serialize(Serializer::new())?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Value::Object(self.inner))
    }
}

pub struct StructVariantSerializer {
    variant: &'static str,
    inner: Map<String, Value>,
}
impl ser::SerializeStructVariant for StructVariantSerializer {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.inner
            .insert(key.to_owned(), value.serialize(Serializer::new())?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        let mut map: Map<String, Value> = Map::with_capacity(1);
        map.insert(self.variant.to_owned(), Value::Object(self.inner));
        Ok(Value::Object(map))
    }
}
