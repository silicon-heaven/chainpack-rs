use crate::{RpcValue, MetaMap, metamap::MetaKey};
use std::io;
use crate::rpcvalue::Value;

pub type WriterResult = std::io::Result<usize>;

pub struct Writer<'a, 'b, W>
{
    writer: &'a mut W,
    pub indent: &'b str,
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
        const B1: &'static [u8] = &['<' as u8];
        const B2: &'static [u8] = &['>' as u8];
        const D1: &'static [u8] = &[':' as u8];
        const D2: &'static [u8] = &[',' as u8];
        const Q1: &'static [u8] = &['"' as u8];
        const L1: &'static [u8] = &['\n' as u8];
        let mut cnt: usize = 0;
        cnt += self.writer.write(&B1)?;
        let mut n = 0;
        for k in map.0.iter() {
            if n == 0 {
                n += 1;
            } else {
                cnt += self.writer.write(D2)?;
            }
            if !self.indent.is_empty() {
                cnt += self.writer.write(L1)?;
                cnt += self.writer.write(self.indent.as_bytes())?;
            }

            match &k.key {
                MetaKey::String(s) => {
                    cnt += self.writer.write(Q1)?;
                    cnt += self.writer.write(s.as_bytes())?;
                    cnt += self.writer.write(Q1)?;
                },
                MetaKey::Int(i) => cnt += self.writer.write(i.to_string().as_bytes())?,
            }
            cnt += self.writer.write(&D1)?;
            cnt += self.write(&k.value)?;
            n += 1;
        }
        if !self.indent.is_empty() {
            cnt += self.writer.write(L1)?;
        }
        cnt += self.writer.write(&B2)?;
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
            // Int(i64),
            _ => cnt += self.writer.write("???".as_bytes())?,
        }
        Ok(cnt)
    }
}

#[cfg(test)]
mod test
{
    use crate::cpon::writer::Writer;
    use crate::{MetaMap, RpcValue};

    #[test]
    fn size() {
        let mut mm = MetaMap::new();

        mm.insert(123, RpcValue::new(1.1));
        mm.insert("foo", RpcValue::new("bar")).insert(123, RpcValue::new("baz"));
        let v1 = vec![RpcValue::new("foo"), RpcValue::new("bar"), RpcValue::new("baz")];
        mm.insert("list", RpcValue::new(v1));

        let mut buff = Vec::new();
        let mut wr = Writer::new(&mut buff);
        wr.indent = "  ";
        let sz = wr.write_meta(&mm);
        println!("size: {} cpon: {}", sz.unwrap(), std::str::from_utf8(&buff).unwrap());
    }

}

