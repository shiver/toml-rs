//! TOML serialization support.

use std::fmt::{self, Write};
use std::error;
use std::cell::Cell;

use serde::ser;

/// Serialize the given data structure as a TOML byte vector.
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to
/// fail, if `T` contains a map with non-string keys, or if `T` attempts to
/// serialize an unsupported datatype such as an enum, tuple, or tuple struct.
pub fn to_vec<T: ?Sized>(value: &T) -> Result<Vec<u8>, Error>
    where T: ser::Serialize,
{
    to_string(value).map(|e| e.into_bytes())
}

/// Serialize the given data structure as a String of TOML.
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to
/// fail, if `T` contains a map with non-string keys, or if `T` attempts to
/// serialize an unsupported datatype such as an enum, tuple, or tuple struct.
pub fn to_string<T: ?Sized>(value: &T) -> Result<String, Error>
    where T: ser::Serialize,
{
    let mut dst = String::with_capacity(128);
    value.serialize(Serializer::new(&mut dst))?;
    Ok(dst)
}

/// Errors that can occur when serializing a type.
#[derive(Debug, Clone)]
pub enum Error {
    /// Indicates that a Rust type was requested to be serialized but it was not
    /// supported.
    ///
    /// Currently the TOML format does not support serializing types such as
    /// enums, tuples and tuple structs.
    UnsupportedType,

    /// The key of all TOML maps must be strings, but serialization was
    /// attempted where the key of a map was not a string.
    KeyNotString,

    /// Keys in maps are not allowed to have newlines.
    KeyNewline,

    /// Arrays in TOML must have a homogenous type, but a heterogeneous array
    /// was emitted.
    ArrayMixedType,

    /// All values in a TOML table must be emitted before further tables are
    /// emitted. If a value is emitted *after* a table then this error is
    /// generated.
    ValueAfterTable,

    /// A custom error which could be generated when serializing a particular
    /// type.
    Custom(String),

    #[doc(hidden)]
    __Nonexhaustive,
}

/// Serialization implementation for TOML.
///
/// This structure implements serialization support for TOML to serialize an
/// arbitrary type to TOML. Note that the TOML format does not support all
/// datatypes in Rust, such as enums, tuples, and tuple structs. These types
/// will generate an error when serialized.
///
/// Currently a serializer always writes its output to an in-memory `String`,
/// which is passed in when creating the serializer itself.
pub struct Serializer<'a> {
    dst: &'a mut String,
    state: State<'a>,
}

#[derive(Debug, Clone)]
enum State<'a> {
    Table {
        key: &'a str,
        parent: &'a State<'a>,
        first: &'a Cell<bool>,
        table_emitted: &'a Cell<bool>,
    },
    Array {
        parent: &'a State<'a>,
        first: &'a Cell<bool>,
        type_: &'a Cell<Option<&'static str>>,
    },
    End,
}

#[doc(hidden)]
pub struct SerializeSeq<'a> {
    ser: Serializer<'a>,
    first: Cell<bool>,
    type_: Cell<Option<&'static str>>,
}

#[doc(hidden)]
pub struct SerializeTable<'a> {
    ser: Serializer<'a>,
    key: String,
    first: Cell<bool>,
    table_emitted: Cell<bool>,
}

