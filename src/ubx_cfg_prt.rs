use serde::{Deserialize, Serialize};

use crate::cid::UbxCID;
use crate::frame::UbxFrameWithData;

const CLS: u8 = 0x06;
const ID: u8 = 0x00;

const PORTID_UART: u8 = 1;

#[derive(Default, Debug, Serialize)]
pub struct DataCfgPrtPoll {
    pub port_id: u8,
}

impl DataCfgPrtPoll {
    pub fn new() -> Self {
        Self {
            port_id: PORTID_UART,
        }
    }
}

pub struct UbxCfgPrtPoll {}

impl UbxCfgPrtPoll {
    pub fn new() -> UbxFrameWithData<DataCfgPrtPoll> {
        UbxFrameWithData::init(
            "UBX-CFG-PRT-POLL",
            UbxCID::new(CLS, ID),
            DataCfgPrtPoll::new(),
        )
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct DataCfgPrt {
    pub port_id: u8,
    pub res1: u8,
    pub tx_ready: u16,
    pub mode: u32,
    pub baudrate: u32,

    pub in_proto_mask: u16,
    pub out_proto_mask: u16,
    pub flags: u16,
    pub res2: [u8; 2],
}

pub struct UbxCfgPrtUart {}

impl UbxCfgPrtUart {
    pub fn new() -> UbxFrameWithData<DataCfgPrt> {
        UbxFrameWithData::new("UBX-CFG-PRT", UbxCID::new(CLS, ID))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{UbxFrameDeSerialize, UbxFrameSerialize};

    #[test]
    fn poll() {
        let dut = UbxCfgPrtPoll::new();
        assert_eq!(dut.name, "UBX-CFG-PRT-POLL");
        assert_eq!(dut.data.port_id, 1);
        let msg = dut.to_bin();
        assert_eq!(msg, [0xb5, 0x62, 0x06, 0x00, 1, 0, 1, 8, 34]);
    }

    #[test]
    fn deserialize() {
        const DATA: [u8; 20] = [
            1, 0, 0, 0, 192, 8, 0, 0, 128, 37, 0, 0, 7, 0, 3, 0, 0, 0, 0, 0,
        ];
        let mut dut = UbxCfgPrtUart::new();
        assert_eq!(dut.name, "UBX-CFG-PRT");
        dut.from_bin(&DATA);

        assert_eq!(dut.data.port_id, 1);
        assert_eq!(dut.data.baudrate, 9600);
        assert_eq!(dut.data.in_proto_mask, 7);
        assert_eq!(dut.data.out_proto_mask, 3);
    }

    #[test]
    fn set() {
        let mut dut = UbxCfgPrtUart::new();
        assert_eq!(dut.name, "UBX-CFG-PRT");
        dut.data.baudrate = 115200;
        let msg = dut.to_bin();
        assert_eq!(msg[0..6], [0xb5, 0x62, 0x06, 0x00, 20, 0]);
        assert_eq!(msg[6 + 8..6 + 12], [0, 194, 1, 0]);
    }
}
