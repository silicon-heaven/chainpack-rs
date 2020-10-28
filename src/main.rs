use chainpack::{RpcValue};
use chainpack::CponReader;

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
    let cpon1 = r#"<1:"foo">[1,2,3]"#;
    let rv1 = RpcValue::from_cpon(cpon1).unwrap();
    let chpk = rv1.to_chainpack();
    let rv2 = RpcValue::from_chainpack(&chpk).unwrap();
    let cpon2 = rv2.to_cpon();
    println!("cpon1: {}", cpon1);
    println!("from cpon: {}", rv1);
    println!("from chainpack: {}", rv2);
    println!("cpon2: {}", cpon2);
}