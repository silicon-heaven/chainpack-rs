use chainpack::{RpcValue};
use log;

// fn to_bin(v: &[u8]) -> String {
//     let mut s = String::new();
//     for b in v {
//         if !s.is_empty() {
//             s.push('|');
//         }
//         s += &format!("{:08b}", b);
//     }
//     s
// }

fn main() {
    env_logger::init();
    let cpon1 = r#"<1:"foo">[1,2,3]"#;
    let rv1 = RpcValue::from_cpon(cpon1).unwrap();
    let chpk = rv1.to_chainpack();
    let rv2 = RpcValue::from_chainpack(&chpk).unwrap();
    let cpon2 = rv2.to_cpon();
    println!("cpon1: {}", cpon1);
    log::info!("from cpon: {}", rv1);
    log::info!("from chainpack: {}", rv2);
    log::info!("cpon2: {}", cpon2);
}