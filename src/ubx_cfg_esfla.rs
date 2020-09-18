use serde::Serialize;

use crate::cid::UbxCID;
use crate::frame::{UbxFrame, UbxFrameInfo, UbxFrameSerialize};


const CLS: u8 = 0x06;
const ID: u8 = 0x2F;


#[derive(Default, Debug, Serialize)]
pub struct Data {
    pub version: u8,
    pub num_configs: u8,
    pub res1: [u8; 2],

    pub leverarm_type: u8,
    pub res2: u8,
    pub leverarm_x: i16,
    pub leverarm_y: i16,
    pub leverarm_z: i16,
}

impl Data {
    pub fn new() -> Self {
        Self { 
            version: 0x00,
            num_configs: 1,
            ..Default::default()
        }
    }
}


#[derive(Default, Debug)]
pub struct UbxCfgEsflaSet {
    pub name: &'static str,
    cid: UbxCID,
    pub data: Data,
}

impl UbxCfgEsflaSet {
    pub fn new() -> Self {
        Self { 
            name: "UBX-CFG-ESFLA",
            cid: UbxCID::new(CLS, ID),
            data: Data::new(), 
            ..Default::default()
        }
    }

    pub fn save(&self) -> Vec<u8> {
        let data = bincode::serialize(&self.data).unwrap();
        assert!(data.len() == 12);
        data
    }
}

impl UbxFrameInfo for UbxCfgEsflaSet {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxCfgEsflaSet {
    fn to_bin(&self) -> Vec<u8> {
        let data = self.save();

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
    fn positive_values() {
        let mut dut = UbxCfgEsflaSet::new();
        assert_eq!(dut.name, "UBX-CFG-ESFLA");
        dut.data.leverarm_type = 2;
        dut.data.leverarm_x = 127;
        dut.data.leverarm_y = 255;
        dut.data.leverarm_z = 1000;

        let msg = dut.to_bin();
        println!("message {:?}", msg);
        assert_eq!(msg, [0xb5, 0x62, 0x06, 0x2F, 12, 0, 0x00, 0x01, 0x00, 0x00, 2, 0x00, 127, 0, 255, 0, 0xe8, 0x03, 173, 173]);
    }

    #[test]
    fn negative_values() {
        let mut dut = UbxCfgEsflaSet::new();
        assert_eq!(dut.name, "UBX-CFG-ESFLA");
        dut.data.leverarm_type = 3;
        dut.data.leverarm_x = -127;
        dut.data.leverarm_y = -255;
        dut.data.leverarm_z = -1000;

        let msg = dut.to_bin();
        println!("message {:?}", msg);
        assert_eq!(msg, [0xb5, 0x62, 0x06, 0x2F, 12, 0, 0x00, 0x01, 0x00, 0x00, 3, 0x00, 129, 255, 1, 255, 24, 252, 217, 26]);
    }
}
