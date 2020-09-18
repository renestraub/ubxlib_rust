use serde::{Serialize};

use crate::cid::UbxCID;
use crate::frame::{UbxFrame, UbxFrameInfo, UbxFrameSerialize};


const CLS: u8 = 0x06;
const ID: u8 = 0x04;

/*
TODO: Enum is serialized as four bytes, thus not working here
See serde customization
https://serde.rs/container-attrs.html
https://serde.rs/enum-representations.html#untagged

#[repr(u16)]
#[derive(Serialize, Debug)]
pub enum BbrMask {
    HOT_START = 0x0000,
    WARM_START = 0x0001,
    COLD_START = 0xFFFF,
}

impl Default for BbrMask {
    fn default() -> Self { BbrMask::HOT_START }
}


#[repr(u8)]
#[derive(Serialize, Debug)]
pub enum ResetMode {
    IMMEDIATE_HW_RESET = 0x00,
    SW_RESET = 0x01,
    HW_RESET = 0x04,
    STOP = 0x08,
    START = 0x09,
}

impl Default for ResetMode {
    fn default() -> Self { ResetMode::IMMEDIATE_HW_RESET }
}
*/

const HOT_START: u16 = 0x0000;
// const WARM_START: u16 = 0x0001;
const COLD_START: u16 = 0xFFFF;

// const IMMEDIATE_HW_RESET: u8 = 0x00;
const SW_RESET: u8 = 0x01;
// const HW_RESET: u8 = 0x04;
const STOP: u8 = 0x08;
// const START: u8 = 0x09;


#[derive(Default, Debug, Serialize)]
pub struct Data {
    // pub nav_bbr_mask: BbrMask,
    // pub reset_mode: ResetMode,
    pub nav_bbr_mask: u16,
    pub reset_mode: u8,
    pub res1: u8,
}

impl Data {
    pub fn new(mask: u16, reset_mode: u8) -> Self {
        Self {
            nav_bbr_mask: mask,
            reset_mode: reset_mode,
            ..Default::default()
        }
    }
}


#[derive(Default, Debug)]
pub struct UbxCfgRstAction {
    pub name: &'static str,
    cid: UbxCID,
    pub data: Data,
}

impl UbxCfgRstAction {
    pub fn cold_start() -> Self {
        Self {
            name: "UBX-CFG-RST",
            cid: UbxCID::new(CLS, ID),
            data: Data::new(COLD_START, SW_RESET),
        }
    }

    pub fn stop() -> Self {
        Self {
            name: "UBX-CFG-RST",
            cid: UbxCID::new(CLS, ID),
            data: Data::new(HOT_START, STOP),
        }
    }


    // TODO: Realize the following as constructors
    // simper to use, just a bit more code here
/*
    pub fn warm_start(&mut self) {
        self.data.reset_mode = SW_RESET;
        self.data.nav_bbr_mask = WARM_START;
    }
*/

/*
    pub fn start(&mut self) {
        self.data.reset_mode = START;
        self.data.nav_bbr_mask = HOT_START;
    }

    pub fn stop(&mut self) {
        self.data.reset_mode = STOP;
        self.data.nav_bbr_mask = HOT_START;
    }
*/

    fn save(&self) -> Vec<u8> {
        let data = bincode::serialize(&self.data).unwrap();
        println!("{:?}", data);
        assert!(data.len() == 4);
        data
    }
}

impl UbxFrameInfo for UbxCfgRstAction {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxCfgRstAction {
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
    fn cold_start() {
        let dut = UbxCfgRstAction::cold_start();
        let msg = dut.to_bin();
        // println!("message {:?}", msg);
        assert_eq!(msg, [0xb5, 0x62, 0x06, 0x04, 4, 0,  0xFF, 0xFF, 0x01, 0, 13, 95]);
    }

    #[test]
    fn stop() {
        let dut = UbxCfgRstAction::stop();
        let msg = dut.to_bin();
        println!("message {:?}", msg);
        assert_eq!(msg, [0xb5, 0x62, 0x06, 0x04, 4, 0,  0x00, 0x00, 0x08, 0, 22, 116]);
    }
}
