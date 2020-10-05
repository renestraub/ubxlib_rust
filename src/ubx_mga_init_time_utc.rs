use chrono::prelude::*;
use serde::Serialize;

use crate::cid::UbxCID;
use crate::frame::{UbxFrameInfo, UbxFrameSerialize, UbxFrameWithData};

const CLS: u8 = 0x13;
const ID: u8 = 0x40;

#[derive(Default, Debug, Serialize)]
pub struct MgaIniTimeUtc {
    pub msg_type: u8, // Name type is a keyword that can't be used in Rust
    pub msg_version: u8,
    pub msg_ref: u8, // Name ref is a keyword that can't be used in Rust

    pub leap_secs: i8,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub res1: u8,
    pub ns: u32,

    pub tacc_s: u16,
    pub res2: [u8; 2],
    pub tacc_ns: u32,
}

pub struct UbxMgaIniTimeUtc {
    frame: UbxFrameWithData<MgaIniTimeUtc>,
}

impl UbxMgaIniTimeUtc {
    pub fn new() -> Self {
        Self {
            frame: UbxFrameWithData::new("UBX-MGA-INI-TIME_UTC", UbxCID::new(CLS, ID)),
        }
    }

    pub fn set_date_time(&mut self, utc: &DateTime<Utc>) {
        let data = &mut self.frame.data;

        data.msg_type = 0x10; // 0x10 for UTC time format
        data.msg_version = 0x00;
        data.msg_ref = 0x00; // receipt of message will be inaccurate

        data.leap_secs = -128; // number of leap seconds is unknown

        data.year = utc.year() as u16;
        data.month = utc.month() as u8;
        data.day = utc.day() as u8;
        data.hour = utc.hour() as u8;
        data.minute = utc.minute() as u8;
        data.second = utc.second() as u8;
        data.ns = 0; // dt.microsecond * 1000.0

        data.tacc_s = 10;
        data.tacc_ns = 0; // 999999999
                          // Accuracy 0 taken from u-center example
                          // not sure whether this is correct
    }
}

impl UbxFrameInfo for UbxMgaIniTimeUtc {
    fn name(&self) -> &'static str {
        self.frame.name()
    }

    fn cid(&self) -> UbxCID {
        self.frame.cid()
    }
}

impl UbxFrameSerialize for UbxMgaIniTimeUtc {
    fn to_bin(&self) -> Vec<u8> {
        self.frame.to_bin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::UbxFrameSerialize;

    #[test]
    fn new() {
        let dut = UbxMgaIniTimeUtc::new();
        let msg = dut.to_bin();
        assert_eq!(msg[0..6], [0xb5, 0x62, 0x13, 0x40, 24, 0]);
    }
}
