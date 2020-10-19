use crate::{RpcValue, MetaMap, metamap::MetaKey};
use std::io;
use io::Read;
use crate::decimal::Decimal;

#[derive(Debug)]
pub struct ReaderError {
    msg: String,
    line: usize,
    col: usize,
}
impl ReaderError {
    fn new(msg: String, line: usize, col: usize) -> ReaderError {
        ReaderError { msg, line, col }
    }
}

pub type ReaderResult = Result<RpcValue, ReaderError>;

pub struct Reader<'a, R>
{
    reader: &'a mut R,
    peeked: Option<u8> ,
    line: usize,
    col: usize,
}

pub fn from_cpon(cpon: &[u8]) -> ReaderResult
{
    Ok(RpcValue::new(()))
}

impl<'a, R> Reader<'a, R>
    where R: Read
{
    pub fn new(reader: &'a mut R) -> Reader<'a, R> {
        const EMPTY: &'static str = "";
        Reader {
            reader,
            peeked: None,
            line: 0,
            col: 0,
        }
    }

    fn new_error(&self, msg: &str) -> ReaderError {
        ReaderError { msg: msg.to_string(), line: self.line, col: self.col }
    }

    pub fn read(&mut self) -> ReaderResult
    {
        Ok(RpcValue::new(()))
    }
/*
    pub fn read_meta(&mut self) -> Result<MetaMap, ReaderError>
    {

    }
    fn read_value(&mut self) -> ReaderResult
    {

    }

 */
    fn get_byte(&mut self) -> Result<u8, ReaderError>
    {
        if let Some(b) = self.peeked {
            self.peeked = None;
            return Ok(b)
        }
        let mut arr: [u8; 1] = [0];
        let r = self.reader.read(&mut arr);
        match r {
            Ok(n) => {
                if n == 0 {
                    return Err(self.new_error("Unexpected end of stream."))
                }
                if arr[0] == b'\n' {
                    self.line += 1;
                    self.col = 0;
                }
                else {
                    self.col += 1;
                }
                return Ok(arr[0])
            }
            Err(e) => return Err(self.new_error(&e.to_string()))
        }
    }
    fn peek_byte(&mut self) -> u8
    {
        let mut arr: [u8; 1] = [0];
        let r = self.reader.read(&mut arr);
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
    fn read_number(&mut self) -> ReaderResult
    {
        let mut mantisa: i64 = 0;
        let mut exponent = 0;
        let mut decimals = 0;
        let mut dec_cnt: i64 = 0;
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
            return Err(self.new_error("Number should contain at least one digit."))
        }
        mantisa = n;
        let b = self.peek_byte();
        match b {
            b'u' => {
                is_uint = true;
                self.get_byte()?;
            }
            b'.' => {
                is_decimal = true;
                self.get_byte()?;
                let (n, digit_cnt) = self.read_int()?;
                decimals = n;
                dec_cnt = digit_cnt as i64;
            }
            b'e' | b'E' => {
                is_decimal = true;
                self.get_byte()?;
                let (n, digit_cnt) = self.read_int()?;
                exponent = n;
                if digit_cnt == 0 {
                    return Err(self.new_error("Malformed number exponetional part."))
                }
            }
            _ => (),
        }
        if is_decimal {
            for _i in 0 .. dec_cnt {
                mantisa *= 10;
            }
            mantisa += decimals;
            if is_neg {
                mantisa = -mantisa
            }
            return Ok(RpcValue::new(Decimal::new(mantisa, (exponent - dec_cnt) as i8)))
        }
        if is_uint {
            return Ok(RpcValue::new(mantisa as u64))
        }
        if is_neg { mantisa = -mantisa }
        return Ok(RpcValue::new(mantisa))
    }

    fn read_int(&mut self) -> Result<(i64, i32), ReaderError>
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

    fn skip_white_insignificant(&mut self) -> Result<(), ReaderError>
	{
		loop {
			let b = self.peek_byte();
			if b == 0 {
                break;
            }
			if b > b' ' {
                match b {
                    b'/' => {
                        let b = self.get_byte()?;
                        match b {
                            b'*' => {
                                // multiline_comment_entered
                                loop {
                                    let b = self.get_byte()?;
                                    if b == b'*' {
                                        let b = self.get_byte()?;
                                        if b == b'/' {
                                            break;
                                        }
                                    }
                                }
                            }
                            b'/' => {
                                // to end of line comment entered
                                loop {
                                    let b = self.get_byte()?;
                                    if b == b'\n' {
                                        break;
                                    }
                                }
                            }
                            _ => {
                                return Err(self.new_error("Malformed comment"))
                            }
                        }
                    }
                    b':' => {
                        self.get_byte()?;
                    }
                    b',' => {
                        self.get_byte()?;
                    }
                    _ => {
                        break;
                    }
                }
			}
			else {
                self.get_byte()?;
			}
		}
        return Ok(())
    }

    fn read_bytes_escaped(&mut self) -> ReaderResult
    {
        let mut bytes = Vec::new();
        self.get_byte()?; // eat '"'
        loop {
            let b = self.get_byte()?;
            if b == b'\\' {
                let b = self.get_byte()?;
                match b {
                    b'\\' => bytes.push(b'\\'),
                    b'"' => bytes.push(b'"'),
                    b'n' => bytes.push(b'\n'),
                    b'r' => bytes.push(b'\r'),
                    b't' => bytes.push(b'\t'),
                    b'0' => bytes.push(b'\0'),
                    _ => bytes.push(b),
                };
            } else {
                match b {
                    b'"' => {
                        // end of string
                        break;
                    }
                    _ => bytes.push(b),
                };
            }
        }
        self.get_byte()?; // eat '"'
        Ok(RpcValue::new(bytes))
    }
    fn read_list(&mut self) -> ReaderResult
    {
        let mut lst = Vec::new();
        self.get_byte()?; // eat '['
        loop {
            self.skip_white_insignificant()?;
            let b = self.peek_byte();
            if b == b']' {
                self.get_byte()?;
                break;
            }
            let val = self.read()?;
            lst.push(val);
        }
        Ok(RpcValue::new(lst))
    }
}
//
// #[cfg(test)]
// mod test
// {
//     use crate::cpon::reader::{Writer, to_cpon};
//     use crate::{MetaMap, RpcValue};
//
//     #[test]
//     fn read() {
//         let mut mm = MetaMap::new();
//
//         mm.insert(123, RpcValue::new(1.1));
//         mm.insert(42, RpcValue::new(1));
//         mm.insert("foo", RpcValue::new("bar")).insert(123, RpcValue::new("baz"));
//         let v1 = vec![RpcValue::new("foo"), RpcValue::new("bar"), RpcValue::new("baz")];
//         mm.insert("list", RpcValue::new(v1));
//
//         let mut buff = Vec::new();
//         let mut wr = Reader::new(&mut buff);
//         wr.indent = "  ";
//         let sz = wr.read_meta(&mm);
//         println!("size: {} cpon: {}", sz.unwrap(), std::str::from_utf8(&buff).unwrap());
//
//         let mut rv = RpcValue::new("test");
//         rv.set_meta(mm);
//         println!("cpon: {}", std::str::from_utf8(&to_cpon(&rv)).unwrap());
//     }
//
// }

