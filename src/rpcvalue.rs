use std::collections::HashMap;
use std::fmt;
use std::string::FromUtf8Error;

use lazy_static::lazy_static;

use crate::datetime::DateTime;
use crate::decimal::Decimal;
use crate::metamap::MetaMap;

// see https://github.com/rhysd/tinyjson/blob/master/src/json_value.rs

const EMPTY_STR_REF: &str = "";
const EMPTY_BYTES_REF: &[u8] = EMPTY_STR_REF.as_bytes();
lazy_static! {
    static ref EMPTY_LIST_REF: Vec<RpcValue> = {
        let v = Vec::new();
        v
    };
    static ref EMPTY_MAP_REF: HashMap<String, RpcValue> = {
        let m = HashMap::new();
        m
    };
    static ref EMPTY_IMAP_REF: HashMap<i32, RpcValue> = {
        let m = HashMap::new();
        m
    };
    static ref EMPTY_METAMAP_REF: MetaMap = MetaMap::new();
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
	Null,
	Int(i64),
	UInt(u64),
	Double(f64),
	Bool(bool),
	DateTime(DateTime),
	Decimal(Decimal),
	//String(Box<String>),
	List(Box<Vec<RpcValue>>),
	Blob(Box<Vec<u8>>),
	Map(Box<HashMap<String, RpcValue>>),
	IMap(Box<HashMap<i32, RpcValue>>),
}

impl Value {
	pub fn type_name(&self) -> &'static str {
		match &self {
			Value::Null => "Null",
			Value::Int(_) => "Int",
			Value::UInt(_) => "UInt",
			Value::Double(_) => "Double",
			Value::Bool(_) => "Bool",
			Value::DateTime(_) => "DateTime",
			Value::Decimal(_) => "Decimal",
			Value::Blob(_) => "Blob",
			Value::List(_) => "List",
			Value::Map(_) => "Map",
			Value::IMap(_) => "IMap",
		}
	}
}

pub trait FromValue {
	fn make_value(self) -> Value;
}

impl FromValue for Value { fn make_value(self) -> Value { self } }
impl FromValue for () { fn make_value(self) -> Value { Value::Null } }
impl FromValue for &str { fn make_value(self) -> Value { Value::Blob(Box::new(self.as_bytes().to_vec())) } }
impl FromValue for &String { fn make_value(self) -> Value { Value::Blob(Box::new(self.as_bytes().to_vec())) } }
impl FromValue for i32 { fn make_value(self) -> Value { Value::Int(self as i64) } }
impl FromValue for usize { fn make_value(self) -> Value { Value::UInt(self as u64) } }
impl FromValue for chrono::NaiveDateTime {
	fn make_value(self) -> Value {
		Value::DateTime(DateTime::from_epoch_msec(self.timestamp_millis()))
	}
}
impl<Tz: chrono::TimeZone> FromValue for chrono::DateTime<Tz> {
	fn make_value(self) -> Value {
		Value::DateTime(DateTime::from_datetime(&self))
	}
}

macro_rules! from_value {
    ($from:ty, $to:ident) => {
		impl FromValue for $from {
			fn make_value(self) -> Value {
				Value::$to(self)
			}
		}
    };
}

from_value!(bool, Bool);
from_value!(i64, Int);
from_value!(u64, UInt);
from_value!(f64, Double);
from_value!(DateTime, DateTime);
from_value!(Decimal, Decimal);

macro_rules! from_value_box {
    ($from:ty, $to:ident) => {
		impl FromValue for $from {
			fn make_value(self) -> Value {
				Value::$to(Box::new(self))
			}
		}
    };
}

from_value_box!(Vec<u8>, Blob);
from_value_box!(Vec<RpcValue>, List);
from_value_box!(HashMap<String, RpcValue>, Map);
from_value_box!(HashMap<i32, RpcValue>, IMap);

#[derive(PartialEq, Clone)]
pub struct RpcValue {
	meta: Option<Box<MetaMap>>,
	value: Value
}

impl RpcValue {
	pub fn new<I>(val: I) -> RpcValue
	where I: FromValue {
		RpcValue {
			meta: None,
			value: val.make_value(),
		}
	}

