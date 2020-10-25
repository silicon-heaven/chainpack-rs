use chrono::Offset;
use chainpack::RpcValue;
use chainpack::chainpack::writer::Writer;
use chainpack::chainpack::CPWriter;

fn to_bin(v: &[u8]) -> String {
    let mut s = String::new();
    for b in v {
        if !s.is_empty() {
            s.push('|');
        }
        s += &format!("{:08b}", b);
    }
    s
}

fn main() {
    let v: Vec<u64> = vec![2, 16, 127, 128, 512, 4096, 32768, 1048576, 8388608, 33554432, 268435456, 68719476736, 17592186044416, 140737488355328, 4503599627370496];
    for n in v {
        let mut buff: Vec<u8> = Vec::new();
        let mut wr = Writer::new(&mut buff);
        let cnt = wr.write(&RpcValue::new(n)).unwrap();
        println!("n: {:16}, len: {}, data: {}", n, cnt, to_bin(&buff));
    }
}