use std::collections::BTreeMap;
use std::fmt;
use log;

use lazy_static::lazy_static;

use crate::datetime::DateTime;
use crate::decimal::Decimal;
use crate::metamap::MetaMap;
use crate::reader::Reader;
use crate::{CponReader, ReadResult};
use crate::writer::Writer;
use crate::CponWriter;
use crate::chainpack::ChainPackWriter;
use crate::chainpack::ChainPackReader;
use std::str::Utf8Error;

// see https://github.com/rhysd/tinyjson/blob/master/src/json_value.rs

const EMPTY_STR_REF: &str = "";
const EMPTY_BYTES_REF: &[u8] = EMPTY_STR_REF.as_bytes();
lazy_static! {
    static ref EMPTY_LIST_REF: Vec<RpcValue> = {
        let v = Vec::new();
        v
    };
    static ref EMPTY_MAP_REF: Map = {
        let m = BTreeMap::new();
        m
    };
    static ref EMPTY_IMAP_REF: IMap = {
        let m = BTreeMap::new();
        m
    };
    static ref EMPTY_METAMAP_REF: MetaMap = MetaMap::new();
}

pub type Data = Vec<u8>;
pub type List = Vec<RpcValue>;
pub type Map = BTreeMap<String, RpcValue>;
pub type IMap = BTreeMap<i32, RpcValue>;

#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
	Null,
	Int(i64),
	UInt(u64),
	Double(f64),
	Bool(bool),
	DateTime(DateTime),
	Decimal(Decimal),
	Data(Box<Data>),
	//Blob(Box<Blob>),
	List(Box<List>),
	Map(Box<Map>),
	IMap(Box<IMap>),
}

impl Value {
	pub(crate) fn new(v: impl FromValue) -> Value {
		v.chainpack_make_value()
	}
	pub fn type_name(&self) -> &'static str {
		match &self {
			Value::Null => "Null",
			Value::Int(_) => "Int",
			Value::UInt(_) => "UInt",
			Value::Double(_) => "Double",
			Value::Bool(_) => "Bool",
			Value::DateTime(_) => "DateTime",
			Value::Decimal(_) => "Decimal",
			Value::Data(_) => "Data",
			//Value::Blob(_) => "Blob",
			Value::List(_) => "List",
			Value::Map(_) => "Map",
			Value::IMap(_) => "IMap",
		}
	}
}

pub trait FromValue {
	fn chainpack_make_value(self) -> Value;
}

impl FromValue for Value { fn chainpack_make_value(self) -> Value { self } }
impl FromValue for () { fn chainpack_make_value(self) -> Value { Value::Null } }
impl FromValue for &str { fn chainpack_make_value(self) -> Value { Value::Data(Box::new(self.as_bytes().to_vec())) } }
impl FromValue for String { fn chainpack_make_value(self) -> Value { Value::Data(Box::new(self.into_bytes())) } }
impl FromValue for Vec<u8> { fn chainpack_make_value(self) -> Value { Value::Data(Box::new(self)) } }
impl FromValue for &[u8] { fn chainpack_make_value(self) -> Value { Value::Data(Box::new(self.to_vec())) } }
impl FromValue for &String { fn chainpack_make_value(self) -> Value { Value::Data(Box::new(self.as_bytes().to_vec())) } }
impl FromValue for i32 { fn chainpack_make_value(self) -> Value { Value::Int(self as i64) } }
impl FromValue for u32 { fn chainpack_make_value(self) -> Value { Value::UInt(self as u64) } }
impl FromValue for isize { fn chainpack_make_value(self) -> Value { Value::Int(self as i64) } }
impl FromValue for usize { fn chainpack_make_value(self) -> Value { Value::UInt(self as u64) } }
impl FromValue for chrono::NaiveDateTime {
	fn chainpack_make_value(self) -> Value {
		Value::DateTime(DateTime::from_epoch_msec(self.timestamp_millis()))
	}
}
impl<Tz: chrono::TimeZone> FromValue for chrono::DateTime<Tz> {
	fn chainpack_make_value(self) -> Value {
		Value::DateTime(DateTime::from_datetime(&self))
	}
}

macro_rules! from_value {
    ($from:ty, $to:ident) => {
		impl FromValue for $from {
			fn chainpack_make_value(self) -> Value {
				Value::$to(self)
			}
		}
    };
}

from_value!(bool, Bool);
//from_value!(i32, Int);
from_value!(i64, Int);
//from_value!(u32, UInt);
from_value!(u64, UInt);
from_value!(f64, Double);
from_value!(DateTime, DateTime);
from_value!(Decimal, Decimal);

