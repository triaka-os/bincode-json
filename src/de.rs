//! Deserializes Rust data from `bincode-json` data.

use crate::{
    error::{Error, Result},
    value::Value,
};
use serde::de::{self, Deserialize, Visitor};

#[cfg(not(features = "preserve_order"))]
use std::collections::hash_map as map;

#[cfg(features = "preserve_order")]
use indexmap::map;

macro_rules! forward_to_deserialize {
    ($(
        $name:ident ( $( $arg:ident : $ty:ty ),* );
    )*) => {
        $(
            forward_to_deserialize!{
                func: $name ( $( $arg: $ty ),* );
            }
        )*
    };

    (func: deserialize_enum ( $( $arg:ident : $ty:ty ),* );) => {
        fn deserialize_enum<V>(
            self,
            $(_: $ty,)*
            _visitor: V,
        ) -> ::std::result::Result<V::Value, Self::Error>
            where V: ::serde::de::Visitor<'de>
        {
            Err(::serde::de::Error::custom("unexpected Enum"))
        }
    };

    (func: $name:ident ( $( $arg:ident : $ty:ty ),* );) => {
        #[inline]
        fn $name<V>(
            self,
            $(_: $ty,)*
            visitor: V,
        ) -> ::std::result::Result<V::Value, Self::Error>
            where V: ::serde::de::Visitor<'de>
        {
            self.deserialize_any(visitor)
        }
    };
}

