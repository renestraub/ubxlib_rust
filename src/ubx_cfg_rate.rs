use serde::{Deserialize, Serialize};

use crate::cid::UbxCID;
use crate::frame::UbxFrameWithData;

const CLS: u8 = 0x06;
const ID: u8 = 0x08;

#[derive(Default, Debug, Serialize)]
pub struct DataPoll {}

pub struct UbxCfgRatePoll {}

impl UbxCfgRatePoll {
    pub fn new() -> UbxFrameWithData<DataPoll> {
        UbxFrameWithData::new("UBX-CFG-RATE-POLL", UbxCID::new(CLS, ID))
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Data {
    pub meas_rate: u16, // Time elapsed between two measuremnts in ms
    pub nav_rate: u16,  // Number of measurements for NAV solution
    pub time_ref: u16,
}

pub struct UbxCfgRate {}

impl UbxCfgRate {
    pub fn new() -> UbxFrameWithData<Data> {
        UbxFrameWithData::new("UBX-CFG-RATE", UbxCID::new(CLS, ID))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{UbxFrameDeSerialize, UbxFrameSerialize};

    #[test]
    fn poll() {
        let dut = UbxCfgRatePoll::new();
        assert_eq!(dut.name, "UBX-CFG-RATE-POLL");
        let msg = dut.to_bin();
        assert_eq!(msg, [0xb5, 0x62, 0x06, 0x08, 0, 0, 14, 48]);
    }

    #[test]
    fn deserialize() {
        const DATA: [u8; 6] = [0xe8, 0x03, 0x01, 0x00, 0x34, 0x12];
        let mut dut = UbxCfgRate::new();
        assert_eq!(dut.name, "UBX-CFG-RATE");
        dut.from_bin(&DATA);

        assert_eq!(dut.data.meas_rate, 1000);
        assert_eq!(dut.data.nav_rate, 1);
        assert_eq!(dut.data.time_ref, 0x1234);
    }

    #[test]
    #[should_panic]
    fn deser_too_few_values() {
        const DATA: [u8; 5] = [0xe8, 0x03, 0x01, 0x00, 0x34];
        let mut dut = UbxCfgRate::new();
        dut.from_bin(&DATA);

        assert_eq!(dut.data.meas_rate, 1000);
        assert_eq!(dut.data.nav_rate, 1);
        assert_eq!(dut.data.time_ref, 0x1234);
    }

    #[test]
    fn modify() {
        let mut dut = UbxCfgRate::new();
        assert_eq!(dut.data.meas_rate, 0);
        assert_eq!(dut.data.nav_rate, 0);
        assert_eq!(dut.data.time_ref, 0);

        dut.data.meas_rate = 1000;
        dut.data.nav_rate = 1;
        dut.data.time_ref = 0x1234;
        let data = dut.to_bin();
        assert_eq!(data[6..12], [0xE8, 3, 1, 0, 0x34, 0x12]);
    }

    #[test]
    fn serialize() {
        let mut dut = UbxCfgRate::new();
        assert_eq!(dut.data.meas_rate, 0);
        assert_eq!(dut.data.nav_rate, 0);
        assert_eq!(dut.data.time_ref, 0);

        dut.data.meas_rate = 1000;
        dut.data.nav_rate = 1;
        dut.data.time_ref = 0x1234;

        let data = dut.to_bin();
        assert_eq!(
            data,
            [0xb5, 0x62, 0x06, 0x08, 0x06, 0x00, 0xE8, 0x03, 0x01, 0x00, 0x34, 0x12, 70, 177]
        );
    }
}