macro_rules! from_value_box {
    ($from:ty, $to:ident) => {
		impl FromValue for $from {
			fn chainpack_make_value(self) -> Value {
				Value::$to(Box::new(self))
			}
		}
    };
}

//from_value_box!(Bytes, Bytes);
from_value_box!(Vec<RpcValue>, List);
from_value_box!(BTreeMap<String, RpcValue>, Map);
from_value_box!(BTreeMap<i32, RpcValue>, IMap);

macro_rules! is_xxx {
    ($name:ident, $variant:pat) => {
        pub fn $name(&self) -> bool {
            match self.value() {
                $variant => true,
                _ => false,
            }
        }
    };
}

#[derive(PartialEq, Clone)]
pub struct RpcValue {
	meta: Option<Box<MetaMap>>,
	value: Value
}

impl RpcValue {
	pub fn default() -> RpcValue {
		RpcValue {
			meta: None,
			value: Value::Null,
		}
	}
	pub fn new<I>(val: I) -> RpcValue
		where I: FromValue
	{
		RpcValue {
			meta: None,
			value: val.chainpack_make_value(),
		}
	}
	pub fn new_with_meta<I>(val: I, meta: Option<MetaMap>) -> RpcValue
		where I: FromValue
	{
		let mm = match meta {
			None => None,
			Some(m) => Some(Box::new(m)),
		};
		RpcValue {
			meta: mm,
			value: val.chainpack_make_value(),
		}
	}

	pub fn has_meta(&self) -> bool {
		match &self.meta {
			Some(_) => true,
			_ => false,
		}
	}
	pub fn meta(&self) -> &MetaMap {
		match &self.meta {
			Some(mm) => mm,
			_ => &EMPTY_METAMAP_REF,
		}
	}
	pub fn meta_mut(&mut self) -> Option<&mut MetaMap> {
		match &mut self.meta {
			Some(mm) => Some(mm.as_mut()),
			_ => None,
		}
	}
	pub fn clear_meta(&mut self) {
		self.meta = None;
	}
	pub fn set_meta(&mut self, m: MetaMap) {
		if m.is_empty() {
			self.meta = None;
		}
		else {
			self.meta = Some(Box::new(m));
		}
	}

	pub fn value(&self) -> &Value {
		&self.value
	}
	pub fn value_mut(&mut self) -> &mut Value {
		&mut self.value
	}

	pub fn type_name(&self) -> &'static str {
		&self.value.type_name()
	}

	is_xxx!(is_null, Value::Null);
	is_xxx!(is_bool, Value::Bool(_));
	is_xxx!(is_int, Value::Int(_));
	is_xxx!(is_data, Value::Data(_));
	is_xxx!(is_list, Value::List(_));
	is_xxx!(is_map, Value::Map(_));
	is_xxx!(is_imap, Value::IMap(_));

	pub fn as_bool(&self) -> bool {
		match &self.value {
			Value::Bool(d) => *d,
			_ => false,
		}
	}
	pub fn as_int(&self) -> i64 {
		return self.as_i64()
	}
	pub fn as_i64(&self) -> i64 {
		match &self.value {
			Value::Int(d) => *d,
			Value::UInt(d) => *d as i64,
			_ => 0,
		}
	}
	pub fn as_i32(&self) -> i32 { self.as_i64() as i32 }
	pub fn as_u64(&self) -> u64 {
		match &self.value {
			Value::Int(d) => *d as u64,
			Value::UInt(d) => *d,
			_ => 0,
		}
	}
	pub fn as_u32(&self) -> u32 { self.as_u64() as u32 }
	pub fn as_f64(&self) -> f64 {
		match &self.value {
			Value::Double(d) => *d,
			_ => 0.,
		}
	}
	pub fn as_datetime(&self) -> DateTime {
		match &self.value {
			Value::DateTime(d) => d.clone(),
			_ => DateTime::invalid(),
		}
	}
	pub fn as_decimal(&self) -> Decimal {
		match &self.value {
			Value::Decimal(d) => d.clone(),
			_ => Decimal::new(0, 0),
		}
	}
	pub fn as_str(&self) -> Result<&str, Utf8Error> {
		match &self.value {
			Value::Data(b) => std::str::from_utf8(b),
			_ => std::str::from_utf8(EMPTY_BYTES_REF),
		}
	}
	pub fn as_data(&self) -> &[u8] {
		match &self.value {
			Value::Data(b) => b,
			_ => EMPTY_BYTES_REF,
		}
	}
	pub fn as_list(&self) -> &Vec<RpcValue> {
		match &self.value {
			Value::List(b) => &b,
			_ => &EMPTY_LIST_REF,
		}
	}
	pub fn as_map(&self) -> &Map {
		match &self.value {
			Value::Map(b) => &b,
			_ => &EMPTY_MAP_REF,
		}
	}
	pub fn as_imap(&self) -> &BTreeMap<i32, RpcValue> {
		match &self.value {
			Value::IMap(b) => &b,
			_ => &EMPTY_IMAP_REF,
		}
	}
	pub fn to_cpon(&self) -> String {
		let mut buff: Vec<u8> = Vec::new();
		let mut wr = CponWriter::new(&mut buff);
		let res = wr.write(self);
		if let Err(e) = res {
			log::warn!("to_cpon write with error: {}", e);
			return String::new()
		}
		match String::from_utf8(buff) {
			Ok(s) => s,
			Err(_) => String::new(),
		}
	}
	pub fn to_chainpack(&self) -> Vec<u8> {
		let mut buff: Vec<u8> = Vec::new();
		let mut wr = ChainPackWriter::new(&mut buff);
		let r = wr.write(self);
		match r {
			Ok(_) => buff,
			Err(_) => Vec::new(),
		}
	}

	pub fn from_cpon(s: &str) -> ReadResult {
		let mut buff = s.as_bytes();
		let mut rd = CponReader::new(&mut buff);
		rd.read()
	}
	pub fn from_chainpack(b: &[u8]) -> ReadResult {
		let mut buff = b;
		let mut rd = ChainPackReader::new(&mut buff);
		rd.read()
	}

}

