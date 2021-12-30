use crate::{RpcValue, rpctype, Value};
use crate::metamap::*;
// use std::collections::BTreeMap;
use crate::rpcvalue::{IMap, List};
// use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicI64, Ordering};
use std::fmt;

static G_RPC_REQUEST_COUNT: AtomicI64 = AtomicI64::new(0);

pub type RqId = i64;
pub type CliId = i32;

#[allow(dead_code)]
pub enum Tag {
    RequestId = rpctype::Tag::USER as isize, // 8
    ShvPath, // 9
    Method,  // 10
    CallerIds, // 11
    ProtocolType, //needed when destination client is using different version than source one to translate raw message data to correct format
    RevCallerIds,
    AccessGrant,
    TunnelCtl,
    UserId,
    MAX
}

#[allow(dead_code)]
pub enum Key {Params = 1, Result, Error, ErrorCode, ErrorMessage, MAX }

//pub type RpcMessageResult = Result<RpcMessage, &str>;

#[derive(Clone, Debug)]
pub struct RpcMessage (RpcValue);
impl RpcMessage {
    pub fn default() -> Self {
        let mut mm = MetaMap::new();
        mm.insert(rpctype::Tag::MetaTypeId as i32, RpcValue::from(rpctype::GlobalNS::MetaTypeID::ChainPackRpcMessage as i32));
        //mm.insert(Tag::Method as i32, RpcValue::from(method));
        RpcMessage(RpcValue::new(IMap::new().into(),Some(mm)))
    }
    pub fn new(meta: MetaMap, value: Value) -> Result<Self, &'static str> {
        Self::from_rpcvalue(RpcValue::new(value, Some(meta)))
    }
    pub fn from_meta(meta: MetaMap) -> Self {
        RpcMessage(RpcValue::from(IMap::new()).set_meta(Some(meta)))
    }
    pub fn from_rpcvalue(rv: RpcValue) -> Result<Self, &'static str> {
        if let Some(id) = rv.meta().tag(rpctype::Tag::MetaTypeId as i32) {
            if id.as_int() == rpctype::GlobalNS::MetaTypeID::ChainPackRpcMessage as i64 {
                if rv.is_imap() {
                    return Ok(Self(rv))
                }
                return Err("Value must be IMap!");
            }
            return Err("Tag MetaTypeId is wrong!");
        }
        return Err("Tag MetaTypeId is missing!");
    }
    pub fn as_rpcvalue(&self) -> &RpcValue {
        return &self.0
    }
    pub fn to_cpon(&self) -> String {
        self.0.to_cpon()
    }

    pub fn next_request_id() -> RqId {
        let old_id = G_RPC_REQUEST_COUNT.fetch_add(1, Ordering::SeqCst);
        old_id + 1
    }

    pub fn params(&self) -> Option<&RpcValue> { self.key(Key::Params as i32) }
    pub fn set_params(&mut self, rv: RpcValue) -> &mut Self  { self.set_key(Key::Params, Some(rv));self }
    pub fn result(&self) -> Option<&RpcValue> { self.key(Key::Result as i32) }
    pub fn set_result(&mut self, rv: RpcValue) -> &mut Self { self.set_key(Key::Result, Some(rv));self }
    pub fn error(&self) -> Option<RpcError> {
        if let Some(rv) = self.key(Key::Error as i32) {
            return RpcError::from_rpcvalue(rv)
        }
        None
    }
    pub fn set_error(&mut self, err: RpcError) -> &mut Self {
        self.set_key(Key::Result, Some(err.to_rpcvalue()));
        self
    }
    pub fn is_success(&self) -> bool {
        match self.result() {
            Some(_) => true,
            None => false,
        }
    }

    fn tag<Idx>(&self, key: Idx) -> Option<&RpcValue>
        where Idx: GetIndex
    {
        self.0.meta().get(key)
    }
    fn set_tag<Idx>(&mut self, key: Idx, rv: Option<RpcValue>) -> &mut Self
        where Idx: GetIndex
    {
        if let Some(mm) = self.0.meta_mut(){
            match rv {
                Some(rv) => { mm.insert(key, rv); },
                None => { mm.remove(key); },
            };
            self
        } else {
            panic!("Not RpcMessage")
        }
    }
    fn key(&self, key: i32) -> Option<&RpcValue> {
        if let Value::IMap(m) = self.0.value() {
            let v = m.get(&key);
            return v;
        }
        None
    }
    fn set_key(&mut self, key: Key, rv: Option<RpcValue>) -> &mut Self {
        if let Value::IMap(m) = self.0.value_mut() {
            match rv {
                Some(rv) => m.insert(key as i32, rv),
                None => m.remove(&(key as i32)),
            };
            self
        } else {
            panic!("Not RpcMessage")
        }
    }
    pub fn create_request(shvpath: &str, method: &str, params: Option<RpcValue>) -> Self {
        Self::create_request_with_id(Self::next_request_id(), shvpath, method, params)
    }
    pub fn create_request_with_id(rq_id: RqId, shvpath: &str, method: &str, params: Option<RpcValue>) -> Self {
        let mut msg = Self::default();
        msg.set_request_id(rq_id);
        if !shvpath.is_empty() {
            msg.set_shvpath(shvpath);
        }
        msg.set_method(method);
        if let Some(rv) = params {
            msg.set_params(rv);
        }
        msg
    }
    pub fn prepare_response(&self) -> Result<Self, &'static str> {
        let meta = Self::prepare_response_meta(self.as_rpcvalue().meta())?;
        Ok(Self::from_meta(meta))
    }
    pub fn prepare_response_meta(src: &MetaMap) -> Result<MetaMap, &'static str> {
        if src.is_request() {
            if let Some(rqid) = src.request_id() {
                let mut dest = MetaMap::new();
                dest.insert(rpctype::Tag::MetaTypeId as i32, RpcValue::from(rpctype::GlobalNS::MetaTypeID::ChainPackRpcMessage as i32));
                dest.set_request_id(rqid);
                dest.set_caller_ids(&src.caller_ids());
                return Ok(dest)
            }
            return Err("Request ID is missing")
        }
        Err("Not RPC Request")
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
pub trait RpcMessageMetaTags {
    type Target;

