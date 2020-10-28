use std::io::Read;
use crate::reader::{ByteReader, Reader, ReadError, ReadResult};
use crate::{RpcValue, MetaMap, Value, Decimal};
use crate::rpcvalue::FromValue;
use std::collections::HashMap;

pub struct ChainPackReader<'a, R>
    where R: Read
{
    byte_reader: ByteReader<'a, R>,
}

impl<'a, R> ChainPackReader<'a, R>
    where R: Read
{
    pub fn new(read: &'a mut R) -> Self {
        ChainPackReader { byte_reader: ByteReader::new(read) }
    }

    fn peek_byte(&mut self) -> u8 {
        self.byte_reader.peek_byte()
    }
    fn get_byte(&mut self) -> Result<u8, ReadError> {
        self.byte_reader.get_byte()
    }
    fn make_error(&self, msg: &str) -> ReadError {
        self.byte_reader.make_error(msg)
    }

    fn read_string(&mut self) -> Result<Value, ReadError> {
        let mut buff: Vec<u8> = Vec::new();
        self.get_byte()?; // eat "
        loop {
            let b = self.get_byte()?;
            match &b {
                b'\\' => {
                    let b = self.get_byte()?;
                    match &b {
                        b'\\' => buff.push(b'\\'),
                        b'"' => buff.push(b'"'),
                        b'n' => buff.push(b'\n'),
                        b'r' => buff.push(b'\r'),
                        b't' => buff.push(b'\t'),
                        b'0' => buff.push(b'\0'),
                        _ => buff.push(b),
                    }
                }
                b'"' => {
                    // end of string
                    break;
                }
                _ => {
                    buff.push(b);
                }
            }
        }
        let s = std::str::from_utf8(&buff);
        match s {
            Ok(s) => return Ok(s.make_value()),
            Err(e) => return Err(self.make_error(&format!("Invalid Map key, Utf8 error: {}", e))),
        }
    }
    fn read_int(&mut self) -> Result<(i64, i32), ReadError>
    {
        let mut base = 10;
        let mut val: i64 = 0;
        let mut neg = false;
        let mut n = 0;
        let mut digit_cnt = 0;
        loop {
            let b = self.peek_byte();
            match b {
                0 => break,
                b'+' | b'-' => {
                    if n != 0 {
                        break;
                    }
                    let b = self.get_byte()?;
                    if b == b'-' {
                        neg = true;
                    }
                }
                b'x' => {
                    if n == 1 && val != 0 {
                        break;
                    }
                    if n != 1 {
                        break;
                    }
                    self.get_byte()?;
                    base = 16;
                }
                b'0' ..= b'9' => {
                    self.get_byte()?;
                    val *= base;
                    val += (b as i64) - 48;
                    digit_cnt += 1;
                }
                b'A' ..= b'F' => {
                    if base != 16 {
                        break;
                    }
                    self.get_byte()?;
                    val *= base;
                    val += (b as i64) - 65 + 10;
                    digit_cnt += 1;
                }
                b'a' ..= b'f' => {
                    if base != 16 {
                        break;
                    }
                    self.get_byte()?;
                    val *= base;
                    val += (b as i64) - 97 + 10;
                    digit_cnt += 1;
                }
                _ => break,
            }
            n += 1;
        }
        if neg {
            val = -val;
        }
        Ok((val, digit_cnt))
    }
    fn read_number(&mut self) -> Result<Value, ReadError>
    {
        let mut mantisa = 0;
        let mut exponent = 0;
        let mut decimals = 0;
        let mut dec_cnt = 0;
        let mut is_decimal = false;
        let mut is_uint = false;
        let mut is_neg = false;

        let b = self.peek_byte();
        if b == b'+' {
            is_neg = false;
            self.get_byte()?;
        }
        else if b == b'-' {
            is_neg = true;
            self.get_byte()?;
        }

        let (n, digit_cnt) = self.read_int()?;
        if digit_cnt == 0 {
            return Err(self.make_error("Number should contain at least one digit."))
        }
        mantisa = n;
        #[derive(PartialEq)]
        enum State { Mantisa, Decimals, Exponent };
        let mut state = State::Mantisa;
        loop {
            let b = self.peek_byte();
            match b {
                b'u' => {
                    is_uint = true;
                    self.get_byte()?;
                    break;
                }
                b'.' => {
                    if state != State::Mantisa {
                        return Err(self.make_error("Unexpected decimal point."))
                    }
                    state = State::Decimals;
                    is_decimal = true;
                    self.get_byte()?;
                    let (n, digit_cnt) = self.read_int()?;
                    decimals = n;
                    dec_cnt = digit_cnt as i64;
                }
                b'e' | b'E' => {
                    if state != State::Mantisa && state != State::Decimals {
                        return Err(self.make_error("Unexpected exponet mark."))
                    }
                    //state = State::Exponent;
                    is_decimal = true;
                    self.get_byte()?;
                    let (n, digit_cnt) = self.read_int()?;
                    exponent = n;
                    if digit_cnt == 0 {
                        return Err(self.make_error("Malformed number exponetional part."))
                    }
                    break;
                }
                _ => { break; }
            }
        }
        if is_decimal {
            for _i in 0 .. dec_cnt {
                mantisa *= 10;
            }
            mantisa += decimals;
            if is_neg {
                mantisa = -mantisa
            }
            return Ok(Decimal::new(mantisa, (exponent - dec_cnt) as i8).make_value())
        }
        if is_uint {
            return Ok((mantisa as u64).make_value())
        }
        if is_neg { mantisa = -mantisa }
        return Ok(mantisa.make_value())
    }
    fn read_list(&mut self) -> Result<Value, ReadError>
    {
        let mut lst = Vec::new();
        self.get_byte()?; // eat '['
        loop {
            let b = self.peek_byte();
            if b == b']' {
                self.get_byte()?;
                break;
            }
            let val = self.read()?;
            lst.push(val);
        }
        return Ok(lst.make_value())
    }

    fn read_map(&mut self) -> Result<Value, ReadError> {
        let mut map: HashMap<String, RpcValue> = HashMap::new();
        self.get_byte()?; // eat '{'
        loop {
            let b = self.peek_byte();
            if b == b'}' {
                self.get_byte()?;
                break;
            }
            let key = self.read_string();
            let skey = match &key {
                Ok(b) => {
                    match b {
                        Value::String(s) => s,
                        _ => return Err(self.make_error("Read MetaMap key internal error")),
                    }
                },
                _ => return Err(self.make_error(&format!("Invalid Map key '{}'", b))),
            };
            let val = self.read()?;
            map.insert(*skey.clone(), val);
        }
        return Ok(map.make_value())
    }
    fn read_imap(&mut self) -> Result<Value, ReadError> {
        self.get_byte()?; // eat 'i'
        let b = self.get_byte()?; // eat '{'
        if b != b'{' {
            return Err(self.make_error("Wrong IMap prefix, '{' expected."))
        }
        let mut map: HashMap<i32, RpcValue> = HashMap::new();
        loop {
            let b = self.peek_byte();
            if b == b'}' {
                self.get_byte()?;
                break;
            }
            let key = self.read_int()?.0;
            let val = self.read()?;
            map.insert(key as i32, val);
        }
        return Ok(map.make_value())
    }
    fn read_datetime(&mut self) -> Result<Value, ReadError> {
        unimplemented!()
    }
    fn read_true(&mut self) -> Result<Value, ReadError> {
        self.read_token("true")?;
        return Ok(true.make_value())
    }
    fn read_false(&mut self) -> Result<Value, ReadError> {
        self.read_token("false")?;
        return Ok(false.make_value())
    }
    fn read_null(&mut self) -> Result<Value, ReadError> {
        self.read_token("null")?;
        return Ok(().make_value())
    }
    fn read_token(&mut self, token: &str) -> Result<(), ReadError> {
        for c in token.as_bytes() {
            let b = self.get_byte()?;
            if b != *c {
                return Err(self.make_error(&format!("Incomplete '{}' literal.", token)))
            }
        }
        return Ok(())
    }

}

