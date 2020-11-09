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
    fn new() -> Self {
        let mut mm = MetaMap::new();
        mm.insert(rpctype::Tag::MetaTypeId as i32, RpcValue::new(rpctype::GlobalNS::MetaTypeID::ChainPackRpcMessage as i32));
        //mm.insert(Tag::Method as i32, RpcValue::new(method));
        RpcMessage(RpcValue::new_with_meta(IMap::new(), Some(mm)))
    }
    fn from_rpcvalue(rv: RpcValue) -> Self {
        RpcMessage(rv)
    }

    fn as_rpcvalue(&self) -> &RpcValue {
        return &self.0
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
    fn request_id_mm(meta: &MetaMap) -> Option<i64> {
        match Self::tag_mm(meta, Tag::RequestId as i32) {
            None => None,
            Some(rv) => Some(rv.to_i64()),
        }
    }
    fn request_id(&self) -> Option<i64> {
        return Self::request_id_mm(self.0.meta())
    }
    fn set_request_id_mm(meta: &mut MetaMap, id: i64) -> &mut MetaMap {
        Self::set_tag_mm(meta, Tag::RequestId as i32, Some(RpcValue::new(id)));
        meta
    }
    fn set_request_id(&mut self, id: i64) -> &mut Self {
        Self::set_request_id_mm(self.0.meta_mut().unwrap(), id);
        self
    }

    fn tag_mm<Idx>(meta: &MetaMap, key: Idx) -> Option<&RpcValue>
        where Idx: IntoMetaKeyRef
    {
        meta.value(key)
    }
    fn set_tag_mm<Idx>(meta: &mut MetaMap, key: Idx, rv: Option<RpcValue>) -> &mut MetaMap
        where Idx: IntoMetaKeyRef
    {
        match rv {
            Some(rv) => { meta.insert(key, rv); },
            None => { meta.remove(key); },
        };
        meta
    }
    fn tag<Idx>(&self, key: Idx) -> Option<&RpcValue>
        where Idx: IntoMetaKeyRef
    {
        Self::tag_mm(self.0.meta(), key)
    }
    fn set_tag<Idx>(&mut self, key: Idx, rv: Option<RpcValue>) -> &mut Self
        where Idx: IntoMetaKeyRef
    {
        if let Some(mm) = self.0.meta_mut(){
            Self::set_tag_mm(mm, key, rv);
            self
        } else {
            panic!("Not RpcMessage");
        }
    }
    fn key(&self, key: i32) -> Option<&RpcValue> {
        if let Value::IMap(m) = self.0.value() {
            let v = m.get(&key);
            return v;
        }
        None
    }
    fn set_key(&mut self, key: i32, rv: Option<RpcValue>) -> &mut Self {
        if let Value::IMap(m) = self.0.value_mut() {
            match rv {
                Some(rv) => m.insert(key, rv),
                None => m.remove(&key),
            };
            self
        } else {
            panic!("Not RpcMessage");
        }
    }

}
/*
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
*/
pub trait RpcRequest<'a> {
    fn new(method: &str, params: Option<RpcValue>) -> RpcMessage;
    fn method_mm(meta: & MetaMap) -> Option<&RpcValue>;
    fn set_method_mm(meta: &'a mut MetaMap, method: &str) -> &'a mut MetaMap;
    fn method(&self) -> Option<&RpcValue>;
    fn set_method(&mut self, method: &str) -> &mut RpcMessage;
    fn params(&self) -> Option<&RpcValue>;
    fn set_params(&mut self, rv: RpcValue) -> &mut RpcMessage;
}
impl<'a> RpcRequest for RpcMessage {
    fn new(method: &str, params: Option<RpcValue>) -> Self {
        let mut msg = Self::new();
        msg.set_method(method);
        if let Some(rv) = params {
             msg.set_params(rv);
        }
        msg
    }
    fn method_mm(meta: &MetaMap) -> Option<&str> {
        match Self::tag_mm(meta, Tag::Method as i32) {
            None => None,
            Some(rv) => rv.to_str()
        }
    }
    fn set_method_mm(meta: &mut MetaMap, method: &str) -> &mut MetaMap {
        Self::set_tag_mm(meta, Tag::Method as i32, Some(RpcValue::new(method)));
        meta
    }
    fn method(&self) -> Option<&str> {
        return Self::method_mm(self.0.meta());
    }
    fn set_method(&mut self, method: &str) -> &mut Self {
        return Self::set_method_mm(self.0.meta_mut().unwrap(), method);
        self
    }

    fn params(&self) -> Option<&RpcValue> {
        self.key(Key::Params as i32)
    }
    fn set_params(&mut self, rv: RpcValue) {
        self.set_key(Key::Params as i32, rv)
    }
}

pub enum RpcErrorCode {
    NoError = 0,
    InvalidRequest,	// The JSON sent is not a valid Request object.
    MethodNotFound,	// The method does not exist / is not available.
    InvalidParams,		// Invalid method parameter(s).
    InternalError,		// Internal JSON-RPC error.
    ParseError,		// Invalid JSON was received by the server. An error occurred on the server while parsing the JSON text.
    MethodCallTimeout,
    MethodCallCancelled,
    MethodCallException,
    Unknown,
    UserCode = 32
}
pub trait RpcError {
    fn new(code: RpcErrorCode, msg: &str) -> Self;
    //fn to_rpcvalue(&self) -> RpcValue;
}

impl RpcError for IMap {
    fn new(code: RpcErrorCode, msg: &str) -> Self {
        enum Key {KeyCode = 1, KeyMessage};
        let mut m = IMap::new();
        m.insert(Key::KeyCode as i32, RpcValue::new(code as i64));
        if msg.len() > 0 {
            m.insert(Key::KeyMessage as i32, RpcValue::new(msg.to_string()));
        }
        m
    }
}
pub trait RpcResponse {
    fn new(result: RpcValue) -> RpcMessage;
    fn new_error(err: impl RpcError) -> RpcMessage;
    fn result(&self) -> Option<&RpcValue>;
    fn set_result(&mut self, rv: RpcValue);
}
impl RpcResponse for RpcMessage {
    fn new(result: RpcValue) -> RpcMessage {
        let mut msg = RpcMessage::new();
        msg.set_result(result);
        msg
    }
    fn new_error(err: impl RpcError) -> RpcMessage {
        unimplemented!()
    }

    fn result(&self) -> Option<&RpcValue> {
        self.key(Key::Result as i32)
    }
    fn set_result(&mut self, rv: RpcValue) {
        self.set_key(Key::Result as i32, rv)
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
        let cpon = rq.as_rpcvalue().to_cpon();
        assert_eq!(cpon, "<1:1,10:\"foo\">i{1:123}");
    }
}