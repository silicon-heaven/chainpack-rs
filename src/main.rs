use chainpack::RpcValue;
use chainpack::cponreader::{CponReader};
use chainpack::reader::{CPReader};

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
    let cpon = "[12 0x3]";
    let mut buff = cpon.as_bytes();
    let mut rd = CponReader::new(&mut buff);
    //let res = CPReader::read(&mut rd);
    let res = rd.read().expect("neco-neco");
    println!("value: {:?}", res);
}