use crate::{RpcValue, rpctype, Value};
use crate::metamap::*;
use std::collections::BTreeMap;
use crate::rpcvalue::IMap;
use std::ops::{Deref, DerefMut};

pub enum Tag {
    RequestId = rpctype::Tag::USER as isize, // 8
    ShvPath, // 9
    Method,  // 10
    CallerIds, // 11
    ProtocolType, //needed when dest client is using different version than source one to translate raw message data to correct format
    RevCallerIds,
    AccessGrant,
    TunnelCtl,
    UserId,
    MAX
}

pub enum Key {Params = 1, Result, Error, ErrorCode, ErrorMessage, MAX }

pub struct RpcMessage (RpcValue);
impl RpcMessage {
    fn new(rv: RpcValue) -> Self {
        RpcMessage(rv)
    }
    fn new_request(method: &str) -> Self {
        let mut mm = MetaMap::new();
        mm.insert(rpctype::Tag::MetaTypeId as i32, RpcValue::new(rpctype::GlobalNS::MetaTypeID::ChainPackRpcMessage as i32));
        mm.insert(Tag::Method as i32, RpcValue::new(method));
        RpcMessage(RpcValue::new_with_meta(IMap::new(), Some(mm)))
    }

    fn is_request(&self) -> bool {
        if let Some(_) = self.request_id() {
            if let Some(_) = self.method() {
                return true;
            }
        }
        return false;
    }
    fn is_response(&self) -> bool {
        if let Some(_) = self.request_id() {
            if let None = self.method() {
                return true;
            }
        }
        return false;
    }
    fn is_signal(&self) -> bool {
        if let None = self.request_id() {
            if let Some(_) = self.method() {
                return true;
            }
        }
        return false;
    }
    fn request_id_mm(meta: &MetaMap) -> Option<i32> {
        match meta.value(Tag::RequestId as i32) {
            Some(v) => Some(v.to_i32()),
            None => None,
        }
    }
    fn request_id(&self) -> Option<i32> {
        return Self::request_id_mm(self.meta())
    }
    fn method_mm(meta: &MetaMap) -> Option<&str> {
        match meta.value(Tag::Method as i32) {
            Some(v) => Some(v.to_str()),
            None => None,
        }
    }
    fn method(&self) -> Option<&str> {
        return Self::method_mm(self.meta());
    }
}

impl Deref for RpcMessage {
    type Target = RpcValue;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for RpcMessage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub trait RpcRequest {
    fn params(&self) -> Option<&RpcValue>;
    fn set_params(&mut self, rv: RpcValue);
}

impl RpcRequest for RpcMessage {
    fn params(&self) -> Option<&RpcValue> {
        if let Value::IMap(m) = self.value() {
            let v = m.get(&(Key::Params as i32));
            return v;
        }
        None
    }
    fn set_params(&mut self, rv: RpcValue) {
        if let Value::IMap(m) = self.value_mut() {
            m.insert(Key::Params as i32, rv);
        } else {
            panic!("Not RpcRequest");
        }
    }
}

#[cfg(test)]
mod test {
    use crate::RpcValue;
    use crate::RpcRequest;
    use crate::RpcMessage;

    #[test]
    fn rpc_request() {
        let mut rq = RpcMessage::new_request("foo");
        rq.set_params(RpcValue::new(123));
        let cpon = rq.to_cpon();
        assert_eq!(cpon, "<1:1,10:\"foo\">i{1:123}");
    }
}