use crate::metamap::MetaMap;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
enum Value {
	Null,
	String(String),
	Int(i64),
	UInt(u64),
	Double(f64),
	Bool(bool),
	DateTime(DateTime),
	List(Vec<RpcValue>),
	Map(HashMap<String, RpcValue>),
	IMap(HashMap<i32, RpcValue>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DateTime {}

#[derive(PartialEq, Clone)]
pub struct RpcValue {
	meta: Box<MetaMap>,
	value: Value
}

impl RpcValue {
	pub fn new() -> RpcValue {
		RpcValue {
			meta: Box::new(MetaMap::new()),
			value: Value::Null
		}
	}
}

impl fmt::Debug for RpcValue {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "RpcValue {{meta: {:?} value: {:?}}}", self.meta, self.value)
	}
}


// see https://github.com/rhysd/tinyjson/blob/master/src/json_value.rs
