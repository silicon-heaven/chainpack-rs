use crate::metamap::MetaMap;
use crate::datetime::DateTime;
use std::fmt;
use std::collections::HashMap;
use lazy_static::lazy_static;

// see https://github.com/rhysd/tinyjson/blob/master/src/json_value.rs

const STR_REF: &str = "";
lazy_static! {
	static ref DATETIME_REF: DateTime = DateTime::invalid();
    static ref LIST_REF: Vec<RpcValue> = {
        let v = Vec::new();
        v
    };
    static ref MAP_REF: HashMap<String, RpcValue> = {
        let m = HashMap::new();
        m
    };
    static ref IMAP_REF: HashMap<i32, RpcValue> = {
        let m = HashMap::new();
        m
    };
    static ref METAMAP_REF: MetaMap = MetaMap::new();
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
	Null,
	Int(i64),
	UInt(u64),
	Double(f64),
	Bool(bool),
	DateTime(DateTime),
	//String(Box<String>),
	List(Box<Vec<RpcValue>>),
	Bytes(Box<Vec<u8>>),
	Map(Box<HashMap<String, RpcValue>>),
	IMap(Box<HashMap<i32, RpcValue>>),
}

impl Value {
	pub fn type_name(&self) -> &'static str {
		match &self {
			Value::Null => "Null",
			Value::Int(n) => "Int",
			Value::UInt(n) => "UInt",
			Value::Double(n) => "Double",
			Value::Bool(b) => "Bool",
			Value::DateTime(dt) => "DateTime",
			Value::Bytes(b) => "Bytes",
			Value::List(l) => "List",
			Value::Map(m) => "Map",
			Value::IMap(m) => "IMap",
		}
	}
}

pub trait FromValue {
	fn from_value(self) -> Value;
}

impl FromValue for () { fn from_value(self) -> Value { Value::Null } }
impl FromValue for &str { fn from_value(self) -> Value { Value::Bytes(Box::new(self.as_bytes().to_vec())) } }
impl FromValue for &String { fn from_value(self) -> Value { Value::Bytes(Box::new(self.as_bytes().to_vec())) } }
impl FromValue for i32 { fn from_value(self) -> Value { Value::Int(self as i64) } }
impl FromValue for usize { fn from_value(self) -> Value { Value::UInt(self as u64) } }
impl FromValue for chrono::NaiveDateTime {
	fn from_value(self) -> Value {
		Value::DateTime(DateTime::from_epoch_msec(self.timestamp_millis(), 0))
	}
}
impl<Tz: chrono::TimeZone> FromValue for chrono::DateTime<Tz> {
	fn from_value(self) -> Value {
		Value::DateTime(DateTime::from_datetime(&self))
	}
}

macro_rules! from_value {
    ($from:ty, $to:ident) => {
		impl FromValue for $from {
			fn from_value(self) -> Value {
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

macro_rules! from_value_box {
    ($from:ty, $to:ident) => {
		impl FromValue for $from {
			fn from_value(self) -> Value {
				Value::$to(Box::new(self))
			}
		}
    };
}

from_value_box!(Vec<u8>, Bytes);
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
			value: val.from_value(),
		}
	}

	pub fn meta(&self) -> &MetaMap {
		match &self.meta {
			Some(mm) => mm,
			_ => &METAMAP_REF,
		}
	}
	pub fn set_meta(&mut self, m: MetaMap) {
		self.meta = Some(Box::new(m));
	}

	pub(crate) fn value(&self) -> &Value {
		&self.value
	}

	pub fn type_name(&self) -> &'static str {
		&self.value.type_name()
	}

	pub fn to_bool(&self) -> bool {
		match &self.value {
			Value::Bool(d) => *d,
			_ => false,
		}
	}
	pub fn to_i32(&self) -> i32 {
		match &self.value {
			Value::Int(d) => *d as i32,
			_ => 0,
		}
	}
	pub fn to_double(&self) -> f64 {
		match &self.value {
			Value::Double(d) => *d,
			_ => 0.,
		}
	}
	pub fn to_datetime(&self) -> &DateTime {
		match &self.value {
			Value::DateTime(d) => d,
			_ => &DATETIME_REF,
		}
	}
	pub fn to_str(&self) -> &str {
		match &self.value {
			Value::Bytes(b) => {
				let a: &[u8] = b;
				std::str::from_utf8(a).unwrap()
			},
			_ => STR_REF,
		}
	}
	pub fn to_list(&self) -> &Vec<RpcValue> {
		match &self.value {
			Value::List(b) => &b,
			_ => &LIST_REF,
		}
	}
	pub fn to_map(&self) -> &HashMap<String, RpcValue> {
		match &self.value {
			Value::Map(b) => &b,
			_ => &MAP_REF,
		}
	}
	pub fn to_imap(&self) -> &HashMap<i32, RpcValue> {
		match &self.value {
			Value::IMap(b) => &b,
			_ => &IMAP_REF,
		}
	}
}

impl fmt::Debug for RpcValue {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "RpcValue {{meta: {:?} value: {:?}}}", self.meta, self.value)
	}
}

#[cfg(test)]
mod test {
	use crate::metamap::MetaMap;
	use crate::rpcvalue::{RpcValue, Value};
	use std::collections::HashMap;
	use std::mem::size_of;
	use crate::datetime::DateTime;
	use chrono::Offset;

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
		assert_eq!(rv.to_datetime(), &dt);

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

