//use crate::rpcvalue::RpcValue;

use chrono::Offset;

/// msec: 57, tz: 7;
/// I'm storing whole DateTime in one i64 to keep size_of RpcValue == 24
#[derive(Debug, Clone, PartialEq)]
pub struct DateTime (i64);

impl DateTime {
    pub fn invalid() -> DateTime {
        DateTime::from_epoch_msec(0)
    }

    pub fn now() -> DateTime {
        let dt = chrono::offset::Local::now();
        let msec = dt.naive_utc().timestamp_millis();
        let offset = dt.offset().local_minus_utc() / 60 / 15;
        DateTime::from_epoch_msec_tz(msec, offset)
    }

    pub fn from_datetime<Tz: chrono::TimeZone>(dt: &chrono::DateTime<Tz>) -> DateTime {
        let msec = dt.naive_utc().timestamp_millis();
        let offset = dt.offset().fix().local_minus_utc();
        DateTime::from_epoch_msec_tz(msec, offset)
    }

    pub fn from_epoch_msec_tz(epoch_msec: i64, utc_offset_sec: i32) -> DateTime {
        let mut msec = epoch_msec;
        // offset in quarters of hour
        const MASK: i64 = 127;
        msec *= MASK + 1;
        msec &= !MASK;
        let offset: i64 = (utc_offset_sec / 60 / 15).into();
        msec |= offset & MASK;
        DateTime(msec)
    }
    pub fn from_epoch_msec(epoch_msec: i64) -> DateTime {
        Self::from_epoch_msec_tz(epoch_msec, 0)
    }

    pub fn to_epoch_msec(&self) -> i64 {
        let mut msec = self.0;
        // offset in quarters of hour
        const MASK: i64 = 127;
        msec /= MASK + 1;
        msec
    }
    pub fn utc_offset(&self) -> i32 {
        let mut offset = self.0;
        // offset in quarters of hour
        const MASK: i64 = 127;
        offset &= MASK;
        if (offset & ((MASK + 1) / 2)) != 0 {
            // sign extension
            offset |= !MASK;
        }
        (offset * 15 * 60) as i32
    }

    pub fn to_naivedatetime(&self) -> chrono::NaiveDateTime {
        chrono::NaiveDateTime::from_timestamp(self.to_epoch_msec(), 0)
    }
    pub fn to_datetime(&self) -> chrono::DateTime<chrono::offset::FixedOffset> {
        chrono::DateTime::from_utc(self.to_naivedatetime()
                                   , chrono::FixedOffset::east(self.utc_offset()))
    }
}

