// use std::fmt;

//extern crate bincode;
//use bincode::{serialize, deserialize};
use serde::{Serialize, Deserialize};

use crate::cid::UbxCID;
use crate::frame::{UbxFrame, UbxFrameInfo, UbxFrameSerialize, UbxFrameDeSerialize};


const CLS: u8 = 0x06;
const ID: u8 = 0x24;

pub struct UbxCfgNav5Poll {
    pub name: &'static str,
    cid: UbxCID,
}


impl UbxCfgNav5Poll {
    pub fn new() -> Self {
        Self {
            name: "UBX-CFG-NAV5-POLL",
            cid: UbxCID::new(CLS, ID), 
        }
    }
}

impl UbxFrameInfo for UbxCfgNav5Poll {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxCfgNav5Poll {
    fn to_bin(&self) -> Vec<u8> {
        let frame = UbxFrame::construct(UbxCID::new(CLS, ID), [].to_vec());
        let msg = frame.to_bytes();
        msg
    }
}

#[derive(Default)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub mask: u16,
    pub dyn_model: u8,
    pub fix_mode: u8,
    pub fixed_alt: i32,
    pub fixed_alt_var: u32,
    pub min_elev: i8,
    pub dr_limit: u8,
    pub pdop: u16,
    pub tdop: u16,
    pub pacc: u16,
    pub tacc: u16,
    pub static_hold_thresh: u8,
    pub dgps_timeout: u8,
    pub cno_thresh_num_svs: u8,
    pub cno_thresh: u8,
    pub pacc_adr: u16,
    pub static_hold_max_dist: u16,
    pub utc_standard: u8,
    pub res: [u8; 5],
}

#[derive(Default, Debug)]
pub struct UbxCfgNav5 {
    pub name: &'static str,
    cid: UbxCID,
    pub data: Data,
}

impl UbxCfgNav5 {
    pub fn new() -> Self {
        Self { 
            name: "UBX-CFG-NAV5",
            cid: UbxCID::new(CLS, ID), 
            ..Default::default()
        }
    }

    // TODO: Remove, write directly in UbxFrameDeSerialize
    pub fn load(&mut self, data: &[u8]) {
        assert!(data.len() == 36);
        self.data = bincode::deserialize(&data).unwrap();
        // println!("Decoded struct is {:?}", self.data);
    }

    // TODO: Remove, write directly in UbxFrameSerialize
    pub fn save(&self) -> Vec<u8> {
        let data = bincode::serialize(&self.data).unwrap();
        data
    }
}

impl UbxFrameInfo for UbxCfgNav5 {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxCfgNav5 {
    fn to_bin(&self) -> Vec<u8> {
        let data = self.save();

        // construct a frame with correct CID and payload
        let frame = UbxFrame::construct(UbxCID::new(CLS, ID), data);
        let msg = frame.to_bytes();
        msg
        // TODO: Combine to one statement
    }
}

impl UbxFrameDeSerialize for UbxCfgNav5 {
    fn from_bin(&mut self, data: Vec<u8>) {
        self.load(&data);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cfg_rate_poll() {
        let dut = UbxCfgNav5Poll::new();
        assert_eq!(dut.name, "UBX-CFG-NAV5-POLL");
        let msg = dut.to_bin();
        assert_eq!(msg, [0xb5, 0x62, CLS, ID, 0, 0, 42, 132]);
    }

    #[test]
    fn cfg_rate_ser_des() {
        let mut dut = UbxCfgNav5::new();
        assert_eq!(dut.name, "UBX-CFG-NAV5");

        dut.data.mask = 0x1122;
        dut.data.dyn_model = 4;
        dut.data.fix_mode = 2;
        dut.data.utc_standard = 3;

        let res = dut.save();
        // println!("Serialized Data is {} {:?}", res.len(), res);
        assert_eq!(res.len(), 36);

        // invalidate data, to check load() is actually working
        dut.data.mask = 0x0000;
        dut.data.dyn_model = 0;
        dut.data.fix_mode = 0;
        dut.data.utc_standard = 0;

        dut.load(&res);
        assert_eq!(dut.data.mask, 0x1122);
        assert_eq!(dut.data.dyn_model, 4);
        assert_eq!(dut.data.fix_mode, 2);
        assert_eq!(dut.data.utc_standard, 3);
    }
}
