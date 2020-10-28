use chainpack::{RpcValue, ToRpcValue};
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
    let cpon = r#"<1:"foo">[12 0x3]"#;
    let rv = cpon.to_rpcvalue().unwrap();
    //let cv = rv.to_cpon().unwrap();
    println!("value: {}", rv);
}