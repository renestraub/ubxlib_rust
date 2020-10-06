use chrono::prelude::*;
use serde::Serialize;

use crate::cid::UbxCID;
use crate::frame::UbxFrameWithData;

const CLS: u8 = 0x13;
const ID: u8 = 0x40;

#[derive(Default, Debug, Serialize)]
pub struct MgaIniTimeUtc {
    pub msg_type: u8, // Name type is a keyword that can't be used in Rust
    pub msg_version: u8,
    pub msg_ref: u8, // Name ref is a keyword that can't be used in Rust

    pub leap_secs: i8, // number of leap seconds since 1980 (or 0x80 = -128 if unknown)
    pub year: u16,
    pub month: u8, // starting at 1
    pub day: u8,   // starting at 1
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub res1: u8,
    pub ns: u32,

    pub tacc_s: u16,
    pub res2: [u8; 2],
    pub tacc_ns: u32,
}

impl MgaIniTimeUtc {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

pub struct UbxMgaIniTimeUtc {}

impl UbxMgaIniTimeUtc {
    pub fn new() -> UbxFrameWithData<MgaIniTimeUtc> {
        UbxFrameWithData::init(
            "UBX-MGA-INI-TIME_UTC",
            UbxCID::new(CLS, ID),
            MgaIniTimeUtc::new(),
        )
    }
}

impl UbxFrameWithData<MgaIniTimeUtc> {
    pub fn set_date_time(&mut self, utc: &DateTime<Utc>) {
        self.data.msg_type = 0x10; // 0x10 for UTC time format
        self.data.msg_version = 0x00;
        self.data.msg_ref = 0x00; // receipt of message will be inaccurate

        self.data.leap_secs = -128; // number of leap seconds is unknown

        self.data.year = utc.year() as u16;
        self.data.month = utc.month() as u8;
        self.data.day = utc.day() as u8;
        self.data.hour = utc.hour() as u8;
        self.data.minute = utc.minute() as u8;
        self.data.second = utc.second() as u8;
        self.data.ns = 0; // dt.microsecond * 1000.0

        self.data.tacc_s = 10;
        self.data.tacc_ns = 0; // 999999999
                               // Accuracy 0 taken from u-center example
                               // not sure whether this is correct
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::UbxFrameInfo;
    use crate::frame::UbxFrameSerialize;
    use chrono::Utc;

    #[test]
    fn new() {
        let dut = UbxMgaIniTimeUtc::new();
        assert_eq!(dut.name(), "UBX-MGA-INI-TIME_UTC");
        let msg = dut.to_bin();
        assert_eq!(msg[0..6], [0xb5, 0x62, 0x13, 0x40, 24, 0]);
    }

    #[test]
    fn set_date_time() {
        let mut dut = UbxMgaIniTimeUtc::new();

        let utc = Utc.ymd(2020, 2, 3).and_hms_milli(11, 22, 33, 444);
        dut.set_date_time(&utc);

        assert_eq!(dut.data.year, 2020);
        assert_eq!(dut.data.month, 2);
        assert_eq!(dut.data.day, 3);
        assert_eq!(dut.data.hour, 11);
        assert_eq!(dut.data.minute, 22);
        assert_eq!(dut.data.second, 33);

        let msg = dut.to_bin();
        assert_eq!(
            msg[0..30],
            [
                0xb5, 0x62, 0x13, 0x40, 24, 0, 0x10, 0x00, 0x00, 128, 228, 7, 2, 3, 11, 22, 33, 0,
                0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0
            ]
        );
    }
}
