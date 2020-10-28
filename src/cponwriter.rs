use std::io::{Write};
use crate::{RpcValue, MetaMap, Value, Decimal, DateTime};
use std::collections::HashMap;
use crate::writer::{WriteResult, Writer, ByteWriter};
use crate::metamap::MetaKey;

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
