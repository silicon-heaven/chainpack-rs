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
enum MetaKey {
	Int(i32),
	String(String),
}

enum MetaKeyRd<'a> {
	Int(i32),
	Str(&'a str),
}
impl<'a> MetaKeyRd<'a> {
	fn to_metakey(&'a self) -> MetaKey {
		match self {
			MetaKeyRd::Int(val) => MetaKey::Int(*val),
			MetaKeyRd::Str(val) => MetaKey::String(val.to_string()),
		}
	}
}

trait IntoMetaKeyRd {
	fn into_metakeyrd(&self) -> MetaKeyRd;
}
impl IntoMetaKeyRd for &str {
   fn into_metakeyrd(&self) -> MetaKeyRd {
	   MetaKeyRd::Str(self)
   }
}
impl IntoMetaKeyRd for i32 {
   fn into_metakeyrd(&self) -> MetaKeyRd {
	   MetaKeyRd::Int(*self)
   }
}

#[derive(Debug, Clone, PartialEq)]
struct MetaKeyVal {
	key: MetaKey,
	value: RpcValue
}

#[derive(PartialEq, Clone)]
struct MetaMap {
	items: Vec<MetaKeyVal>
}

impl MetaMap {

	fn new() -> MetaMap {
		MetaMap {
			items: Vec::new()
		}
	}

	pub fn insert<I>(&mut self, key: I, value: &RpcValue)
	where I: IntoMetaKeyRd {
		self.items.push(MetaKeyVal{key: key.into_metakeyrd().to_metakey(), value: value.clone()});
	}
	pub fn value<'a, I>(&'a self, key: I, def_val: Option<&'a RpcValue>) -> Option<&'a RpcValue>
	where I: IntoMetaKeyRd {
		for kv in self.items.iter() {
			let mk = key.into_metakeyrd();
			match &kv.key {
				MetaKey::String(k1) => {
					match &mk {
						MetaKeyRd::Str(k2) => if k1 == k2 {return Some(&kv.value)}
				_ 		=> (),
					}
				},
				MetaKey::Int(k1) => {
					match &mk {
						MetaKeyRd::Int(k2) => if k1 == k2 {return Some(&kv.value)}
				_ 		=> (),
					}
				},
			}
		}
		def_val
	}
	/*
	fn svalue<'a>(&'a self, key: &str, def_val: Option<&'a RpcValue>) -> Option<&'a RpcValue> {
		for kv in self.items.iter() {
			match &kv.key {
				MetaKey::String(ref k1) => if k1 == key { return Some(&kv.value) },
				_ => (),
			}
		}
		def_val
	}
	fn ivalue<'a>(&'a self, key: i32, def_val: Option<&'a RpcValue>) -> Option<&'a RpcValue> {
		for kv in self.items.iter() {
			match &kv.key {
				MetaKey::Int(k1) => if *k1 == key { return Some(&kv.value) },
				_ => (),
			}
		}
		def_val
	}
	*/
}

impl fmt::Debug for MetaMap {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self.items)
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct DateTime {}

#[derive(PartialEq, Clone)]
pub struct RpcValue {
	meta: MetaMap,
	value: Value
}

impl RpcValue {
	fn new() -> RpcValue {
		RpcValue {
			meta: MetaMap::new(),
			value: Value::Null
		}
	}
}

impl fmt::Debug for RpcValue {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "RpcValue {{meta: {:?} value: {:?}}}", self.meta, self.value)
	}
}

#[cfg(test)]
#[test]
fn metamap_insert() {
	let mut mm = MetaMap::new();
	let rv = RpcValue::new();

	mm.insert(123, &rv);
	let vv = mm.value(123, None);
	println!("inserted and retrieved: {:?}", vv);
	assert_eq!(vv, Some(&rv));

	mm.insert("abc", &rv);
	let vv = mm.value("abc", None);
	println!("inserted and retrieved: {:?}", vv);
	assert_eq!(vv, Some(&rv));

	println!("metamap: {:?}", mm);
}
