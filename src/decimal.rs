//use crate::rpcvalue::RpcValue;

/// mantisa: 56, exponent: 8;
/// I'm storing whole Decimal in one i64 to keep size_of RpcValue == 24
#[derive(Debug, Clone, PartialEq)]
pub struct Decimal (i64);

impl Decimal {

    pub fn new(mantisa: i64, exponent: i8) -> Decimal {
        let mut n = mantisa * 256;
        n |= exponent as i64;
        Decimal(n)
    }

    pub fn decode(&self) -> (i64, i8) {
        let m = self.0 / 256;
        let e = self.0 as i8;
        (m, e)
    }
}

/*
#[derive(Debug, Clone, PartialEq)]
pub struct Decimal(chrono::Decimal<chrono::FixedOffset>);

impl Decimal {
    pub fn now() -> Decimal {
        let dt = chrono::offset::Local::now();
        Decimal(chrono::Decimal::from_utc(dt.naive_utc(), dt.offset().clone()))
    }
    pub fn from_msec_since_epoch(epoch_msec: i64) -> Decimal {
        let dt = chrono::NaiveDecimal::from_timestamp(epoch_msec, 0);
        Decimal(chrono::Decimal::from_utc(dt, chrono::FixedOffset::east(0)))
    }
    pub fn from_msec_since_epoch_tz(epoch_msec: i64, utc_offset: i32) -> Decimal {
        let dt = chrono::NaiveDecimal::from_timestamp(epoch_msec, 0);
        Decimal(chrono::Decimal::from_utc(dt, chrono::FixedOffset::east(utc_offset)))
    }
}
*/
