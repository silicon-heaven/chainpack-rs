use crate::{RpcValue, MetaMap, metamap::MetaKey, Decimal, DateTime};
use std::io;
use crate::rpcvalue::Value;
use std::string::FromUtf8Error;
use std::collections::HashMap;
use chrono::{NaiveDateTime, Datelike};
//use std::slice::Iter;
//use std::intrinsics::write_bytes;

pub type WriterResult = std::io::Result<usize>;

pub struct Writer<'a, 'b, W>
{
    write: &'a mut W,
    pub indent: &'b str,
}

pub fn to_cpon(rv: &RpcValue) -> Vec<u8>
{
    let mut buff = Vec::new();
    let mut wr = Writer::new(&mut buff);
    let res= wr.write(rv);
    if let Ok(_) = res {
        return buff
    }
    return Vec::new()
}
pub fn to_cpon_string(rv: &RpcValue) -> Result<String, FromUtf8Error> {
    let cp = to_cpon(rv);
    String::from_utf8(cp)
}

impl<'a, 'b, W> Writer<'a, 'b, W>
where W: io::Write
{
    pub fn new(write: &'a mut W) -> Writer<'a, 'b, W> {
        const EMPTY: &'static str = "";
        Writer {
            write,
            indent: EMPTY,
        }
    }

    pub fn write_meta(&mut self, map: &MetaMap) -> WriterResult
    {
        let mut cnt: usize = 0;
        cnt += self.write_byte(b'<')?;
        let mut n = 0;
        for k in map.0.iter() {
            if n == 0 {
                n += 1;
            } else {
                cnt += self.write_byte(b',')?;
            }
            if !self.indent.is_empty() {
                cnt += self.write_byte(b'\n')?;
                cnt += self.write.write(self.indent.as_bytes())?;
            }

            match &k.key {
                MetaKey::String(s) => {
                    cnt += self.write_bytes_escaped(s.as_bytes())?;
                },
                MetaKey::Int(i) => cnt += self.write.write(i.to_string().as_bytes())?,
            }
            cnt += self.write_byte(b':')?;
            cnt += self.write(&k.value)?;
            n += 1;
        }
        if !self.indent.is_empty() {
            cnt += self.write_byte(b'\n')?;
        }
        cnt += self.write_byte(b'>')?;
        Ok(cnt)
    }
    pub fn write(&mut self, val: &RpcValue) -> WriterResult
    {
        let mm = val.meta();
        let mut cnt: usize = 0;
        if !mm.is_empty() {
            cnt += self.write_meta(mm)?;
        }
        cnt += self.write_value(val.value())?;
        Ok(cnt)
    }
    fn write_value(&mut self, val: &Value) -> WriterResult
    {
        let mut cnt: usize = 0;
        match val {
            Value::Null => cnt += self.write.write("null".as_bytes())?,
            Value::Bool(b) => cnt += if *b {
                self.write.write("true".as_bytes())?
            } else {
                self.write.write("false".as_bytes())?
            },
            Value::Int(n) => cnt += self.write_int(*n)?,
            Value::UInt(n) => {
                cnt += self.write_uint(*n)?;
                cnt += self.write_byte(b'u')?;
            },
            Value::Blob(b) => cnt += self.write_bytes_escaped(b)?,
            Value::Double(n) => cnt += self.write_double(*n)?,
            Value::Decimal(d) => cnt += self.write_decimal(d)?,
            Value::DateTime(d) => cnt += self.write_datetime(d)?,
            Value::List(lst) => cnt += self.write_list(lst)?,
            Value::Map(map) => cnt += self.write_map(map)?,
            Value::IMap(map) => cnt += self.write_imap(map)?,
        }
        Ok(cnt)
    }
    fn write_byte(&mut self, b: u8) -> WriterResult
    {
        let mut arr: [u8; 1] = [0];
        arr[0] = b;
        self.write.write(&arr)
    }
    fn write_bytes(&mut self, arr: &[u8]) -> WriterResult
    {
        self.write.write(&arr)
    }
    fn write_int(&mut self, n: i64) -> WriterResult
    {
        let s = n.to_string();
        let cnt = self.write.write(s.as_bytes())?;
        Ok(cnt)
    }
    fn write_uint(&mut self, n: u64) -> WriterResult
    {
        let s = n.to_string();
        let cnt = self.write.write(s.as_bytes())?;
        Ok(cnt)
    }
    fn write_double(&mut self, n: f64) -> WriterResult
    {
        let s = n.to_string();
        let cnt = self.write.write(s.as_bytes())?;
        Ok(cnt)
    }
    fn write_bytes_escaped(&mut self, bytes: &[u8]) -> WriterResult
    {
        let mut cnt: usize = 0;
        cnt += self.write_byte(b'"')?;
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
    fn write_decimal(&mut self, decimal: &Decimal) -> WriterResult {
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
    fn write_datetime(&mut self, dt: &DateTime) -> WriterResult {
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
    fn write_list(&mut self, lst: &Vec<RpcValue>) -> WriterResult
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
    fn write_map(&mut self, map: &HashMap<String, RpcValue>) -> WriterResult
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
            cnt += self.write_bytes_escaped(k.as_bytes())?;
            cnt += self.write_byte(b':')?;
            cnt += self.write(v)?;
        }
        cnt += self.write_byte(b'}')?;
        Ok(cnt)
    }
    fn write_imap(&mut self, map: &HashMap<i32, RpcValue>) -> WriterResult
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

#[cfg(test)]
mod test
{
    use crate::cpon::writer::{Writer, to_cpon, to_cpon_string};
    use crate::{MetaMap, RpcValue, DateTime};

    #[test]
    fn write() {
        let dt = DateTime::from_epoch_msec(0);
        let cpon = r#"d"1970-01-01T00:00:00Z""#;
        assert_eq!(RpcValue::new(dt).to_cpon_string().unwrap(), cpon);
    }

}

