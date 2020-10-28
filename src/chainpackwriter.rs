use crate::{RpcValue, MetaMap, metamap::MetaKey, Decimal, DateTime, WriteResult, Value};
use std::io;
use crate::writer::{ByteWriter, Writer};
use std::io::Write;
use std::collections::HashMap;

pub(crate) enum PackingSchema {
    Null = 128,
    UInt,
    Int,
    Double,
    Bool,
    Blob,
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

pub struct ChainPackWriter<'a, W>
    where W: Write
{
    byte_writer: ByteWriter<'a, W>,
}

impl<'a, W> ChainPackWriter<'a, W>
    where W: 'a + io::Write
{
    pub fn new(write: &'a mut W) -> Self {
        ChainPackWriter {
            byte_writer: ByteWriter::new(write),
        }
    }

    fn write_byte(&mut self, b: u8) -> WriteResult {
        self.byte_writer.write_byte(b)
    }
    fn write_bytes(&mut self, b: &[u8]) -> WriteResult {
        self.byte_writer.write_bytes(b)
    }

    /// see https://en.wikipedia.org/wiki/Find_first_set#CLZ
    fn significant_bits_part_length(num: u64) -> u32 {
        let mut len = 0;
        let mut n = num;
        if (n & 0xFFFFFFFF00000000) != 0 {
            len += 32;
            n >>= 32;
        }
        if (n & 0xFFFF0000) != 0 {
            len += 16;
            n >>= 16;
        }
        if (n & 0xFF00) != 0 {
            len += 8;
            n >>= 8;
        }
        if (n & 0xF0) != 0 {
            len += 4;
            n >>= 4;
        }
        const SIG_TABLE_4BIT: [u8; 16] =  [ 0, 1, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4 ];
        len += SIG_TABLE_4BIT[n as usize];
        return len as u32
    }
    /// number of bytes needed to encode bit_len
    fn bytes_needed(bit_len: u32) -> u32 {
        let mut cnt = 0;
        if bit_len <= 28 {
            cnt = (bit_len - 1) / 7 + 1;
        } else {
            cnt = (bit_len - 1) / 8 + 2;
        }
        return cnt
    }
    /// return max bit length >= bit_len, which can be encoded by same number of bytes
    fn expand_bit_len(bit_len: u32) -> u32 {
        let byte_cnt = Self::bytes_needed(bit_len);
        if bit_len <= 28 {
            byte_cnt * (8 - 1) - 1
        } else {
            (byte_cnt - 1) * 8 - 1
        }
    }
    /** UInt
    0 ...  7 bits  1  byte  |0|x|x|x|x|x|x|x|<-- LSB
    8 ... 14 bits  2  bytes |1|0|x|x|x|x|x|x| |x|x|x|x|x|x|x|x|<-- LSB
    15 ... 21 bits  3  bytes |1|1|0|x|x|x|x|x| |x|x|x|x|x|x|x|x| |x|x|x|x|x|x|x|x|<-- LSB
    22 ... 28 bits  4  bytes |1|1|1|0|x|x|x|x| |x|x|x|x|x|x|x|x| |x|x|x|x|x|x|x|x| |x|x|x|x|x|x|x|x|<-- LSB
    29+       bits  5+ bytes |1|1|1|1|n|n|n|n| |x|x|x|x|x|x|x|x| |x|x|x|x|x|x|x|x| |x|x|x|x|x|x|x|x| ... <-- LSB
                    n ==  0 ->  4 bytes number (32 bit number)
                    n ==  1 ->  5 bytes number
                    n == 14 -> 18 bytes number
                    n == 15 -> for future (number of bytes will be specified in next byte)
    */
    fn write_uint_data_helper(&mut self, number: u64, bit_len: u32) -> WriteResult {
        const BYTE_CNT_MAX: u32 = 32;
        let byte_cnt = Self::bytes_needed(bit_len);
        assert!(byte_cnt <= BYTE_CNT_MAX, format!("Max int byte size {} exceeded", BYTE_CNT_MAX));
        let mut bytes: [u8; BYTE_CNT_MAX as usize] = [0; BYTE_CNT_MAX as usize];
        let mut num = number;
        let mut len = 0;
        for i in (0 .. byte_cnt).rev() {
            let r = (num & 255) as u8;
            bytes[i as usize] = r;
            num = num >> 8;
            len = i;
        }
        if bit_len <= 28 {
            let mut mask = 0xf0 << (4 - byte_cnt);
            bytes[0] = bytes[0] & ((!mask) as u8);
            mask <<= 1;
            bytes[0] |= mask;
        }
        else {
            bytes[0] = (0xf0 | (byte_cnt - 5)) as u8;
        }
        let mut cnt = 0;
        for i in 0 .. byte_cnt {
            let r = bytes[i as usize];
            self.write_byte(r)?;
        }
        return Ok(cnt)
    }
    pub fn write_uint_data(&mut self, number: u64) -> WriteResult {
        let bitlen = Self::significant_bits_part_length(number);
        let cnt = self.write_uint_data_helper(number, bitlen)?;
        Ok(cnt)
    }
    fn write_int_data(&mut self, number: i64) -> WriteResult {
        let mut num;
        let neg;
        if number < 0 {
            num = (-number) as u64;
            neg = true;
        } else {
            num = number as u64;
            neg = false;
        };

        let bitlen = Self::significant_bits_part_length(num as u64) + 1; // add sign bit
        if neg {
            let sign_pos = Self::expand_bit_len(bitlen);
            let sign_bit_mask = (1 as u64) << sign_pos;
            num |= sign_bit_mask;
        }
        let cnt = self.write_uint_data_helper(num as u64, bitlen)?;
        Ok(cnt)
    }

    fn write_int(&mut self, n: i64) -> WriteResult {
        let mut cnt = 0;
        if n < 64 {
            self.write_byte(((n % 64) + 64) as u8)?;
        }
        else {
            self.write_byte(PackingSchema::Int as u8)?;
            self.write_int_data(n)?;
        }
        Ok(cnt)
    }
    fn write_uint(&mut self, n: u64) -> WriteResult {
        let mut cnt = 0;
        if n < 64 {
            self.write_byte((n % 64) as u8)?;
        }
        else {
            self.write_byte(PackingSchema::UInt as u8)?;
            self.write_uint_data(n)?;
        }
        Ok(cnt)
    }
    fn write_double(&mut self, n: f64) -> WriteResult {
        let s = n.to_string();
        let cnt = self.write_bytes(s.as_bytes())?;
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
        self.write_bytes(s.as_bytes())?;
        self.write_byte(b'"')?;
        return Ok(cnt)
    }
    fn write_list(&mut self, lst: &Vec<RpcValue>) -> WriteResult {
        let cnt = self.write_byte(PackingSchema::List as u8)?;
        for v in lst {
            self.write(v)?;
        }
        self.write_byte(PackingSchema::TERM as u8)?;
        Ok(self.byte_writer.count() - cnt)
    }
    fn write_map(&mut self, map: &HashMap<String, RpcValue>) -> WriteResult {
        let cnt = self.write_byte(PackingSchema::Map as u8)?;
        for (k, v) in map {
            self.write_string(k)?;
            self.write(v)?;
        }
        self.write_byte(PackingSchema::TERM as u8)?;
        Ok(self.byte_writer.count() - cnt)
    }
    fn write_imap(&mut self, map: &HashMap<i32, RpcValue>) -> WriteResult {
        let cnt = self.write_byte(PackingSchema::IMap as u8)?;
        for (k, v) in map {
            self.write_int(*k as i64)?;
            self.write(v)?;
        }
        self.write_byte(PackingSchema::TERM as u8)?;
        Ok(self.byte_writer.count() - cnt)
    }
    fn write_string(&mut self, s: &str) -> WriteResult {
        let cnt = self.write_byte(PackingSchema::String as u8)?;
        self.write_uint_data(s.len() as u64)?;
        self.write_bytes(s.as_bytes())?;
        Ok(self.byte_writer.count() - cnt)
    }
    fn write_blob(&mut self, blob: &[u8]) -> WriteResult {
        let cnt = self.write_byte(PackingSchema::Blob as u8)?;
        self.write_uint_data(blob.len() as u64)?;
        self.write_bytes(blob)?;
        Ok(self.byte_writer.count() - cnt)
    }
}

impl<'a, W> Writer for ChainPackWriter<'a, W>
    where W: io::Write
{
    fn write_meta(&mut self, map: &MetaMap) -> WriteResult
    {
        let cnt = self.byte_writer.count();
        self.write_byte(PackingSchema::MetaMap as u8)?;
        for k in map.0.iter() {
            match &k.key {
                MetaKey::String(s) => self.write_blob(s.as_bytes())?,
                MetaKey::Int(i) => self.write_int(*i as i64)?,
            };
            self.write(&k.value)?;
        }
        self.write_byte(PackingSchema::TERM as u8)?;
        Ok(self.byte_writer.count() - cnt)
    }
    fn write(&mut self, val: &RpcValue) -> WriteResult
    {
        let cnt = self.byte_writer.count();
        let mm = val.meta();
        if !mm.is_empty() {
            self.write_meta(mm)?;
        }
        self.write_value(val.value())?;
        Ok(self.byte_writer.count() - cnt)
    }
    fn write_value(&mut self, val: &Value) -> WriteResult
    {
        let cnt = self.byte_writer.count();
        match val {
            Value::Null => self.write_byte(PackingSchema::Null as u8)?,
            Value::Bool(b) => if *b {
                self.write_byte(PackingSchema::TRUE as u8)?
            } else {
                self.write_byte(PackingSchema::FALSE as u8)?
            },
            Value::Int(n) => self.write_int(*n)?,
            Value::UInt(n) => self.write_uint(*n)?,
            Value::String(s) => self.write_string(s)?,
            Value::Blob(b) => self.write_blob(b)?,
            Value::Double(n) => self.write_double(*n)?,
            Value::Decimal(d) => self.write_decimal(d)?,
            Value::DateTime(d) => self.write_datetime(d)?,
            Value::List(lst) => self.write_list(lst)?,
            Value::Map(map) => self.write_map(map)?,
            Value::IMap(map) => self.write_imap(map)?,
        };
        Ok(self.byte_writer.count() - cnt)
    }

}