    fn tag(&self, id: i32) -> Option<&RpcValue>;
    fn set_tag(&mut self, id: i32, val: Option<RpcValue>) -> &mut Self::Target;

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

    fn request_id(&self) -> Option<RqId> {
        let t = self.tag(Tag::RequestId as i32);
        match t {
            None => None,
            Some(rv) => Some(rv.as_i64()),
        }
    }
    // fn try_request_id(&self) -> Result<RqId, &'static str> {
    //     match self.request_id() {
    //         None => Err("Request id not exists."),
    //         Some(id) => Ok(id),
    //     }
    // }
    fn set_request_id(&mut self, id: RqId) -> &mut Self::Target {
        self.set_tag(Tag::RequestId as i32, Some(RpcValue::from(id)))
    }
    fn shv_path(&self) -> Option<&str> {
        let t = self.tag(Tag::ShvPath as i32);
        match t {
            None => None,
            Some(rv) => Some(rv.as_str()),
        }
    }
    fn set_shvpath(&mut self, shv_path: &str) -> &mut Self::Target {
        self.set_tag(Tag::ShvPath as i32, Some(RpcValue::from(shv_path)))
    }
    fn method(&self) -> Option<&str> {
        let t = self.tag(Tag::Method as i32);
        match t {
            None => None,
            Some(rv) => Some(rv.as_str()),
        }
    }
    fn set_method(&mut self, method: &str) -> &mut Self::Target {
        self.set_tag(Tag::Method as i32, Some(RpcValue::from(method)))
    }

    fn caller_ids(&self) -> Vec<CliId> {
        let t = self.tag(Tag::CallerIds as i32);
        match t {
            None => Vec::new(),
            Some(rv) => {
                if rv.is_int() {
                    return vec![rv.as_int() as CliId];
                }
                if rv.is_list() {
                    return rv.as_list().into_iter().map(|v| v.as_int() as CliId).collect();
                }
                return Vec::new()
            },
        }
    }