impl<'a> Serializer<'a> {
    /// Creates a new serializer which will emit TOML into the buffer provided.
    ///
    /// The serializer can then be used to serialize a type after which the data
    /// will be present in `dst`.
    pub fn new(dst: &'a mut String) -> Serializer<'a> {
        Serializer {
            dst: dst,
            state: State::End,
        }
    }

    fn display<T: fmt::Display>(mut self,
                                t: T,
                                type_: &'static str) -> Result<(), Error> {
        self.emit_key(type_)?;
        drop(write!(self.dst, "{}", t));
        if let State::Table { .. } = self.state {
            self.dst.push_str("\n");
        }
        Ok(())
    }

    fn emit_key(&mut self, type_: &'static str) -> Result<(), Error> {
        println!("emit_key -- {:?}", self.state);
        self.array_type(type_)?;
        let state = self.state.clone();
        self.emit_key_(&state)
    }

    fn emit_key_(&mut self, state: &State) -> Result<(), Error> {
        match *state {
            State::End => Ok(()),
            State::Array { parent, first, type_ } => {
                assert!(type_.get().is_some());
                if first.get() {
                    self.emit_key_(parent)?;
                }
                self.emit_array(first)
            }
            State::Table { parent, first, table_emitted, key } => {
                if table_emitted.get() {
                    return Err(Error::ValueAfterTable)
                }
                if first.get() {
                    self.emit_table_header(parent)?;
                    first.set(false);
                }
                self.escape_key(key)?;
                self.dst.push_str(" = ");
                Ok(())
            }
        }
    }

    fn emit_array(&mut self, first: &Cell<bool>) -> Result<(), Error> {
        if first.get() {
            self.dst.push_str("[");
        } else {
            self.dst.push_str(", ");
        }
        Ok(())
    }

    fn array_type(&mut self, type_: &'static str) -> Result<(), Error> {
        let prev = match self.state {
            State::Array { type_, .. } => type_,
            _ => return Ok(()),
        };
        if let Some(prev) = prev.get() {
            if prev != type_ {
                return Err(Error::ArrayMixedType)
            }
        } else {
            prev.set(Some(type_));
        }
        Ok(())
    }

    fn escape_key(&mut self, key: &str) -> Result<(), Error> {
        let ok = key.chars().all(|c| {
            match c {
                'a' ... 'z' |
                'A' ... 'Z' |
                '0' ... '9' |
                '-' | '_' => true,
                _ => false,
            }
        });
        if ok {
            drop(write!(self.dst, "{}", key));
        } else {
            self.emit_str(key)?;
        }
        Ok(())
    }

    fn emit_str(&mut self, value: &str) -> Result<(), Error> {
        drop(write!(self.dst, "\""));
        for ch in value.chars() {
            match ch {
                '\u{8}' => drop(write!(self.dst, "\\b")),
                '\u{9}' => drop(write!(self.dst, "\\t")),
                '\u{a}' => drop(write!(self.dst, "\\n")),
                '\u{c}' => drop(write!(self.dst, "\\f")),
                '\u{d}' => drop(write!(self.dst, "\\r")),
                '\u{22}' => drop(write!(self.dst, "\\\"")),
                '\u{5c}' => drop(write!(self.dst, "\\\\")),
                c if c < '\u{1f}' => {
                    drop(write!(self.dst, "\\u{:04}", ch as u32))
                }
                ch => drop(write!(self.dst, "{}", ch)),
            }
        }
        drop(write!(self.dst, "\""));
        Ok(())
    }

    fn emit_table_header(&mut self, state: &State) -> Result<(), Error> {
        let array_of_tables = match *state {
            State::End => return Ok(()),
            State::Array { .. } => true,
            _ => false,
        };
        match *state {
            State::Table { first , .. } |
            State::Array { parent: &State::Table { first, .. }, .. } => {
                if !first.get() {
                    self.dst.push_str("\n");
                }
            }
            _ => {}
        }
        self.dst.push_str("[");
        if array_of_tables {
            self.dst.push_str("[");
        }
        self.emit_key_part(&state)?;
        if array_of_tables {
            self.dst.push_str("]");
        }
        self.dst.push_str("]\n");
        Ok(())
    }

    fn emit_key_part(&mut self, key: &State) -> Result<bool, Error> {
        match *key {
            State::Array { parent, .. } => self.emit_key_part(parent),
            State::End => Ok(true),
            State::Table { key, parent, table_emitted, .. } => {
                table_emitted.set(true);
                let first  = self.emit_key_part(parent)?;
                if !first {
                    self.dst.push_str(".");
                }
                self.escape_key(key)?;
                Ok(false)
            }
        }
    }
}

impl<'a> ser::Serializer for Serializer<'a> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SerializeSeq<'a>;
    type SerializeTuple = Impossible;
    type SerializeTupleStruct = Impossible;
    type SerializeTupleVariant = Impossible;
    type SerializeMap = SerializeTable<'a>;
    type SerializeStruct = SerializeTable<'a>;
    type SerializeStructVariant = Impossible;