impl<'a, R> Reader for ChainPackReader<'a, R>
    where R: Read
{
    fn read(&mut self) -> ReadResult {
        let mut b = self.peek_byte();
        let mut mm: Option<MetaMap> = None;
        if b == b'<' {
            mm = Some(self.read_meta()?);
            b = self.peek_byte();
        }
        let v = match &b {
            b'0' ..= b'9' | b'+' | b'-' => self.read_number(),
            b'"' => self.read_string(),
            b'[' => self.read_list(),
            b'{' => self.read_map(),
            b'i' => self.read_imap(),
            b'd' => self.read_datetime(),
            b't' => self.read_true(),
            b'f' => self.read_false(),
            b'n' => self.read_null(),
            _ => Err(self.make_error(&format!("Invalid char {}", b))),
        }?;
        let mut rv = RpcValue::new(v);
        if let Some(m) = mm {
            rv.set_meta(m);
        }
        return Ok(rv)
    }
    fn read_meta(&mut self) -> Result<MetaMap, ReadError> {
        self.get_byte()?; // eat '<'
        let mut map = MetaMap::new();
        loop {
            let b = self.peek_byte();
            if b == b'>' {
                self.get_byte()?;
                break;
            }
            let key = self.read()?;
            let val = self.read()?;
            if key.is_int() {
                map.insert(key.to_i32(), val);
            }
            else {
                map.insert(key.to_str(), val);
            }
        }
        Ok(map)
    }
    fn read_value(&mut self) -> Result<Value, ReadError> {
        let b = self.peek_byte();
        let v = match &b {
            b'0' ..= b'9' | b'+' | b'-' => self.read_number(),
            b'"' => self.read_string(),
            b'[' => self.read_list(),
            b'{' => self.read_map(),
            b'i' => self.read_imap(),
            b'd' => self.read_datetime(),
            b't' => self.read_true(),
            b'f' => self.read_false(),
            b'n' => self.read_null(),
            _ => Err(self.make_error(&format!("Invalid char {}", b))),
        }?;
        Ok(v)
    }
}

