use crate::rpcvalue::RpcValue;
use std::fmt;
use std::ops::Index;

#[derive(Debug, Clone, PartialEq)]
pub enum MetaKey {
    Int(i32),
    String(String),
}

pub enum MetaKeyRef<'a> {
    Int(i32),
    Str(&'a str),
}
impl<'a> MetaKeyRef<'a> {
    fn to_metakey(&'a self) -> MetaKey {
        match self {
            MetaKeyRef::Int(val) => MetaKey::Int(*val),
            MetaKeyRef::Str(val) => MetaKey::String(val.to_string()),
        }
    }
}

pub trait IntoMetaKeyRef {
    fn to_metakeyref(&self) -> MetaKeyRef;
}
impl IntoMetaKeyRef for &str {
    fn to_metakeyref(&self) -> MetaKeyRef {
        MetaKeyRef::Str(self)
    }
}
impl IntoMetaKeyRef for i32 {
    fn to_metakeyref(&self) -> MetaKeyRef {
        MetaKeyRef::Int(*self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MetaKeyVal {
    pub(crate) key: MetaKey,
    pub(crate) value: RpcValue
}

#[derive(PartialEq, Clone)]
pub struct MetaMap(pub(crate) Vec<MetaKeyVal>);

impl MetaMap {

    pub fn new() -> MetaMap {
        MetaMap(Vec::new())
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn insert<I>(&mut self, key: I, value: RpcValue) -> &mut Self
        where I: IntoMetaKeyRef {
        let ix = self.find(&key);
        match ix {
            None => self.0.push(MetaKeyVal{key: key.to_metakeyref().to_metakey(), value }),
            Some(ix) => self.0[ix].value = value,
        }
        self
    }

    fn find<I>(&self, key: &I) -> Option<usize>
        where I: IntoMetaKeyRef
    {
        let mut ix = 0;
        for kv in self.0.iter() {
            let mk = key.to_metakeyref();
            match &kv.key {
                MetaKey::String(k1) => {
                    match &mk {
                        MetaKeyRef::Str(k2) => if k1 == k2 {return Some(ix)}
                        _ 		=> (),
                    }
                },
                MetaKey::Int(k1) => {
                    match &mk {
                        MetaKeyRef::Int(k2) => if k1 == k2 {return Some(ix)}
                        _ 		=> (),
                    }
                },
            }
            ix = ix + 1;
        }
        None
    }

    // pub fn value<'a, I>(&'a self, key: I, def_val: Option<&'a RpcValue>) -> Option<&'a RpcValue>
    //     where I: IntoMetaKeyRef {
    //     let ix = self.find(&key);
    //     match ix {
    //         Some(ix) => Some(&self.0[ix].value),
    //         None => def_val
    //     }
    // }
}

impl fmt::Debug for MetaMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Index<&str> for MetaMap {
    type Output = RpcValue;

    fn index(&self, key: &str) -> &'_ Self::Output {
        let ix = self.find(&key);
        match ix {
            Some(ix) => &self.0[ix].value,
            None => panic!("Invalid MetaMap key '{}'", key),
        }
    }
}

impl Index<i32> for MetaMap {
    type Output = RpcValue;

    fn index(&self, key: i32) -> &'_ Self::Output {
        let ix = self.find(&key);
        match ix {
            Some(ix) => &self.0[ix].value,
            None => panic!("Invalid MetaMap key '{}'", key),
        }
    }
}

pub trait MetaMapValue<'a, Idx: ?Sized> {
    fn value(&'a self, ix: Idx) -> Option<&'a RpcValue>;
    fn value_or_default(&'a self, ix: Idx, def_val: &'a RpcValue) -> &'a RpcValue;
}
impl<'a> MetaMapValue<'a, i32> for MetaMap {
    fn value(&self, ix: i32) -> Option<&RpcValue> {
        match self.find(&ix) {
            Some(ix) => Some(&self.0[ix].value),
            None => None,
        }
    }
    fn value_or_default(&'a self, ix: i32, def_val: &'a RpcValue) -> &'a RpcValue {
        match self.find(&ix) {
            Some(ix) => &self.0[ix].value,
            None => def_val,
        }
    }
}


#[cfg(test)]
mod test {
    use crate::metamap::MetaMap;
    use crate::rpcvalue::RpcValue;
    use std::collections::BTreeMap;

    #[test]
    fn metamap_insert() {
        let mut mm = MetaMap::new();

        mm.insert(123, RpcValue::new(1.1));
        assert_eq!(mm[123].to_f64(), 1.1);
        // let vv = mm.value(1234, Some(&RpcValue::new(123)));
        // println!("inserted and retrieved: {:?}", vv);
        // assert_eq!(123, vv.unwrap().to_i32().unwrap());

        mm.insert("foo", RpcValue::new("bar")).insert(123, RpcValue::new("baz"));
        assert_eq!(mm["foo"].to_str(), "bar");
        // println!("val: {:?}", mm[123]);
        assert_eq!(mm[123].to_str(), "baz");

        let v1 = vec![RpcValue::new("foo"), RpcValue::new("bar"), RpcValue::new("baz")];
        let v2 = v1.clone();
        mm.insert("list", RpcValue::new(v1));
        assert_eq!(mm["list"].to_list(), &v2);

        let mut v1: BTreeMap<i32, RpcValue> = BTreeMap::new();
        v1.insert(1, RpcValue::new("foo"));
        v1.insert(2, RpcValue::new("bar"));
        v1.insert(3, RpcValue::new("baz"));
        let v2 = v1.clone();
        mm.insert("imap", RpcValue::new(v1));
        assert_eq!(mm["imap"].to_imap(), &v2);

        let mut v1: BTreeMap<String, RpcValue> = BTreeMap::new();
        v1.insert("a".to_string(), RpcValue::new("foo"));
        v1.insert("b".to_string(), RpcValue::new("bar"));
        v1.insert("c".to_string(), RpcValue::new("baz"));
        let v2 = v1.clone();
        mm.insert("map", RpcValue::new(v1));
        assert_eq!(mm["map"].to_map(), &v2);

    }
    /*
    #[test]
    #[should_panic]
    fn metamap_invalid_string_key() {
        let mm = MetaMap::new();
        let _a = &mm["abcd"];
    }

    #[test]
    #[should_panic]
    fn metamap_invalid_int_key() {
        let mut mm = MetaMap::new();
        mm.insert(123, RpcValue::new(()));
        let _a = &mm[1234];
    }
    */
}
