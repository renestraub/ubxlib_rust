use serde::{Deserialize, Serialize};

use crate::cid::UbxCID;
use crate::frame::{UbxFrame, UbxFrameDeSerialize, UbxFrameInfo, UbxFrameSerialize};

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
        UbxFrame::bytes(UbxCID::new(CLS, ID), [].to_vec())
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
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
        // let data = self.save();
        let data = bincode::serialize(&self.data).unwrap();
        UbxFrame::bytes(UbxCID::new(CLS, ID), data)
    }
}

impl UbxFrameDeSerialize for UbxCfgNav5 {
    fn from_bin(&mut self, data: Vec<u8>) {
        assert_eq!(data.len(), 36);
        self.data = bincode::deserialize(&data).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poll() {
        let dut = UbxCfgNav5Poll::new();
        assert_eq!(dut.name, "UBX-CFG-NAV5-POLL");
        let msg = dut.to_bin();
        assert_eq!(msg, [0xb5, 0x62, CLS, ID, 0, 0, 42, 132]);
    }

    #[test]
    fn serialize_deser() {
        let mut dut = UbxCfgNav5::new();
        assert_eq!(dut.name, "UBX-CFG-NAV5");

        dut.data.mask = 0x1122;
        dut.data.dyn_model = 4;
        dut.data.fix_mode = 2;
        dut.data.utc_standard = 3;

        let res = dut.to_bin();
        assert_eq!(res.len(), 36 + 8);
        assert_eq!(res[6 + 2], 4);
        assert_eq!(res[6 + 3], 2);
        assert_eq!(res[6 + 30], 3);

        // invalidate data, to check load() is actually working
        dut.data.mask = 0x0000;
        dut.data.dyn_model = 0;
        dut.data.fix_mode = 0;
        dut.data.utc_standard = 0;

        dut.from_bin(res[6..42].to_vec());
        assert_eq!(dut.data.mask, 0x1122);
        assert_eq!(dut.data.dyn_model, 4);
        assert_eq!(dut.data.fix_mode, 2);
        assert_eq!(dut.data.utc_standard, 3);
    }
}
