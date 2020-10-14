use crate::rpcvalue::RpcValue;
use std::fmt;
use std::ops::Index;

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

trait IntoMetaKeyRef {
    fn to_metakeyref(&self) -> MetaKeyRd;
}
impl IntoMetaKeyRef for &str {
    fn to_metakeyref(&self) -> MetaKeyRd {
        MetaKeyRd::Str(self)
    }
}
impl IntoMetaKeyRef for i32 {
    fn to_metakeyref(&self) -> MetaKeyRd {
        MetaKeyRd::Int(*self)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct MetaKeyVal {
    key: MetaKey,
    value: RpcValue
}

#[derive(PartialEq, Clone)]
pub struct MetaMap(Vec<MetaKeyVal>);

impl MetaMap {

    pub fn new() -> MetaMap {
        MetaMap(Vec::new())
    }

    pub fn insert<I>(&mut self, key: I, value: &RpcValue)
        where I: IntoMetaKeyRef {
        self.0.push(MetaKeyVal{key: key.to_metakeyref().to_metakey(), value: value.clone()});
    }

    pub fn find<I>(&self, key: I) -> Option<&MetaKeyVal>
        where I: IntoMetaKeyRef {
        for kv in self.0.iter() {
            let mk = key.to_metakeyref();
            match &kv.key {
                MetaKey::String(k1) => {
                    match &mk {
                        MetaKeyRd::Str(k2) => if k1 == k2 {return Some(kv)}
                        _ 		=> (),
                    }
                },
                MetaKey::Int(k1) => {
                    match &mk {
                        MetaKeyRd::Int(k2) => if k1 == k2 {return Some(kv)}
                        _ 		=> (),
                    }
                },
            }
        }
        None
    }

    pub fn value<'a, I>(&'a self, key: I, def_val: Option<&'a RpcValue>) -> Option<&'a RpcValue>
        where I: IntoMetaKeyRef {
        let kv = self.find(key);
        match kv {
            Some(val) => Some(&val.value),
            None => def_val
        }
    }
}

impl fmt::Debug for MetaMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Index<&str> for MetaMap {
    type Output = RpcValue;

    fn index(&self, key: &str) -> &'_ Self::Output {
        let kv = self.find(key);
        match kv {
            Some(val) => &val.value,
            None => panic!("Invalid MetaMap key '{}'", key),
        }
    }
}

impl Index<i32> for MetaMap {
    type Output = RpcValue;

    fn index(&self, key: i32) -> &'_ Self::Output {
        let kv = self.find(key);
        match kv {
            Some(val) => &val.value,
            None => panic!("Invalid MetaMap key '{}'", key),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::metamap::MetaMap;
    use crate::rpcvalue::RpcValue;

    #[test]
    fn metamap_insert() {
        let mut mm = MetaMap::new();
        let rv = RpcValue::new();

        mm.insert(123, &rv);
        assert_eq!(mm[123], rv);
        let vv = mm.value(123, None);
        println!("inserted and retrieved: {:?}", vv);
        assert_eq!(vv, Some(&rv));

        mm.insert("abc", &rv);
        assert_eq!(mm["abc"], rv);
        let vv = mm.value("abc", None);
        println!("inserted and retrieved: {:?}", vv);
        assert_eq!(vv, Some(&rv));

        println!("metamap: {:?}", mm);
    }

    #[test]
    #[should_panic]
    fn metamap_invalid_string_key() {
        let mut mm = MetaMap::new();
        let _a = &mm["abcd"];
    }

    #[test]
    #[should_panic]
    fn metamap_invalid_int_key() {
        let mut mm = MetaMap::new();
        let rv = RpcValue::new();
        mm.insert(123, &rv);
        let _a = &mm[1234];
    }

}
