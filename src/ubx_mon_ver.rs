// use std::fmt;

use crate::cid::UbxCID as UbxCID;
use crate::frame::UbxFrame as UbxFrame;
use crate::frame::UbxFrameInfo as UbxFrameInfo;
use crate::frame::UbxFrameSerialize as UbxFrameSerialize;


const CLS: u8 = 0x0A;
const ID: u8 = 0x04;

// const CID: UbxCID = UbxCID::new(CLS, ID);

pub struct UbxMonVerPoll {
    pub name: String,
    cid: UbxCID,
}


impl UbxMonVerPoll {
    pub fn new() -> Self {
        Self {
            name: String::from("UBX-MON-VER-POLL"),
            cid: UbxCID::new(CLS, ID), 
        }
    }
}

// TODO: Can we use a generic/template here
// Add new member name
/*
use duplicate::duplicate;
#[duplicate(name; [UbxCfgRatePoll]; [UbxCfgRate])]
impl UbxFrameInfo for name {
    fn name(&self) -> &str {
        &self.name
    }

    fn cls(&self) -> u8 {
        self.frame.cid.cls()
    }

    fn id(&self) -> u8 {
        self.frame.cid.id()
    }
}
*/

impl UbxFrameInfo for UbxMonVerPoll {
    fn name(&self) -> String {
        String::from(&self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }

    fn cls(&self) -> u8 {
        self.cid.cls()
    }

    fn id(&self) -> u8 {
        self.cid.id()
    }
}

impl UbxFrameSerialize for UbxMonVerPoll {
    fn to_bin(&self) -> Vec<u8> {
        // poll frame has no data to serialize
        // just build empty frame
        let frame = UbxFrame::construct(UbxCID::new(CLS, ID), [].to_vec());
        let msg = frame.to_bytes();
        msg
    }

    // TODO: Split into two traits (serialize, deser...)
    // TODO: Only implement serialize here
    fn from_bin(&mut self, _data: Vec<u8>) {
        panic!("can't load into this frame");
        // no fields, so nothing to do
        // assert_eq!(data.len(), 0);
    }
}


#[derive(Default)]
#[derive(Debug)]
pub struct UbxMonVer {
    pub name: String,
    cid: UbxCID,

    pub sw_version: String,
    pub hw_version: String,
    pub hw_extension: String,   // TODO: Should be array of Strings
}

impl UbxMonVer {
    pub fn new() -> Self {
        let mut obj: UbxMonVer = Default::default();
        obj.name = String::from("UBX-MON-VER");
        obj.cid = UbxCID::new(CLS, ID);
        obj
    }

    pub fn load(&mut self, data: &[u8]) {
        let bytes = data.len();
        println!("UbxMonVer, got {} bytes", bytes);
        assert!(bytes >= 40);

        // TODO: make better
        let text = &data[0..30].to_vec();
        let text = String::from_utf8_lossy(text);
        self.sw_version = text.replace(|c: char| c == '\0', "");

        let text = &data[30..40].to_vec();
        let text = String::from_utf8_lossy(text);
        self.hw_version = text.replace(|c: char| c == '\0', "");

        if bytes > 40 {
            assert!((bytes - 40) % 30 == 0);

            let mut offset = 40;
            let size = 30;
            while offset < bytes {
                let text = &data[offset..offset+size].to_vec();
                let text = String::from_utf8_lossy(text);
                let text = text.replace(|c: char| c == '\0', "");
                println!("{}: {}", offset, text);

                offset += size;
            }
        }
    }
}

impl UbxFrameInfo for UbxMonVer {
    fn name(&self) -> String {
        String::from(&self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }

    fn cls(&self) -> u8 {
        self.cid.cls()
    }

    fn id(&self) -> u8 {
        self.cid.id()
    }
}

impl UbxFrameSerialize for UbxMonVer {
    // TODO: Split into two traits (serialize, deser...)
    // TODO: Only implement deserialize here
    fn to_bin(&self) -> Vec<u8> {
        panic!("can't serialize this frame")
    }

    fn from_bin(&mut self, data: Vec<u8>) {
        self.load(&data);
    }
}

/*
impl fmt::Debug for UbxMonVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UbxCfgRate")
        .field("cid", &self.cid)
        .field("measRate", &self.meas_rate)
        .field("navRate", &self.nav_rate)
        .field("timeRef", &self.time_ref)
        .finish()
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ubx_mon_ver_poll() {
        let dut = UbxMonVerPoll::new();
        assert_eq!(dut.name, "UBX-MON-VER-POLL");
        let msg = dut.to_bin();
        println!("message {:?}", msg);
        assert_eq!(msg, [0xb5, 0x62, 0x0A, 0x04, 0, 0, 14, 52]);
    }

    #[test]
    fn ubx_mon_ver() {
        let dut = UbxMonVer::new();
        assert_eq!(dut.name, "UBX-MON-VER");

        println!("message {:?}", dut);

        assert_eq!(dut.sw_version, "");
        assert_eq!(dut.hw_version, "");
    }

     #[test]
    #[should_panic]
    fn cfg_rate_serialize() {
        let dut = UbxMonVer::new();
        let _data = dut.to_bin();
    }

    /*
    #[test]
    fn cfg_rate_deserialize() {
        const DATA: [u8; 6] = [0xE8, 0x03, 0x01, 0x00, 0x34, 0x12];

        let mut dut = UbxCfgRate::new();
        dut.load(&DATA);
        assert_eq!(dut.meas_rate, 1000);
        assert_eq!(dut.nav_rate, 1);
        assert_eq!(dut.time_ref, 0x1234);
    }
    */
}
