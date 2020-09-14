use std::fmt;

use crate::cid::UbxCID as UbxCID;
use crate::frame::UbxFrame as UbxFrame;
use crate::frame::UbxFrameInfo as UbxFrameInfo;
use crate::frame::UbxFrameSerialize as UbxFrameSerialize;
use crate::frame::UbxFrameDeSerialize as UbxFrameDeSerialize;


const CLS: u8 = 0x06;
const ID: u8 = 0x08;


pub struct UbxCfgRatePoll {
    pub name: &'static str,
    cid: UbxCID,
}


impl UbxCfgRatePoll {
    pub fn new() -> Self {
        Self {
            name: "UBX-CFG-RATE-POLL",
            cid: UbxCID::new(CLS, ID), 
        }
    }
}

impl UbxFrameInfo for UbxCfgRatePoll {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxCfgRatePoll {
    fn to_bin(&self) -> Vec<u8> {
        // poll frame has no data to serialize
        // just build empty frame
        let frame = UbxFrame::construct(
            UbxCID::new(CLS, ID), 
            [].to_vec());
        let msg = frame.to_bytes();
        msg
    }
}



#[derive(Default)]
pub struct UbxCfgRate {
    pub name: &'static str,
    cid: UbxCID,

    pub meas_rate: u16,  // Time elapsed between two measuremnts in ms
    pub nav_rate: u16,   // Number of measurements for NAV solution
    pub time_ref: u16,
}

impl UbxCfgRate {
    // TODO: ctor to create frame with data
    // loads frame ...
    pub fn new() -> Self {
        Self { 
            name: "UBX-CFG-RATE",
            cid: UbxCID::new(CLS, ID), 
            ..Default::default()
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        assert!(data.len() == 6);

        // TODO: improve further, this can't be the best solution
        self.meas_rate = u16::from_le_bytes([data[0], data[1]]);
        self.nav_rate = u16::from_le_bytes([data[2], data[3]]);
        self.time_ref = u16::from_le_bytes([data[4], data[5]]);
    }

    pub fn save(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(6);
        let bytes1 = self.meas_rate.to_le_bytes();
        let bytes2 = self.nav_rate.to_le_bytes();
        let bytes3 = self.time_ref.to_le_bytes();
        // println!("{:?} {:?} {:?}" , bytes1, bytes2, bytes3);

        data.append(&mut bytes1.to_vec());
        data.append(&mut bytes2.to_vec());
        data.append(&mut bytes3.to_vec());
        data
    }
}

impl UbxFrameInfo for UbxCfgRate {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxCfgRate {
    fn to_bin(&self) -> Vec<u8> {
        // println!("{:?}", &self);
        // update binary data in frame
        let data = self.save();

        // construct a frame with correct CID and payload
        let frame = UbxFrame::construct(UbxCID::new(CLS, ID), data);
        let msg = frame.to_bytes();
        msg
        // TODO: Combine to one statement
    }
}

impl UbxFrameDeSerialize for UbxCfgRate {
    fn from_bin(&mut self, data: Vec<u8>) {
        assert_eq!(data.len(), 6);
        self.load(&data);
    }
}


impl fmt::Debug for UbxCfgRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UbxCfgRate")
        .field("cid", &self.cid)
        .field("measRate", &self.meas_rate)
        .field("navRate", &self.nav_rate)
        .field("timeRef", &self.time_ref)
        .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cfg_rate_poll() {
        let dut = UbxCfgRatePoll::new();
        assert_eq!(dut.name, "UBX-CFG-RATE-POLL");
        let msg = dut.to_bin();
        println!("message {:?}", msg);
        assert_eq!(msg, [0xb5, 0x62, CLS, ID, 0, 0, 14, 48]);
    }

    #[test]
    fn cfg_rate_load() {
        const DATA: [u8; 6] = [0xe8, 0x03, 0x01, 0x00, 0x34, 0x12];
        let mut dut = UbxCfgRate::new();
        dut.load(&DATA.to_vec());

        assert_eq!(dut.meas_rate, 1000);
        assert_eq!(dut.nav_rate, 1);
        assert_eq!(dut.time_ref, 0x1234);
    }

    #[test]
    #[should_panic]
    fn cfg_rate_load_too_few_values() {
        const DATA: [u8; 5] = [0xe8, 0x03, 0x01, 0x00, 0x34];
        let mut dut = UbxCfgRate::new();
        // println!("{:?}", dut);
        dut.load(&DATA.to_vec());
        // println!("{:?}", dut);

        assert_eq!(dut.meas_rate, 1000);
        assert_eq!(dut.nav_rate, 1);
        assert_eq!(dut.time_ref, 0x1234);
    }

    #[test]
    fn cfg_rate_change() {
        let mut dut = UbxCfgRate::new();
        assert_eq!(dut.meas_rate, 0);
        assert_eq!(dut.nav_rate, 0);
        assert_eq!(dut.time_ref, 0);

        dut.meas_rate = 1000;
        dut.nav_rate = 1;
        dut.time_ref = 0x1234;
        let data = dut.save();
        assert_eq!(data, [0xE8, 3, 1, 0, 0x34, 0x12]);
    }

    #[test]
    fn cfg_rate_serialize() {
        let mut dut = UbxCfgRate::new();
        assert_eq!(dut.meas_rate, 0);
        assert_eq!(dut.nav_rate, 0);
        assert_eq!(dut.time_ref, 0);

        dut.meas_rate = 1000;
        dut.nav_rate = 1;
        dut.time_ref = 0x1234;

        let data = dut.to_bin();
        assert_eq!(data, [0xb5, 0x62, 0x06, 0x08, 0x06, 0x00, 0xE8, 0x03, 0x01, 0x00, 0x34, 0x12, 70, 177]);
    }

    #[test]
    fn cfg_rate_deserialize() {
        const DATA: [u8; 6] = [0xE8, 0x03, 0x01, 0x00, 0x34, 0x12];

        let mut dut = UbxCfgRate::new();
        dut.load(&DATA);
        assert_eq!(dut.meas_rate, 1000);
        assert_eq!(dut.nav_rate, 1);
        assert_eq!(dut.time_ref, 0x1234);
    }
}
