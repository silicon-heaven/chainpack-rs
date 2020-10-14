use crate::metamap::MetaMap;
use std::fmt;

// see https://github.com/rhysd/tinyjson/blob/master/src/json_value.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
	Null,
	String(String),
	Int(i64),
	UInt(u64),
	Double(f64),
	Bool(bool),
	// DateTime(DateTime),
	List(Vec<RpcValue>),
	// Map(HashMap<String, RpcValue>),
	// IMap(HashMap<i32, RpcValue>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DateTime {}

pub trait ToValue {
	fn to_value(self) -> Value;
}

impl ToValue for () { fn to_value(self) -> Value { Value::Null } }
impl ToValue for &str { fn to_value(self) -> Value { Value::String(String::from(self)) } }
impl ToValue for i32 { fn to_value(self) -> Value { Value::Int(self as i64) } }
impl ToValue for usize { fn to_value(self) -> Value { Value::UInt(self as u64) } }
impl ToValue for Vec<RpcValue> { fn to_value(self) -> Value { Value::List(self) } }

macro_rules! to_value {
    ($from:ty, $to:ident) => {
		impl ToValue for $from {
			fn to_value(self) -> Value {
				Value::$to(self)
			}
		}
    };
}

to_value!(i64, Int);
to_value!(u64, UInt);
to_value!(f64, Double);
to_value!(bool, Bool);

#[derive(PartialEq, Clone)]
pub struct RpcValue {
	meta: Box<MetaMap>,
	value: Value
}

impl RpcValue {
	pub fn new<I>(val: I) -> RpcValue
	where I: ToValue {
		RpcValue {
			meta: Box::new(MetaMap::new()),
			value: val.to_value(),
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
	use crate::rpcvalue::RpcValue;

	#[test]
	fn rpcval_new() {
		let _rv = RpcValue::new(true);
		let _rv = RpcValue::new("foo");
		let _rv = RpcValue::new(123);
		let _rv = RpcValue::new(12.3);
		let _rv = RpcValue::new(vec![RpcValue::new(123), RpcValue::new("foo")]);

	}


}