	pub fn meta(&self) -> &MetaMap {
		match &self.meta {
			Some(mm) => mm,
			_ => &EMPTY_METAMAP_REF,
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

	pub(crate) fn value(&self) -> &Value {
		&self.value
	}

	pub fn type_name(&self) -> &'static str {
		&self.value.type_name()
	}

	pub fn is_null(&self) -> bool {
		match &self.value {
			Value::Null => true,
			_ => false,
		}
	}
	pub fn is_int(&self) -> bool {
		match &self.value {
			Value::Int(_) => true,
			_ => false,
		}
	}

	pub fn to_bool(&self) -> bool {
		match &self.value {
			Value::Bool(d) => *d,
			_ => false,
		}
	}
	pub fn to_i64(&self) -> i64 {
		match &self.value {
			Value::Int(d) => *d,
			_ => 0,
		}
	}
	pub fn to_i32(&self) -> i32 { self.to_i64() as i32 }
	pub fn to_u64(&self) -> u64 {
		match &self.value {
			Value::UInt(d) => *d,
			_ => 0,
		}
	}
	pub fn to_u32(&self) -> u32 { self.to_u64() as u32 }
	pub fn to_double(&self) -> f64 {
		match &self.value {
			Value::Double(d) => *d,
			_ => 0.,
		}
	}
	pub fn to_datetime(&self) -> DateTime {
		match &self.value {
			Value::DateTime(d) => d.clone(),
			_ => DateTime::invalid(),
		}
	}
	pub fn to_decimal(&self) -> Decimal {
		match &self.value {
			Value::Decimal(d) => d.clone(),
			_ => Decimal::new(0, 0),
		}
	}
	pub fn to_str(&self) -> &str {
		match &self.value {
			Value::Blob(b) => {
				let a: &[u8] = b;
				std::str::from_utf8(a).unwrap()
			},
			_ => EMPTY_STR_REF,
		}
	}
	pub fn to_bytes(&self) -> &[u8] {
		match &self.value {
			Value::Blob(b) => {
				let a: &[u8] = b;
				a
			},
			_ => EMPTY_BYTES_REF,
		}
	}
	pub fn to_string(&self) -> Result<String, FromUtf8Error> {
		match &self.value {
			Value::Blob(b) => {
				let c = b.as_ref();
				return String::from_utf8(b.as_ref().clone())
			},
			_ => Ok(String::new()),
		}
	}
	pub fn to_list(&self) -> &Vec<RpcValue> {
		match &self.value {
			Value::List(b) => &b,
			_ => &EMPTY_LIST_REF,
		}
	}
	pub fn to_map(&self) -> &HashMap<String, RpcValue> {
		match &self.value {
			Value::Map(b) => &b,
			_ => &EMPTY_MAP_REF,
		}
	}
	pub fn to_imap(&self) -> &HashMap<i32, RpcValue> {
		match &self.value {
			Value::IMap(b) => &b,
			_ => &EMPTY_IMAP_REF,
		}
	}
	pub fn to_cpon(&self) -> Vec<u8> {
		crate::cpon::writer::to_cpon(self)
	}
	pub fn to_cpon_string(&self) -> Result<String, FromUtf8Error> {
		crate::cpon::writer::to_cpon_string(self)
	}
}

impl fmt::Debug for RpcValue {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "RpcValue {{meta: {:?} value: {:?}}}", self.meta, self.value)
	}
}

#[cfg(test)]
mod test {
	use std::collections::HashMap;
	use std::mem::size_of;

	use chrono::Offset;

	use crate::DateTime;
	use crate::Decimal;
	use crate::metamap::MetaMap;
	use crate::rpcvalue::{RpcValue, Value};

	macro_rules! show_size {
		(header) => (
			println!("{:<22} {:>4}    ", "Type", "T");
			println!("------------------------------");
		);
		($t:ty) => (
			println!("{:<22} {:4}", stringify!($t), size_of::<$t>())
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
		assert_eq!(rv.to_bool(), true);
		let rv = RpcValue::new("foo");
		assert_eq!(rv.to_str(), "foo");
		let rv = RpcValue::new(&"bar".to_string());
		assert_eq!(rv.to_str(), "bar");
		let rv = RpcValue::new(123);
		assert_eq!(rv.to_i32(), 123);
		let rv = RpcValue::new(12.3);
		assert_eq!(rv.to_double(), 12.3);

		let dt = DateTime::now();
		let rv = RpcValue::new(dt.clone());
		assert_eq!(rv.to_datetime(), dt);

		let dc = Decimal::new(123, -1);
		let rv = RpcValue::new(dc.clone());
		assert_eq!(rv.to_decimal(), dc);

		let dt = chrono::offset::Utc::now();
		let rv = RpcValue::new(dt.clone());
		assert_eq!(rv.to_datetime().to_epoch_msec(), dt.timestamp_millis());

		let dt = chrono::offset::Local::now();
		let rv = RpcValue::new(dt.clone());
		assert_eq!(rv.to_datetime().to_epoch_msec() + rv.to_datetime().utc_offset() as i64 * 1000
				   , dt.timestamp_millis() + dt.offset().fix().local_minus_utc() as i64 * 1000);

		let vec1 = vec![RpcValue::new(123), RpcValue::new("foo")];
		let rv = RpcValue::new(vec1.clone());
		assert_eq!(rv.to_list(), &vec1);

		let mut m: HashMap<String, RpcValue> = HashMap::new();
		m.insert("foo".to_string(), RpcValue::new(123));
		m.insert("bar".to_string(), RpcValue::new("foo"));
		let rv = RpcValue::new(m.clone());
		assert_eq!(rv.to_map(), &m);

		let mut m: HashMap<i32, RpcValue> = HashMap::new();
		m.insert(1, RpcValue::new(123));
		m.insert(2, RpcValue::new("foo"));
		let rv = RpcValue::new(m.clone());
		assert_eq!(rv.to_imap(), &m);
	}

}

