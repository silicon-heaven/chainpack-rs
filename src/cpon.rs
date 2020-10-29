use std::io::{Write, Read};
use crate::{RpcValue, MetaMap, Value, Decimal, DateTime, ReadResult};
use std::collections::HashMap;
use crate::writer::{WriteResult, Writer, ByteWriter};
use crate::metamap::MetaKey;
use crate::reader::{Reader, ByteReader, ReadError};
use crate::rpcvalue::FromValue;

pub struct CponWriter<'a, W>
    where W: Write
{
    byte_writer: ByteWriter<'a, W>,
    indent: String,
}

impl<'a, W> CponWriter<'a, W>
    where W: Write
{
    pub fn new(write: &'a mut W) -> Self {
        CponWriter {
            byte_writer: ByteWriter::new(write),
            indent: "".to_string()
        }
    }

    fn write_byte(&mut self, b: u8) -> WriteResult {
        self.byte_writer.write_byte(b)
    }
    fn write_bytes(&mut self, b: &[u8]) -> WriteResult {
        self.byte_writer.write_bytes(b)
    }

    fn write_int(&mut self, n: i64) -> WriteResult
    {
        let s = n.to_string();
        let cnt = self.write_bytes(s.as_bytes())?;
        Ok(cnt)
    }
    fn write_uint(&mut self, n: u64) -> WriteResult
    {
        let s = n.to_string();
        let cnt = self.write_bytes(s.as_bytes())?;
        Ok(cnt)
    }
    fn write_double(&mut self, n: f64) -> WriteResult
    {
        let s = n.to_string();
        let cnt = self.write_bytes(s.as_bytes())?;
        Ok(cnt)
    }
    fn write_string(&mut self, s: &str) -> WriteResult
    {
        let mut cnt: usize = 0;
        cnt += self.write_byte(b'"')?;
        let bytes = s.as_bytes();
        for b in bytes {
            match b {
                0 => {
                    cnt += self.write_byte(b'\\')?;
                    cnt += self.write_byte(b'0')?;
                }
                b'\\' => {
                    cnt += self.write_byte(b'\\')?;
                    cnt += self.write_byte(b'\\')?;
                }
                b'\t' => {
                    cnt += self.write_byte(b'\\')?;
                    cnt += self.write_byte(b't')?;
                }
                b'\r' => {
                    cnt += self.write_byte(b'\\')?;
                    cnt += self.write_byte(b'r')?;
                }
                b'\n' => {
                    cnt += self.write_byte(b'\\')?;
                    cnt += self.write_byte(b'n')?;
                }
                b'"' => {
                    cnt += self.write_byte(b'\\')?;
                    cnt += self.write_byte(b'"')?;
                }
                _ => {
                    cnt += self.write_byte(*b)?;
                }
            }
        }
        cnt += self.write_byte(b'"')?;
        Ok(cnt)
    }
    fn write_decimal(&mut self, decimal: &Decimal) -> WriteResult {
        let mut neg = false;
        let (mut mantisa, exponent) = decimal.decode();
        if mantisa < 0 {
            mantisa = -mantisa;
            neg = true;
        }
        //let buff: Vec<u8> = Vec::new();
        let mut s = mantisa.to_string();

        let n = s.len() as i8;
        let dec_places = -exponent as i8;
        if dec_places > 0 && dec_places < n {
            // insert decimal point
            let dot_ix = n - dec_places;
            s.insert(dot_ix as usize, '.');
        }
        else if dec_places > 0 && dec_places <= 3 {
            // prepend 0.00000..
            let extra_0_cnt = dec_places - n;
            s = "0.".to_string()
                + &*std::iter::repeat("0").take(extra_0_cnt as usize).collect::<String>()
                + &*s;
        }
        else if dec_places < 0 && n + exponent <= 9 {
            // append ..000000.
            s = s + &*std::iter::repeat("0").take(exponent as usize).collect::<String>();
            s.push('.');
        }
        else if dec_places == 0 {
            // just append decimal point
            s.push('.');
        }
        else {
            // exponential notation
            s.push('e');
            s += &*exponent.to_string();
        }
        if neg {
            s.insert(0, '-');
        }
        let cnt = self.write_bytes(s.as_bytes())?;
        return Ok(cnt)
    }
    fn write_datetime(&mut self, dt: &DateTime) -> WriteResult {
        let mut cnt = self.write_bytes("d\"".as_bytes())?;
        let dtf = dt.to_datetime();
        let mut s = format!("{}", dtf.format("%Y-%m-%dT%H:%M:%S"));
        let ms = dt.to_epoch_msec() % 1000;
        if ms > 0 {
            s.push_str(&format!(".{:03}", ms));
        }
        let mut offset = dt.utc_offset();
        if offset == 0 {
            s.push('Z');
        }
        else {
            if offset < 0 {
                s.push('-');
                offset = -offset;
            } else {
                s.push('+');
            }
            let offset_hr = offset / 60 / 60;
            let offset_min = offset / 60 % 60;
            s += &format!("{:02}", offset_hr);
            if offset_min > 0 {
                s += &format!("{:02}", offset_min);
            }
        }
        cnt += self.write_bytes(s.as_bytes())?;
        cnt += self.write_byte(b'"')?;
        return Ok(cnt)
    }
    fn write_list(&mut self, lst: &Vec<RpcValue>) -> WriteResult
    {
        let mut cnt = 0;
        cnt += self.write_byte(b'[')?;
        let mut n = 0;
        let it = lst.iter();
        for v in it {
            if n == 0 {
                n += 1;
            } else {
                cnt += self.write_byte(b',')?;
            }
            cnt += self.write(v)?;
        }
        cnt += self.write_byte(b']')?;
        Ok(cnt)
    }
    fn write_map(&mut self, map: &HashMap<String, RpcValue>) -> WriteResult
    {
        let mut cnt = 0;
        cnt += self.write_byte(b'{')?;
        let mut n = 0;
        for (k, v) in map {
            if n == 0 {
                n += 1;
            } else {
                cnt += self.write_byte(b',')?;
            }
            cnt += self.write_string(k)?;
            cnt += self.write_byte(b':')?;
            cnt += self.write(v)?;
        }
        cnt += self.write_byte(b'}')?;
        Ok(cnt)
    }
    fn write_imap(&mut self, map: &HashMap<i32, RpcValue>) -> WriteResult
    {
        let mut cnt = 0;
        cnt += self.write_byte(b'i')?;
        cnt += self.write_byte(b'{')?;
        let mut n = 0;
        for (k, v) in map {
            if n == 0 {
                n += 1;
            } else {
                cnt += self.write_byte(b',')?;
            }
            cnt += self.write_int(*k as i64)?;
            cnt += self.write_byte(b':')?;
            cnt += self.write(v)?;
        }
        cnt += self.write_byte(b'}')?;
        Ok(cnt)
    }
}

