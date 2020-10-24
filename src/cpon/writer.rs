use crate::{RpcValue, MetaMap, metamap::MetaKey};
use std::io;
use crate::rpcvalue::Value;
//use std::slice::Iter;
//use std::intrinsics::write_bytes;

pub type WriterResult = std::io::Result<usize>;

pub struct Writer<'a, 'b, W>
{
    writer: &'a mut W,
    pub indent: &'b str,
}

pub fn to_cpon(rv: &RpcValue) -> String
{
    let mut buff = Vec::new();
    let mut wr = Writer::new(&mut buff);
    let res= wr.write(rv);
    if let Ok(_) = res {
        unsafe {
            // writer should not generate UTF8 invalid text
            return String::from_utf8_unchecked(buff)
        }
    }
    String::new()
}

impl<'a, 'b, W> Writer<'a, 'b, W>
where W: io::Write
{
    pub fn new(writer: &'a mut W) -> Writer<'a, 'b, W> {
        const EMPTY: &'static str = "";
        Writer {
            writer,
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
                cnt += self.writer.write(self.indent.as_bytes())?;
            }

            match &k.key {
                MetaKey::String(s) => {
                    cnt += self.write_bytes_escaped(s.as_bytes())?;
                },
                MetaKey::Int(i) => cnt += self.writer.write(i.to_string().as_bytes())?,
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
            Value::Null => cnt += self.writer.write("null".as_bytes())?,
            Value::Int(n) => cnt += self.write_int(*n)?,
            Value::UInt(n) => {
                cnt += self.write_uint(*n)?;
                cnt += self.write_byte(b'u')?;
            },
            Value::Bytes(b) => cnt += self.write_bytes_escaped(b)?,
            Value::Double(n) => cnt += self.write_double(*n)?,
            Value::List(lst) => cnt += self.write_list(lst.iter())?,
            _ => {
                cnt += self.writer.write("?".as_bytes())?;
                cnt += self.writer.write(val.type_name().as_bytes())?;
                cnt += self.writer.write("?".as_bytes())?;
            },
        }
        Ok(cnt)
    }
    fn write_byte(&mut self, b: u8) -> WriterResult
    {
        let mut arr: [u8; 1] = [0];
        arr[0] = b;
        self.writer.write(&arr)
    }
    fn write_int(&mut self, n: i64) -> WriterResult
    {
        let s = n.to_string();
        let cnt = self.writer.write(s.as_bytes())?;
        Ok(cnt)
    }
    fn write_uint(&mut self, n: u64) -> WriterResult
    {
        let s = n.to_string();
        let cnt = self.writer.write(s.as_bytes())?;
        Ok(cnt)
    }
    fn write_double(&mut self, n: f64) -> WriterResult
    {
        let s = n.to_string();
        let cnt = self.writer.write(s.as_bytes())?;
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
    fn write_list(&mut self, it: std::slice::Iter<RpcValue>) -> WriterResult
    {
        let mut cnt = 0;
        cnt += self.write_byte(b'[')?;
        let mut n = 0;
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
}

#[cfg(test)]
mod test
{
    use crate::cpon::writer::{Writer, to_cpon};
    use crate::{MetaMap, RpcValue};

    #[test]
    fn write() {
        let mut mm = MetaMap::new();

        mm.insert(123, RpcValue::new(1.1));
        mm.insert(42, RpcValue::new(1));
        mm.insert("foo", RpcValue::new("bar")).insert(123, RpcValue::new("baz"));
        let v1 = vec![RpcValue::new("foo"), RpcValue::new("bar"), RpcValue::new("baz")];
        mm.insert("list", RpcValue::new(v1));

        let mut buff = Vec::new();
        let mut wr = Writer::new(&mut buff);
        wr.indent = "  ";
        let sz = wr.write_meta(&mm);
        println!("size: {} cpon: {}", sz.unwrap(), std::str::from_utf8(&buff).unwrap());

        let mut rv = RpcValue::new("test");
        rv.set_meta(mm);
        println!("cpon: {}", to_cpon(&rv));
    }

}

