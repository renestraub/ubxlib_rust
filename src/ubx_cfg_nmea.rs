use serde::{Deserialize, Serialize};

use crate::cid::UbxCID;
use crate::frame::{UbxFrame, UbxFrameDeSerialize, UbxFrameInfo, UbxFrameSerialize};

const CLS: u8 = 0x06;
const ID: u8 = 0x17;

pub struct UbxCfgNmeaPoll {
    pub name: &'static str,
    cid: UbxCID,
}

impl UbxCfgNmeaPoll {
    pub fn new() -> Self {
        Self {
            name: "UBX-CFG-NMEA-POLL",
            cid: UbxCID::new(CLS, ID),
        }
    }
}

impl UbxFrameInfo for UbxCfgNmeaPoll {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxCfgNmeaPoll {
    fn to_bin(&self) -> Vec<u8> {
        UbxFrame::bytes(UbxCID::new(CLS, ID), [].to_vec())
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Data {
    pub filter: u8,
    pub nmea_version: u8,
    pub num_sv: u8,
    pub flags: u8,
    pub gnss_to_filter: u32,
    pub sv_numbering: u8,
    pub main_talker_id: u8,
    pub gsv_talker_id: u8,
    pub version: u8,
    pub bds_talker_id: [u8; 2],
    pub res1: [u8; 6],
}

#[derive(Default, Debug)]
pub struct UbxCfgNmea {
    pub name: &'static str,
    cid: UbxCID,
    pub data: Data,
}

impl UbxCfgNmea {
    pub fn new() -> Self {
        Self {
            name: "UBX-CFG-NMEA",
            cid: UbxCID::new(CLS, ID),
            ..Default::default()
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        assert!(data.len() == 20);
        self.data = bincode::deserialize(&data).unwrap();
    }
}

impl UbxFrameInfo for UbxCfgNmea {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxCfgNmea {
    fn to_bin(&self) -> Vec<u8> {
        let data = bincode::serialize(&self.data).unwrap();
        assert_eq!(data.len(), 20);
        UbxFrame::bytes(UbxCID::new(CLS, ID), data)
    }
}

impl UbxFrameDeSerialize for UbxCfgNmea {
    fn from_bin(&mut self, data: Vec<u8>) {
        self.load(&data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poll() {
        let dut = UbxCfgNmeaPoll::new();
        assert_eq!(dut.name, "UBX-CFG-NMEA-POLL");
        let msg = dut.to_bin();
        assert_eq!(msg, [0xb5, 0x62, 0x06, 0x17, 0, 0, 29, 93]);
    }

    #[test]
    fn set() {
        let mut dut = UbxCfgNmea::new();
        assert_eq!(dut.name, "UBX-CFG-NMEA");
        dut.data.nmea_version = 0x41;
        let msg = dut.to_bin();
        assert_eq!(msg[6 + 1], 0x41);
    }
}