    fn serialize_bool(self, v: bool) -> Result<(), Self::Error> {
        self.display(v, "bool")
	}

    fn serialize_i8(self, v: i8) -> Result<(), Self::Error> {
        self.display(v, "integer")
    }

    fn serialize_i16(self, v: i16) -> Result<(), Self::Error> {
        self.display(v, "integer")
    }

    fn serialize_i32(self, v: i32) -> Result<(), Self::Error> {
        self.display(v, "integer")
    }

    fn serialize_i64(self, v: i64) -> Result<(), Self::Error> {
        self.display(v, "integer")
    }

    fn serialize_u8(self, v: u8) -> Result<(), Self::Error> {
        self.display(v, "integer")
    }

    fn serialize_u16(self, v: u16) -> Result<(), Self::Error> {
        self.display(v, "integer")
    }

    fn serialize_u32(self, v: u32) -> Result<(), Self::Error> {
        self.display(v, "integer")
    }

    fn serialize_u64(self, v: u64) -> Result<(), Self::Error> {
        self.display(v, "integer")
    }

    fn serialize_f32(mut self, v: f32) -> Result<(), Self::Error> {
        self.emit_key("float")?;
        drop(write!(self.dst, "{}", v));
        if v % 1.0 == 0.0 {
            drop(write!(self.dst, ".0"));
        }
        Ok(())
    }

    fn serialize_f64(mut self, v: f64) -> Result<(), Self::Error> {
        self.emit_key("float")?;
        drop(write!(self.dst, "{}", v));
        if v % 1.0 == 0.0 {
            drop(write!(self.dst, ".0"));
        }
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<(), Self::Error> {
        let mut buf = [0; 4];
        self.serialize_str(v.encode_utf8(&mut buf))
    }

    fn serialize_str(mut self, value: &str) -> Result<(), Self::Error> {
        self.emit_key("string")?;
        self.emit_str(value)?;
        if let State::Table { .. } = self.state {
            self.dst.push_str("\n");
        }
        Ok(())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<(), Self::Error> {
        Err(Error::UnsupportedType)
    }

    fn serialize_none(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<(), Self::Error>
        where T: ser::Serialize
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<(), Self::Error> {
        Err(Error::UnsupportedType)
    }

    fn serialize_unit_struct(self,
                             _name: &'static str)
                             -> Result<(), Self::Error> {
        Err(Error::UnsupportedType)
    }

    fn serialize_unit_variant(self,
                              _name: &'static str,
                              _variant_index: usize,
                              _variant: &'static str)
                              -> Result<(), Self::Error> {
        Err(Error::UnsupportedType)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T)
                                           -> Result<(), Self::Error>
        where T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(self,
                                            _name: &'static str,
                                            _variant_index: usize,
                                            _variant: &'static str,
                                            _value: &T)
                                            -> Result<(), Self::Error>
        where T: ser::Serialize,
    {
        Err(Error::UnsupportedType)
    }

    fn serialize_seq(mut self, _len: Option<usize>)
                     -> Result<Self::SerializeSeq, Self::Error> {
        self.array_type("array")?;
        Ok(SerializeSeq {
            ser: self,
            first: Cell::new(true),
            type_: Cell::new(None),
        })
    }

    fn serialize_seq_fixed_size(self, size: usize)
                                -> Result<Self::SerializeSeq, Self::Error> {
        self.serialize_seq(Some(size))
    }

    fn serialize_tuple(self, _len: usize)
                       -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::UnsupportedType)
    }

    fn serialize_tuple_struct(self, _name: &'static str, _len: usize)
                              -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::UnsupportedType)
    }

    fn serialize_tuple_variant(self,
                               _name: &'static str,
                               _variant_index: usize,
                               _variant: &'static str,
                               _len: usize)
                               -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::UnsupportedType)
    }

