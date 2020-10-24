use serde::{Deserialize, Serialize};

use crate::ubxlib::cid::UbxCID;
use crate::ubxlib::frame::{UbxFramePoll, UbxFrameWithData};

const CLS: u8 = 0x06;
const ID: u8 = 0x24;

pub struct UbxCfgNav5Poll {}

impl UbxCfgNav5Poll {
    pub fn create() -> UbxFramePoll {
        UbxFramePoll::new("UBX-CFG-NAV5-POLL", UbxCID::new(CLS, ID))
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct DataCfgNav5 {
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

pub struct UbxCfgNav5 {}

impl UbxCfgNav5 {
    pub fn create() -> UbxFrameWithData<DataCfgNav5> {
        UbxFrameWithData::new("UBX-CFG-NAV5", UbxCID::new(CLS, ID))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ubxlib::frame::{UbxFrameDeSerialize, UbxFrameSerialize};

    #[test]
    fn poll() {
        let dut = UbxCfgNav5Poll::create();
        assert_eq!(dut.name, "UBX-CFG-NAV5-POLL");
        let msg = dut.to_bin();
        assert_eq!(msg, [0xb5, 0x62, CLS, ID, 0, 0, 42, 132]);
    }

    #[test]
    fn serialize_deser() {
        let mut dut = UbxCfgNav5::create();
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

        dut.from_bin(&res[6..42]);
        assert_eq!(dut.data.mask, 0x1122);
        assert_eq!(dut.data.dyn_model, 4);
        assert_eq!(dut.data.fix_mode, 2);
        assert_eq!(dut.data.utc_standard, 3);
    }

    #[test]
    fn deser() {
        const DATA: [u8; 36] = [
            255, 255, 4, 3, 0, 0, 0, 0, 16, 39, 0, 0, 10, 0, 250, 0, 250, 0, 100, 0, 94, 1, 0, 60,
            0, 0, 16, 39, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let mut dut = UbxCfgNav5::create();
        assert_eq!(dut.name, "UBX-CFG-NAV5");

        dut.from_bin(&DATA);
        assert_eq!(dut.data.dyn_model, 4);
        assert_eq!(dut.data.fix_mode, 3);
        assert_eq!(dut.data.pdop, 250);
        assert_eq!(dut.data.cno_thresh_num_svs, 0);
        assert_eq!(dut.data.cno_thresh, 0);
        assert_eq!(dut.data.utc_standard, 0);
    }
}
