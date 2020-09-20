use serde::{Deserialize, Serialize};

use crate::cid::UbxCID;
use crate::frame::{UbxFrame, UbxFrameDeSerialize, UbxFrameInfo, UbxFrameSerialize};

const CLS: u8 = 0x06;
const ID: u8 = 0x56;

pub struct UbxCfgEsfAlgPoll {
    pub name: &'static str,
    cid: UbxCID,
}

impl UbxCfgEsfAlgPoll {
    pub fn new() -> Self {
        Self {
            name: "UBX-CFG-ESFALG-POLL",
            cid: UbxCID::new(CLS, ID),
        }
    }
}

impl UbxFrameInfo for UbxCfgEsfAlgPoll {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxCfgEsfAlgPoll {
    fn to_bin(&self) -> Vec<u8> {
        UbxFrame::bytes(UbxCID::new(CLS, ID), [].to_vec())
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Data {
    pub bitfield: u32, // u-blox describes as U4, bit is X4
    pub yaw: u32,      // 1e-2, 0..360°
    pub pitch: i16,    // 1e-2, -90..90°
    pub roll: i16,     // 1e-2, -180..180°
}

#[derive(Default, Debug)]
pub struct UbxCfgEsfAlg {
    pub name: &'static str,
    cid: UbxCID,
    pub data: Data,
}

impl UbxCfgEsfAlg {
    pub fn new() -> Self {
        Self {
            name: "UBX-CFG-ESFALG",
            cid: UbxCID::new(CLS, ID),
            ..Default::default()
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        assert!(data.len() == 12);
        self.data = bincode::deserialize(&data).unwrap();
    }
}

impl UbxFrameInfo for UbxCfgEsfAlg {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxCfgEsfAlg {
    fn to_bin(&self) -> Vec<u8> {
        let data = bincode::serialize(&self.data).unwrap();
        UbxFrame::bytes(UbxCID::new(CLS, ID), data)
    }
}

impl UbxFrameDeSerialize for UbxCfgEsfAlg {
    fn from_bin(&mut self, data: Vec<u8>) {
        self.load(&data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poll() {
        let dut = UbxCfgEsfAlgPoll::new();
        assert_eq!(dut.name, "UBX-CFG-ESFALG-POLL");
        let msg = dut.to_bin();
        assert_eq!(msg, [0xb5, 0x62, 0x06, 0x56, 0, 0, 92, 26]);
    }

    #[test]
    #[should_panic]
    fn too_few_values() {
        const DATA: [u8; 5] = [0xe8, 0x03, 0x01, 0x00, 0x34];
        let mut dut = UbxCfgEsfAlg::new();
        dut.load(&DATA.to_vec());
    }

    #[test]
    fn load() {
        const DATA: [u8; 12] = [
            0xff, 0xfe, 0xfd, 0xfc, 0x04, 0x03, 0x02, 0x01, 0x08, 0x07, 0x06, 0x05,
        ];
        let mut dut = UbxCfgEsfAlg::new();
        dut.load(&DATA.to_vec());

        assert_eq!(dut.data.bitfield, 0xfcfdfeffu32);
        assert_eq!(dut.data.yaw, 0x01020304u32);
        assert_eq!(dut.data.pitch, 0x0708i16);
        assert_eq!(dut.data.roll, 0x0506i16);
    }
}
