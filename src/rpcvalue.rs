use crate::metamap::MetaMap;
use std::fmt;
use std::collections::HashMap;
use lazy_static::lazy_static;

// see https://github.com/rhysd/tinyjson/blob/master/src/json_value.rs


const STR_REF: &str = "";
// const LIST_REF: Vec<RpcValue> = Vec::new();

//const MAP_REF: HashMap<String, RpcValue> = HashMap::new();
//const IMAP_REF: HashMap<i32, RpcValue> = HashMap::new();
lazy_static! {
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
	Null,
	Int(i64),
	UInt(u64),
	Double(f64),
	Bool(bool),
	DateTime(i64),
	Bytes(Box<Vec<u8>>),
	String(Box<String>),
	List(Box<Vec<RpcValue>>),
	Map(Box<HashMap<String, RpcValue>>),
	IMap(Box<HashMap<i32, RpcValue>>),
}


#[derive(Debug, Clone, PartialEq)]
pub struct DateTime {}

pub trait FromValue {
	fn from_value(self) -> Value;
}

impl FromValue for () { fn from_value(self) -> Value { Value::Null } }
impl FromValue for &str { fn from_value(self) -> Value { Value::String(Box::new(String::from(self))) } }
impl FromValue for i32 { fn from_value(self) -> Value { Value::Int(self as i64) } }
impl FromValue for usize { fn from_value(self) -> Value { Value::UInt(self as u64) } }

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

macro_rules! from_value_box {
    ($from:ty, $to:ident) => {
		impl FromValue for $from {
			fn from_value(self) -> Value {
				Value::$to(Box::new(self))
			}
		}
    };
}

from_value_box!(String, String);
from_value_box!(Vec<RpcValue>, List);
from_value_box!(HashMap<String, RpcValue>, Map);
from_value_box!(HashMap<i32, RpcValue>, IMap);

#[derive(PartialEq, Clone)]
pub struct RpcValue {
	meta: Box<MetaMap>,
	value: Value
}

impl RpcValue {
	pub fn new<I>(val: I) -> RpcValue
	where I: FromValue {
		RpcValue {
			meta: Box::new(MetaMap::new()),
			value: val.from_value(),
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
	pub fn to_str(&self) -> &str {
		match &self.value {
			Value::String(s) => &s,
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
		let _rv = RpcValue::new(true);
		let _rv = RpcValue::new("foo");
		let _rv = RpcValue::new("bar".to_string());
		let _rv = RpcValue::new(123);
		let _rv = RpcValue::new(12.3);
		let _rv = RpcValue::new(vec![RpcValue::new(123), RpcValue::new("foo")]);

		let mut m: HashMap<String, RpcValue> = HashMap::new();
		m.insert("foo".to_string(), RpcValue::new(123));
		m.insert("bar".to_string(), RpcValue::new("foo"));
		let _rv = RpcValue::new(m);

		let mut m: HashMap<i32, RpcValue> = HashMap::new();
		m.insert(1, RpcValue::new(123));
		m.insert(2, RpcValue::new("foo"));
		let _rv = RpcValue::new(m);
	}

}

