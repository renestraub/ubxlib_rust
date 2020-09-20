use chrono::prelude::*;
use serde::Serialize;

use crate::cid::UbxCID;
use crate::frame::{UbxFrame, UbxFrameInfo, UbxFrameSerialize};

const CLS: u8 = 0x13;
const ID: u8 = 0x40;

#[derive(Default, Debug, Serialize)]
pub struct Data {
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

#[derive(Default, Debug)]
pub struct UbxMgaIniTimeUtc {
    pub name: &'static str,
    cid: UbxCID,
    pub data: Data,
}

impl UbxMgaIniTimeUtc {
    pub fn new() -> Self {
        Self {
            name: "UBX-MGA-INI-TIME_UTC",
            cid: UbxCID::new(CLS, ID),
            ..Default::default()
        }
    }

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

    fn save(&self) -> Vec<u8> {
        let data = bincode::serialize(&self.data).unwrap();
        assert!(data.len() == 24);
        data
    }
}

impl UbxFrameInfo for UbxMgaIniTimeUtc {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxMgaIniTimeUtc {
    fn to_bin(&self) -> Vec<u8> {
        // update binary data in frame
        let data = self.save();

        // construct a frame with correct CID and payload
        let frame = UbxFrame::construct(UbxCID::new(CLS, ID), data);
        let msg = frame.to_bytes();
        msg
        // TODO: Combine to one statement
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let dut = UbxMgaIniTimeUtc::new();
        let msg = dut.to_bin();
        assert_eq!(msg[0..6], [0xb5, 0x62, 0x13, 0x40, 24, 0]);
    }
}