    fn set_caller_ids(&mut self, ids: &Vec<CliId>) -> &mut Self::Target {
        if ids.len() == 0 {
            return self.set_tag(Tag::CallerIds as i32, None);
        }
        if ids.len() == 1 {
            return self.set_tag(Tag::CallerIds as i32, Some(RpcValue::from(ids[0] as CliId)));
        }
        let lst: List = ids.into_iter().map(|v| RpcValue::from(*v)).collect();
        return self.set_tag(Tag::CallerIds as i32, Some(RpcValue::from(lst)));
    }

    fn push_caller_id(&mut self, id: CliId) -> &mut Self::Target {
        let mut ids = self.caller_ids();
        ids.push(id as CliId);
        self.set_caller_ids(&ids)
    }
    fn pop_caller_id(&mut self) -> Option<CliId> {
        let mut ids = self.caller_ids();
        let id = ids.pop();
        match id {
            Some(id) => {
                self.set_caller_ids(&ids);
                Some(id)
            }
            None => {
                None
            }
        }
    }
}
impl RpcMessageMetaTags for RpcMessage {
    type Target = RpcMessage;

    fn tag(&self, id: i32) -> Option<&RpcValue> {
        self.tag(id)
    }
    fn set_tag(&mut self, id: i32, val: Option<RpcValue>) -> &mut Self::Target {
        self.set_tag(id, val)
    }
}

impl RpcMessageMetaTags for MetaMap {
    type Target = MetaMap;

    fn tag(&self, id: i32) -> Option<&RpcValue> {
        self.get(id)
    }
    fn set_tag(&mut self, id: i32, val: Option<RpcValue>) -> &mut Self::Target {
        match val {
            Some(rv) => {
                self.insert(id, rv);
                self
            }
            None => {
                self.remove(id);
                self
            }
        }
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
pub struct RpcError(IMap);

impl RpcError {
    pub fn new(code: RpcErrorCode, msg: &str) -> Self {
        enum Key {KeyCode = 1, KeyMessage}
        let mut m = IMap::new();
        m.insert(Key::KeyCode as i32, RpcValue::from(code as i64));
        if msg.len() > 0 {
            m.insert(Key::KeyMessage as i32, RpcValue::from(msg.to_string()));
        }
        RpcError(m)
    }
    pub fn from_rpcvalue(rv: &RpcValue) -> Option<Self> {
        if rv.is_imap() {
            return Some(RpcError(rv.as_imap().clone()))
        }
        None
    }
    pub fn to_rpcvalue(&self) -> RpcValue {
        RpcValue::from(self.0.clone())
    }
}

impl fmt::Display for RpcMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_rpcvalue().to_cpon())
    }
}
/*
impl fmt::Debug for RpcMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_rpcvalue().to_cpon())
    }
}
*/
#[cfg(test)]
mod test {
    use crate::RpcValue;
    use crate::RpcMessage;
    use crate::rpcmessage::RpcMessageMetaTags;

    #[test]
    fn rpc_request() {
        let id = RpcMessage::next_request_id();
        let mut rq = RpcMessage::create_request_with_id(id, "foo/bar", "baz", None);
        let params = RpcValue::from(123);
        rq.set_params(params.clone());
        assert_eq!(rq.params(), Some(&params));
        let caller_ids = vec![1,2,3];
        rq.set_caller_ids(&caller_ids);
        assert_eq!(&rq.caller_ids(), &caller_ids);
        let id = rq.pop_caller_id();
        assert_eq!(id, Some(3));
        assert_eq!(rq.caller_ids(), vec![1,2]);
        let id = rq.pop_caller_id();
        assert_eq!(id, Some(2));
        let id = rq.pop_caller_id();
        assert_eq!(id, Some(1));
        let id = rq.pop_caller_id();
        assert_eq!(id, None);
        rq.push_caller_id(4);
        let mut resp = rq.prepare_response().unwrap();
        assert_eq!(&resp.caller_ids(), &vec![4]);
        assert_eq!(resp.pop_caller_id(), Some(4));
        //let cpon = rq.as_rpcvalue().to_cpon();
        //assert_eq!(cpon, format!("<1:1,8:{},10:\"foo\">i{{1:123}}", id + 1));
    }
}