    fn serialize_map(mut self, _len: Option<usize>)
                     -> Result<Self::SerializeMap, Self::Error> {
        self.array_type("table")?;
        Ok(SerializeTable {
            ser: self,
            key: String::new(),
            first: Cell::new(true),
            table_emitted: Cell::new(false),
        })
    }

    fn serialize_struct(mut self, _name: &'static str, _len: usize)
                        -> Result<Self::SerializeStruct, Self::Error> {
        self.array_type("table")?;
        Ok(SerializeTable {
            ser: self,
            key: String::new(),
            first: Cell::new(true),
            table_emitted: Cell::new(false),
        })
    }

    fn serialize_struct_variant(self,
                                _name: &'static str,
                                _variant_index: usize,
                                _variant: &'static str,
                                _len: usize)
                                -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::UnsupportedType)
    }
}

impl<'a> ser::SerializeSeq for SerializeSeq<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
        where T: ser::Serialize,
    {
        value.serialize(Serializer {
            dst: &mut *self.ser.dst,
            state: State::Array {
                parent: &self.ser.state,
                first: &self.first,
                type_: &self.type_,
            },
        })?;
        self.first.set(false);
        Ok(())
    }

    fn end(self) -> Result<(), Error> {
        match self.type_.get() {
            Some("table") => return Ok(()),
            Some(_) => self.ser.dst.push_str("]"),
            None => self.ser.dst.push_str("[]"),
        }
        if let State::Table { .. } = self.ser.state {
            self.ser.dst.push_str("\n");
        }
        Ok(())
    }
}

impl<'a> ser::SerializeMap for SerializeTable<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Error>
        where T: ser::Serialize,
    {
        self.key.truncate(0);
        key.serialize(StringExtractor { slot: &mut self.key })?;
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
        where T: ser::Serialize,
    {
        value.serialize(Serializer {
            dst: &mut *self.ser.dst,
            state: State::Table {
                key: &self.key,
                parent: &self.ser.state,
                first: &self.first,
                table_emitted: &self.table_emitted,
            },
        })?;
        self.first.set(false);
        Ok(())
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for SerializeTable<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T)
                                  -> Result<(), Error>
        where T: ser::Serialize,
    {
        value.serialize(Serializer {
            dst: &mut *self.ser.dst,
            state: State::Table {
                key: key,
                parent: &self.ser.state,
                first: &self.first,
                table_emitted: &self.table_emitted,
            },
        })?;
        self.first.set(false);
        Ok(())
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

#[doc(hidden)]
pub enum Impossible {}

impl ser::SerializeSeq for Impossible {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, _value: &T) -> Result<(), Error>
        where T: ser::Serialize,
    {
        match *self {}
    }

    fn end(self) -> Result<(), Error> {
        match self {}
    }
}

impl ser::SerializeMap for Impossible {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> Result<(), Error>
        where T: ser::Serialize,
    {
        match *self {}
    }

    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<(), Error>
        where T: ser::Serialize,
    {
        match *self {}
    }

    fn end(self) -> Result<(), Error> {
        match self {}
    }
}

impl ser::SerializeStruct for Impossible {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, _value: &T)
                                  -> Result<(), Error>
        where T: ser::Serialize,
    {
        match *self {}
    }

    fn end(self) -> Result<(), Error> {
        match self {}
    }
}

impl ser::SerializeTuple for Impossible {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, _value: &T) -> Result<(), Error>
        where T: ser::Serialize,
    {
        match *self {}
    }

    fn end(self) -> Result<(), Error> {
        match self {}
    }
}

impl ser::SerializeTupleStruct for Impossible {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<(), Error>
        where T: ser::Serialize,
    {
        match *self {}
    }

    fn end(self) -> Result<(), Error> {
        match self {}
    }
}

impl ser::SerializeTupleVariant for Impossible {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<(), Error>
        where T: ser::Serialize,
    {
        match *self {}
    }

    fn end(self) -> Result<(), Error> {
        match self {}
    }
}