pub struct Deserializer {
    value: Option<Value>,
}
impl From<Value> for Deserializer {
    fn from(value: Value) -> Self {
        Self { value: Some(value) }
    }
}
impl<'de> de::Deserializer<'de> for Deserializer {
    type Error = Error;

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value.take() {
            Some(Value::Null) => visitor.visit_none(),
            Some(Value::Boolean(b)) => visitor.visit_bool(b),
            Some(Value::Blob(b)) => visitor.visit_byte_buf(b),
            Some(Value::Array(a)) => {
                let len = a.len();
                visitor.visit_seq(SeqDeserializer {
                    iter: a.into_iter(),
                    len,
                })
            }
            Some(Value::Integer(i)) => visitor.visit_i64(i),
            Some(Value::Float(f)) => visitor.visit_f64(f),
            Some(Value::Object(o)) => {
                let len = o.len();
                visitor.visit_map(MapDeserializer {
                    iter: o.into_iter(),
                    value: None,
                    len,
                })
            }
            Some(Value::String(s)) => visitor.visit_string(s),
            None => Err(Error::Eof),
        }
    }
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Null) => visitor.visit_none(),
            Some(_) => visitor.visit_some(self),
            None => Err(Error::Eof),
        }
    }
    fn deserialize_enum<V>(
        mut self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = match self.value.take() {
            Some(Value::Object(value)) => value,
            Some(Value::String(variant)) => {
                return visitor.visit_enum(EnumDeserializer {
                    val: Value::String(variant),
                    deserializer: VariantDeserializer { val: None },
                });
            }
            Some(v) => {
                return Err(Error::Expected(
                    v.error_description().into(),
                    "expected an enum".into(),
                ));
            }
            None => {
                return Err(Error::Eof);
            }
        };

        let mut iter = value.into_iter();

        let (variant, value) = match iter.next() {
            Some(v) => v,
            None => {
                return Err(Error::Expected(
                    "variant name".into(),
                    "empty object".into(),
                ))
            }
        };

        match iter.next() {
            Some((k, _)) => Err(Error::Expected(
                "map with a single key".into(),
                format!("extra key \"{}\"", k),
            )),
            None => visitor.visit_enum(EnumDeserializer {
                val: Value::String(variant),
                deserializer: VariantDeserializer { val: Some(value) },
            }),
        }
    }
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    forward_to_deserialize! {
        deserialize_bool();
        deserialize_u8();
        deserialize_u16();
        deserialize_u32();
        deserialize_u64();
        deserialize_i8();
        deserialize_i16();
        deserialize_i32();
        deserialize_i64();
        deserialize_f32();
        deserialize_f64();
        deserialize_char();
        deserialize_str();
        deserialize_bytes();
        deserialize_string();
        deserialize_unit();
        deserialize_seq();
        deserialize_map();
        deserialize_unit_struct(name: &'static str);
        deserialize_tuple_struct(name: &'static str, len: usize);
        deserialize_struct(name: &'static str, fields: &'static [&'static str]);
        deserialize_tuple(len: usize);
        deserialize_identifier();
        deserialize_ignored_any();
        deserialize_byte_buf();
    }
}

struct SeqDeserializer {
    iter: std::vec::IntoIter<Value>,
    len: usize,
}
impl<'de> de::Deserializer<'de> for SeqDeserializer {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.len == 0 {
            visitor.visit_unit()
        } else {
            visitor.visit_seq(self)
        }
    }
    forward_to_deserialize! {
        deserialize_bool();
        deserialize_u8();
        deserialize_u16();
        deserialize_u32();
        deserialize_u64();
        deserialize_i8();
        deserialize_i16();
        deserialize_i32();
        deserialize_i64();
        deserialize_f32();
        deserialize_f64();
        deserialize_char();
        deserialize_str();
        deserialize_string();
        deserialize_unit();
        deserialize_option();
        deserialize_seq();
        deserialize_bytes();
        deserialize_map();
        deserialize_unit_struct(name: &'static str);
        deserialize_newtype_struct(name: &'static str);
        deserialize_tuple_struct(name: &'static str, len: usize);
        deserialize_struct(name: &'static str, fields: &'static [&'static str]);
        deserialize_tuple(len: usize);
        deserialize_enum(name: &'static str, variants: &'static [&'static str]);
        deserialize_identifier();
        deserialize_ignored_any();
        deserialize_byte_buf();
    }
}
impl<'de> de::SeqAccess<'de> for SeqDeserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            None => Ok(None),
            Some(value) => {
                self.len -= 1;
                let de = Deserializer::from(value);
                match seed.deserialize(de) {
                    Ok(value) => Ok(Some(value)),
                    Err(err) => Err(err),
                }
            }
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

struct MapDeserializer {
    iter: map::IntoIter<String, Value>,
    value: Option<Value>,
    len: usize,
}
impl<'de> de::MapAccess<'de> for MapDeserializer {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.len -= 1;
                self.value = Some(value);

                let de = Deserializer::from(Value::String(key));
                match seed.deserialize(de) {
                    Ok(val) => Ok(Some(val)),
                    Err(e) => Err(e),
                }
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        let value = self.value.take().ok_or(Error::Eof)?;
        let de = Deserializer::from(value);
        seed.deserialize(de)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

impl<'de> de::Deserializer<'de> for MapDeserializer {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }
    forward_to_deserialize! {
        deserialize_bool();
        deserialize_u8();
        deserialize_u16();
        deserialize_u32();
        deserialize_u64();
        deserialize_i8();
        deserialize_i16();
        deserialize_i32();
        deserialize_i64();
        deserialize_f32();
        deserialize_f64();
        deserialize_char();
        deserialize_str();
        deserialize_string();
        deserialize_unit();
        deserialize_option();
        deserialize_seq();
        deserialize_bytes();
        deserialize_map();
        deserialize_unit_struct(name: &'static str);
        deserialize_newtype_struct(name: &'static str);
        deserialize_tuple_struct(name: &'static str, len: usize);
        deserialize_struct(name: &'static str, fields: &'static [&'static str]);
        deserialize_tuple(len: usize);
        deserialize_enum(name: &'static str, variants: &'static [&'static str]);
        deserialize_identifier();
        deserialize_ignored_any();
        deserialize_byte_buf();
    }
}

struct EnumDeserializer {
    val: Value,
    deserializer: VariantDeserializer,
}

impl<'de> de::EnumAccess<'de> for EnumDeserializer {
    type Error = Error;
    type Variant = VariantDeserializer;
    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let dec = Deserializer::from(self.val);
        let value = seed.deserialize(dec)?;
        Ok((value, self.deserializer))
    }
}

struct VariantDeserializer {
    val: Option<Value>,
}

impl<'de> de::VariantAccess<'de> for VariantDeserializer {
    type Error = Error;

    fn unit_variant(mut self) -> Result<()> {
        match self.val.take() {
            None => Ok(()),
            Some(val) => Value::deserialize(Deserializer::from(val)).map(|_| ()),
        }
    }

    fn newtype_variant_seed<T>(mut self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        let dec = Deserializer::from(self.val.take().ok_or(Error::Eof)?);
        seed.deserialize(dec)
    }

    fn tuple_variant<V>(mut self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.val.take().ok_or(Error::Eof)? {
            Value::Array(fields) => {
                let des = SeqDeserializer {
                    len: fields.len(),
                    iter: fields.into_iter(),
                };
                de::Deserializer::deserialize_any(des, visitor)
            }
            other => Err(Error::Expected(
                other.error_description().into(),
                "expected a tuple".into(),
            )),
        }
    }

    fn struct_variant<V>(mut self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.val.take().ok_or(Error::Eof)? {
            Value::Object(fields) => {
                let des = MapDeserializer {
                    len: fields.len(),
                    iter: fields.into_iter(),
                    value: None,
                };
                de::Deserializer::deserialize_any(des, visitor)
            }
            ref other => Err(Error::Expected(
                other.error_description().into(),
                "expected a struct".into(),
            )),
        }
    }
}
