use crate::{MetaMap, RpcValue, Decimal, DateTime};
use crate::rpcvalue::Value;
use std::collections::HashMap;

pub mod writer;
pub mod reader;

enum PackingSchema {
    Null = 128,
    UInt,
    Int,
    Double,
    Bool,
    Blob_depr, // deprecated
    String,
    DateTimeEpoch_depr, // deprecated
    List,
    Map,
    IMap,
    MetaMap,
    Decimal,
    DateTime,
    CString,
    FALSE = 253,
    TRUE = 254,
    TERM = 255,
}

pub type WriterResult = std::io::Result<usize>;

pub trait CPWriter {
    fn write_meta(&mut self, map: &MetaMap) -> WriterResult;
    fn write(&mut self, val: &RpcValue) -> WriterResult;
    fn write_value(&mut self, val: &Value) -> WriterResult;
    fn write_byte(&mut self, b: u8) -> WriterResult;
    fn write_bytes(&mut self, arr: &[u8]) -> WriterResult;
    fn write_blob(&mut self, arr: &[u8]) -> WriterResult;
    fn write_int(&mut self, n: i64) -> WriterResult;
    fn write_uint(&mut self, n: u64) -> WriterResult;
    fn write_double(&mut self, n: f64) -> WriterResult;
    fn write_decimal(&mut self, decimal: &Decimal) -> WriterResult;
    fn write_datetime(&mut self, dt: &DateTime) -> WriterResult;
    fn write_list(&mut self, lst: &Vec<RpcValue>) -> WriterResult;
    fn write_map(&mut self, map: &HashMap<String, RpcValue>) -> WriterResult;
    fn write_imap(&mut self, map: &HashMap<i32, RpcValue>) -> WriterResult;
}