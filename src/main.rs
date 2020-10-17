use chrono::Offset;
use chainpack::RpcValue;

fn main() {
    let dt = chrono::offset::Local::now();
    let rv = RpcValue::new(dt.clone());
    println!("cr dt {} + {}", dt.timestamp_millis(), dt.offset().fix().local_minus_utc());
    println!("cp dt {} + {}", rv.to_datetime().to_epoch_msec(), rv.to_datetime().utc_offset());
    assert_eq!(rv.to_datetime().to_epoch_msec() + rv.to_datetime().utc_offset() as i64 * 1000
               , dt.timestamp_millis() + dt.offset().fix().local_minus_utc() as i64 * 1000);
}