impl fmt::Debug for RpcValue {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		//write!(f, "RpcValue {{meta: {:?} value: {:?}}}", self.meta, self.value)
		write!(f, "{}", self.to_cpon())
	}
}
impl fmt::Display for RpcValue {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.to_cpon())
	}
}

#[cfg(test)]
mod test {
	use std::collections::BTreeMap;
	use std::mem::size_of;

	use chrono::Offset;

	use crate::{DateTime, Data};
	use crate::Decimal;
	use crate::metamap::MetaMap;
	use crate::rpcvalue::{RpcValue, Value, Map};

	macro_rules! show_size {
		(header) => (
			log::debug!("{:<22} {:>4}    ", "Type", "T");
			log::debug!("------------------------------");
		);
		($t:ty) => (
			log::debug!("{:<22} {:4}", stringify!($t), size_of::<$t>())
		)
	}

	#[test]
	fn size() {
		show_size!(header);
		show_size!(usize);
		show_size!(MetaMap);
		show_size!(Box<MetaMap>);
		show_size!(Option<MetaMap>);
		show_size!(Option<Box<MetaMap>>);
		show_size!(Value);
		show_size!(Option<Value>);
		show_size!(RpcValue);
	}

	#[test]
	fn rpcval_new()
	{
		let rv = RpcValue::new(true);
		assert_eq!(rv.as_bool(), true);
		let rv = RpcValue::new("foo");
		assert_eq!(rv.as_str().unwrap(), "foo");
		let rv = RpcValue::new(&"bar".to_string());
		assert_eq!(rv.as_data(), b"bar");
		let rv = RpcValue::new(123);
		assert_eq!(rv.as_i32(), 123);
		let rv = RpcValue::new(12.3);
		assert_eq!(rv.as_f64(), 12.3);

		let dt = DateTime::now();
		let rv = RpcValue::new(dt.clone());
		assert_eq!(rv.as_datetime(), dt);

		let dc = Decimal::new(123, -1);
		let rv = RpcValue::new(dc.clone());
		assert_eq!(rv.as_decimal(), dc);

		let dt = chrono::offset::Utc::now();
		let rv = RpcValue::new(dt.clone());
		assert_eq!(rv.as_datetime().epoch_msec(), dt.timestamp_millis());

		let dt = chrono::offset::Local::now();
		let rv = RpcValue::new(dt.clone());
		assert_eq!(rv.as_datetime().epoch_msec() + rv.as_datetime().utc_offset() as i64 * 1000
				   , dt.timestamp_millis() + dt.offset().fix().local_minus_utc() as i64 * 1000);

		let vec1 = vec![RpcValue::new(123), RpcValue::new("foo")];
		let rv = RpcValue::new(vec1.clone());
		assert_eq!(rv.as_list(), &vec1);

		let mut m: Map = BTreeMap::new();
		m.insert("foo".to_string(), RpcValue::new(123));
		m.insert("bar".to_string(), RpcValue::new("foo"));
		let rv = RpcValue::new(m.clone());
		assert_eq!(rv.as_map(), &m);

		let mut m: BTreeMap<i32, RpcValue> = BTreeMap::new();
		m.insert(1, RpcValue::new(123));
		m.insert(2, RpcValue::new("foo"));
		let rv = RpcValue::new(m.clone());
		assert_eq!(rv.as_imap(), &m);
	}

}