impl<'a, W> Writer for CponWriter<'a, W>
    where W: Write
{
    fn write(&mut self, val: &RpcValue) -> WriteResult
    {
        let cnt: usize = self.byte_writer.count();
        let mm = val.meta();
        if !mm.is_empty() {
            self.write_meta(mm)?;
        }
        self.write_value(val.value())?;
        Ok(self.byte_writer.count() - cnt)
    }
    fn write_meta(&mut self, map: &MetaMap) -> WriteResult
    {
        let cnt: usize = self.byte_writer.count();
        self.write_byte(b'<')?;
        let mut n = 0;
        for k in map.0.iter() {
            if n == 0 {
                n += 1;
            } else {
                self.write_byte(b',')?;
            }
            if !self.indent.is_empty() {
                self.write_byte(b'\n')?;
                let idn = self.indent.as_bytes();
                self.byte_writer.write_bytes(idn)?;
            }

            match &k.key {
                MetaKey::String(s) => {
                    self.write_string(s)?;
                },
                MetaKey::Int(i) => {
                    self.write_bytes(i.to_string().as_bytes())?;
                },
            }
            self.write_byte(b':')?;
            self.write(&k.value)?;
            n += 1;
        }
        if !self.indent.is_empty() {
            self.write_byte(b'\n')?;
        }
        self.write_byte(b'>')?;
        Ok(self.byte_writer.count() - cnt)
    }
    fn write_value(&mut self, val: &Value) -> WriteResult
    {
        let cnt: usize = self.byte_writer.count();
        match val {
            Value::Null => self.write_bytes("null".as_bytes()),
            Value::Bool(b) => if *b {
                self.write_bytes("true".as_bytes())
            } else {
                self.write_bytes("false".as_bytes())
            },
            Value::Int(n) => self.write_int(*n),
            Value::UInt(n) => {
                self.write_uint(*n);
                self.write_byte(b'u')
            },
            Value::String(s) => self.write_string(s),
            Value::Blob(b) => unimplemented!(),
            Value::Double(n) => self.write_double(*n),
            Value::Decimal(d) => self.write_decimal(d),
            Value::DateTime(d) => self.write_datetime(d),
            Value::List(lst) => self.write_list(lst),
            Value::Map(map) => self.write_map(map),
            Value::IMap(map) => self.write_imap(map),
        }?;
        Ok(self.byte_writer.count() - cnt)
    }
}

