use serde::{Deserialize, Serialize};

use crate::ubxlib::cid::UbxCID;
use crate::ubxlib::frame::{UbxFramePoll, UbxFrameWithData};

const CLS: u8 = 0x06;
const ID: u8 = 0x17;

pub struct UbxCfgNmeaPoll {}

impl UbxCfgNmeaPoll {
    pub fn create() -> UbxFramePoll {
        UbxFramePoll::new("UBX-CFG-NMEA-POLL", UbxCID::new(CLS, ID))
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct DataCfgNmea {
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

pub struct UbxCfgNmea {}

impl UbxCfgNmea {
    pub fn create() -> UbxFrameWithData<DataCfgNmea> {
        UbxFrameWithData::new("UBX-CFG-NMEA", UbxCID::new(CLS, ID))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ubxlib::frame::{UbxFrameDeSerialize, UbxFrameSerialize};

    #[test]
    fn poll() {
        let dut = UbxCfgNmeaPoll::create();
        assert_eq!(dut.name, "UBX-CFG-NMEA-POLL");
        let msg = dut.to_bin();
        assert_eq!(msg, [0xb5, 0x62, 0x06, 0x17, 0, 0, 29, 93]);
    }

    #[test]
    fn deserialize() {
        const DATA: [u8; 20] = [0, 64, 0, 2, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut dut = UbxCfgNmea::create();
        dut.from_bin(&DATA);

        assert_eq!(dut.data.nmea_version, 0x40);
    }

    #[test]
    fn set() {
        let mut dut = UbxCfgNmea::create();
        assert_eq!(dut.name, "UBX-CFG-NMEA");
        dut.data.nmea_version = 0x41;
        let msg = dut.to_bin();
        assert_eq!(msg[6 + 1], 0x41);
    }
}
