use crate::{RpcValue, rpctype};
use crate::metamap::*;

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
//
// pub enum Key {Params = 1, Result, Error, ErrorCode, ErrorMessage, MAX }
//
// pub struct MetaType {
//     ID: i32,
// }

// impl MetaType {
//     pub fn new() -> Self {
//         MetaType {
//             ID: rpctype.GlobalNS.MetaTypeID.ChainPackRpcMessage,
//         }
//     }
// }

pub struct RpcMessage (RpcValue);

impl RpcMessage {
    pub fn new(rv: RpcValue) -> Self {
        RpcMessage(rv)
    }
    pub fn is_request(&self) -> bool {
        if let Some(_) = self.request_id() {
            if let Some(_) = self.method() {
                return true;
            }
        }
        return false;
    }
    pub fn is_response(&self) -> bool {
        if let Some(_) = self.request_id() {
            if let None = self.method() {
                return true;
            }
        }
        return false;
    }
    pub fn is_signal(&self) -> bool {
        if let None = self.request_id() {
            if let Some(_) = self.method() {
                return true;
            }
        }
        return false;
    }

    // pub fn isRequest(const ref RpcValue.Meta meta) -> bool {
    // return hasRequestId(meta) && hasMethod(meta);
    // }
    // pub fn isResponse(const ref RpcValue.Meta meta) -> bool {
    // return hasRequestId(meta) && !hasMethod(meta);
    // }
    // pub fn isSignal(const ref RpcValue.Meta meta) -> bool {
    // return !hasRequestId(meta) && hasMethod(meta);
    // }
    //
    // void setRequestId(ref RpcValue.Meta meta, RpcValue request_id) {
    // meta[MetaType.Tag.RequestId] = request_id;
    // }
    // void setRequestId(RpcValue request_id) {
    // setRequestId(m_value.meta, request_id);
    // }
    // pub fn has_request_id_mm(mm: &MetaMap) -> bool {
    //     match meta.find(Tag::RequestId as i32) {
    //         Some(_) => true,
    //         None => false,
    //     }
    // }
    pub fn request_id_mm(meta: &MetaMap) -> Option<i32> {
        match meta.value(Tag::RequestId as i32) {
            Some(v) => Some(v.to_i32()),
            None => None,
        }
    }
    pub fn request_id(&self) -> Option<i32> {
        return Self::request_id_mm(self.0.meta())
    }
    pub fn method_mm(meta: &MetaMap) -> Option<&str> {
        match meta.value(Tag::Method as i32) {
            Some(v) => Some(v.to_str()),
            None => None,
        }
    }
    pub fn method(&self) -> Option<&str> {
        return Self::method_mm(self.0.meta());
    }
}