pub struct CponReader<'a, R>
    where R: Read
{
    byte_reader: ByteReader<'a, R>,
}

impl<'a, R> CponReader<'a, R>
    where R: Read
{
    pub fn new(read: &'a mut R) -> Self {
        CponReader { byte_reader: ByteReader::new(read) }
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

    fn skip_white_insignificant(&mut self) -> Result<(), ReadError> {
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
                                return Err(self.make_error("Malformed comment"))
                            }
                        }
                    }
                    b':' => {
                        self.get_byte()?; // skip key delimiter
                    }
                    b',' => {
                        self.get_byte()?; // skip val delimiter
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
            self.skip_white_insignificant()?;
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
            self.skip_white_insignificant()?;
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
            self.skip_white_insignificant()?;
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
            self.skip_white_insignificant()?;
            let b = self.peek_byte();
            if b == b'}' {
                self.get_byte()?;
                break;
            }
            let key = self.read_int()?.0;
            self.skip_white_insignificant()?;
            let val = self.read()?;
            map.insert(key as i32, val);
        }
        return Ok(map.make_value())
    }
    fn read_datetime(&mut self) -> Result<Value, ReadError> {
        self.get_byte()?; // eat 'd'
        let v = self.read_string()?;
        if let Value::String(s) = v {
            const PATTERN: &'static str = "2020-02-03T11:59:43";
            if s.len() >= PATTERN.len() {
                let naive_str = &s[..PATTERN.len()];
                if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(naive_str, "%Y-%m-%dT%H:%M:%S") {
                    let mut msec = 0;
                    let mut offset = 0;
                    let mut rest = &naive_str[PATTERN.len()..];
                    if rest.len() > 0 && rest.as_bytes()[1] == b'.' {
                        rest = &rest[1..];
                        if rest.len() >= 3 {
                            if let Ok(msec) = rest[..3].parse::<i32>() {
                                rest = &rest[3..];
                            } else {
                                return Err(self.make_error(&format!("Invalid DateTime msec part: '{}", rest)))
                            }
                        }
                        if rest.len() == 3 {
                            if let Ok(hrs) = rest.parse::<i32>() {
                                offset = 60 * 60 * hrs;
                            } else {
                                return Err(self.make_error(&format!("Invalid DateTime TZ part: '{}", rest)))
                            }
                        }
                        if rest.len() == 5 {
                            if let Ok(hrs) = rest.parse::<i32>() {
                                offset = 60 * (60 * (hrs / 60) + (hrs % 60));
                            } else {
                                return Err(self.make_error(&format!("Invalid DateTime TZ part: '{}", rest)))
                            }
                        }
                    }
                    let dt = DateTime::from_epoch_msec_tz(ndt.timestamp(), offset);
                    return Ok(dt.make_value())
                }
            }
            return Err(self.make_error(&format!("Invalid DateTime: '{}", s)))
        }
        return Err(self.make_error("Invalid DateTime"))
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

impl<'a, R> Reader for CponReader<'a, R>
    where R: Read
{
    fn read(&mut self) -> ReadResult {
        self.skip_white_insignificant()?;
        let mut b = self.peek_byte();
        let mut mm: Option<MetaMap> = None;
        if b == b'<' {
            mm = Some(self.read_meta()?);
            self.skip_white_insignificant()?;
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
            self.skip_white_insignificant()?;
            let b = self.peek_byte();
            if b == b'>' {
                self.get_byte()?;
                break;
            }
            let key = self.read()?;
            self.skip_white_insignificant()?;
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
        self.skip_white_insignificant()?;
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

#[cfg(test)]
mod test
{
    use crate::{MetaMap, RpcValue};
    use crate::Decimal;
    use std::collections::HashMap;
    use crate::rpcvalue::ToRpcValue;
    use crate::cponreader::CponReader;
    use crate::reader::Reader;

    #[test]
    fn test_read() {
        assert_eq!("null".to_rpcvalue().unwrap().is_null(), true);
        assert_eq!("false".to_rpcvalue().unwrap().to_bool(), false);
        assert_eq!("true".to_rpcvalue().unwrap().to_bool(), true);
        assert_eq!("0".to_rpcvalue().unwrap().to_i32(), 0);
        assert_eq!("123".to_rpcvalue().unwrap().to_i32(), 123);
        assert_eq!("-123".to_rpcvalue().unwrap().to_i32(), -123);
        assert_eq!("+123".to_rpcvalue().unwrap().to_i32(), 123);
        assert_eq!("123u".to_rpcvalue().unwrap().to_u32(), 123u32);
        assert_eq!("0xFF".to_rpcvalue().unwrap().to_i32(), 255);
        assert_eq!("-0x1000".to_rpcvalue().unwrap().to_i32(), -4096);
        assert_eq!("123.4".to_rpcvalue().unwrap().to_decimal(), Decimal::new(1234, -1));
        assert_eq!("0.123".to_rpcvalue().unwrap().to_decimal(), Decimal::new(123, -3));
        assert_eq!("-0.123".to_rpcvalue().unwrap().to_decimal(), Decimal::new(-123, -3));
        assert_eq!("0e0".to_rpcvalue().unwrap().to_decimal(), Decimal::new(0, 0));
        assert_eq!("0.123e3".to_rpcvalue().unwrap().to_decimal(), Decimal::new(123, 0));
        assert_eq!("1000000.".to_rpcvalue().unwrap().to_decimal(), Decimal::new(1000000, 0));

        assert_eq!(r#""foo""#.to_rpcvalue().unwrap().to_str(), "foo");
        assert_eq!(r#""ěščřžýáí""#.to_rpcvalue().unwrap().to_str(), "ěščřžýáí");
        assert_eq!("\"foo\tbar\nbaz\"".to_rpcvalue().unwrap().to_str(), "foo\tbar\nbaz");
        assert_eq!(r#""foo\"bar""#.to_rpcvalue().unwrap().to_str(), r#"foo"bar"#);

        let lst1 = vec![RpcValue::new(123), RpcValue::new("foo")];
        let cpon = r#"[123 , "foo"]"#;
        let rv = cpon.to_rpcvalue().unwrap();
        let lst2 = rv.to_list();
        assert_eq!(lst2, &lst1);

        let mut map: HashMap<String, RpcValue> = HashMap::new();
        map.insert("foo".to_string(), RpcValue::new(123));
        map.insert("bar".to_string(), RpcValue::new("baz"));
        let cpon = r#"{"foo": 123,"bar":"baz"}"#;
        assert_eq!(cpon.to_rpcvalue().unwrap().to_map(), &map);

        let mut map: HashMap<i32, RpcValue> = HashMap::new();
        map.insert(1, RpcValue::new(123));
        map.insert(2, RpcValue::new("baz"));
        let cpon = r#"i{1: 123,2:"baz"}"#;
        assert_eq!(cpon.to_rpcvalue().unwrap().to_imap(), &map);

        let cpon = r#"<1: 123,2:"baz">"#;
        let mut b = cpon.as_bytes();
        let mut rd = CponReader::new(&mut b);
        let mm1 = rd.read_meta().unwrap();
        let mut mm2 = MetaMap::new();
        mm2.insert(1, RpcValue::new(123));
        mm2.insert(2, RpcValue::new("baz"));
        assert_eq!(mm1, mm2);

        //let cpon1 = r#"<1:123,2:"foo","bar":"baz">42"#;
        //let rv = cpon1.to_rpcvalue().unwrap();
        //let cpon2 = rv.to_cpon_string().unwrap();
        //assert_eq!(cpon1, cpon2);
    }

}