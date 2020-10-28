use std::io::Read;
use crate::{MetaMap, RpcValue};
use crate::rpcvalue::Value;

#[derive(Debug)]
pub struct ReadError {
    msg: String,
    line: usize,
    col: usize,
}
impl ReadError {
    fn new(msg: String, line: usize, col: usize) -> ReadError {
        ReadError { msg, line, col }
    }
}

pub struct ByteReader<'a, R>
{
    read: &'a mut R,
    peeked: Option<u8> ,
    line: usize,
    col: usize,
}

impl<'a, R> ByteReader<'a, R>
where R: Read
{
    pub(crate) fn new(read: &'a mut R) -> ByteReader<'a, R> {
        ByteReader {
            read,
            peeked: None,
            line: 0,
            col: 0,
        }
    }

    pub(crate) fn peek_byte(&mut self) -> u8 {
        if let Some(b) = self.peeked {
            return b
        }
        let mut arr: [u8; 1] = [0];
        let r = self.read.read(&mut arr);
        match r {
            Ok(n) => {
                if n == 0 {
                    return 0
                }
                self.peeked = Some(arr[0]);
                arr[0]
            }
            _ => 0
        }
    }
    pub(crate) fn get_byte(&mut self) -> Result<u8, ReadError> {
        let ret_b;
        if let Some(b) = self.peeked {
            self.peeked = None;
            ret_b = b;
        } else {
            let mut arr: [u8; 1] = [0];
            let r = self.read.read(&mut arr);
            match r {
                Ok(n) => {
                    if n == 0 {
                        return Err(self.make_error("Unexpected end of stream."))
                    }
                    ret_b = arr[0];
                }
                Err(e) => return Err(self.make_error(&e.to_string()))
            }
        }
        if ret_b == b'\n' {
            self.line += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }
        Ok(ret_b)
    }

    pub(crate) fn make_error(&self, msg: &str) -> ReadError {
        ReadError { msg: msg.to_string(), line: self.line, col: self.col }
    }
}

pub(crate) type ReadResult = Result<RpcValue, ReadError>;
pub(crate) type ReadValueResult = Result<Value, ReadError>;

pub trait Reader {
    fn read(&mut self) -> ReadResult;
    fn read_meta(&mut self) -> Result<MetaMap, ReadError>;
    fn read_value(&mut self) -> ReadValueResult;
}