impl ser::SerializeStructVariant for Impossible {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, _value: &T)
                                  -> Result<(), Error>
        where T: ser::Serialize,
    {
        match *self {}
    }

    fn end(self) -> Result<(), Error> {
        match self {}
    }
}

struct StringExtractor<'a> {
    slot: &'a mut String,
}

impl<'a> ser::Serializer for StringExtractor<'a> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Impossible;
    type SerializeTuple = Impossible;
    type SerializeTupleStruct = Impossible;
    type SerializeTupleVariant = Impossible;
    type SerializeMap = Impossible;
    type SerializeStruct = Impossible;
    type SerializeStructVariant = Impossible;

    fn serialize_bool(self, _v: bool) -> Result<(), Self::Error> {
        Err(Error::KeyNotString)
	}

    fn serialize_i8(self, _v: i8) -> Result<(), Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_i16(self, _v: i16) -> Result<(), Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_i32(self, _v: i32) -> Result<(), Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_i64(self, _v: i64) -> Result<(), Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_u8(self, _v: u8) -> Result<(), Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_u16(self, _v: u16) -> Result<(), Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_u32(self, _v: u32) -> Result<(), Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_u64(self, _v: u64) -> Result<(), Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_f32(self, _v: f32) -> Result<(), Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_f64(self, _v: f64) -> Result<(), Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_char(self, _v: char) -> Result<(), Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_str(self, value: &str) -> Result<(), Self::Error> {
        if value.contains("\n") {
            return Err(Error::KeyNewline)
        }
        assert_eq!(self.slot.len(), 0);
        self.slot.push_str(value);
        Ok(())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<(), Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_none(self) -> Result<(), Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<(), Self::Error>
        where T: ser::Serialize
    {
        Err(Error::KeyNotString)
    }

    fn serialize_unit(self) -> Result<(), Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_unit_struct(self,
                             _name: &'static str)
                             -> Result<(), Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_unit_variant(self,
                              _name: &'static str,
                              _variant_index: usize,
                              _variant: &'static str)
                              -> Result<(), Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, _value: &T)
                                           -> Result<(), Self::Error>
        where T: ser::Serialize,
    {
        Err(Error::KeyNotString)
    }

    fn serialize_newtype_variant<T: ?Sized>(self,
                                            _name: &'static str,
                                            _variant_index: usize,
                                            _variant: &'static str,
                                            _value: &T)
                                            -> Result<(), Self::Error>
        where T: ser::Serialize,
    {
        Err(Error::KeyNotString)
    }

    fn serialize_seq(self, _len: Option<usize>)
                     -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_seq_fixed_size(self, _size: usize)
                                -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_tuple(self, _len: usize)
                       -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_tuple_struct(self, _name: &'static str, _len: usize)
                              -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_tuple_variant(self,
                               _name: &'static str,
                               _variant_index: usize,
                               _variant: &'static str,
                               _len: usize)
                               -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_map(self, _len: Option<usize>)
                     -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize)
                        -> Result<Self::SerializeStruct, Self::Error> {
        Err(Error::KeyNotString)
    }

    fn serialize_struct_variant(self,
                                _name: &'static str,
                                _variant_index: usize,
                                _variant: &'static str,
                                _len: usize)
                                -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::KeyNotString)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::UnsupportedType => "unsupported Rust type".fmt(f),
            Error::KeyNotString => "map key was not a string".fmt(f),
            Error::KeyNewline => "map keys cannot contain newlines".fmt(f),
            Error::ArrayMixedType => "arrays cannot have mixed types".fmt(f),
            Error::ValueAfterTable => "values must be emitted before tables".fmt(f),
            Error::Custom(ref s) => s.fmt(f),
            Error::__Nonexhaustive => panic!(),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::UnsupportedType => "unsupported Rust type",
            Error::KeyNotString => "map key was not a string",
            Error::KeyNewline => "map keys cannot contain newlines",
            Error::ArrayMixedType => "arrays cannot have mixed types",
            Error::ValueAfterTable => "values must be emitted before tables",
            Error::Custom(_) => "custom error",
            Error::__Nonexhaustive => panic!(),
        }
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Error {
        Error::Custom(msg.to_string())
    }
}
