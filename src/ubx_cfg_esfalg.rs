use serde::{Deserialize, Serialize};

use crate::cid::UbxCID;
use crate::frame::UbxFrameWithData;

const CLS: u8 = 0x06;
const ID: u8 = 0x56;

#[derive(Default, Debug, Serialize)]
pub struct DataPoll {}

pub struct UbxCfgEsfAlgPoll {}

impl UbxCfgEsfAlgPoll {
    pub fn new() -> UbxFrameWithData<DataPoll> {
        UbxFrameWithData::new("UBX-CFG-ESFALG-POLL", UbxCID::new(CLS, ID))
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Data {
    pub bitfield: u32, // u-blox describes as U4, bit is X4
    pub yaw: u32,      // 1e-2, 0..360°
    pub pitch: i16,    // 1e-2, -90..90°
    pub roll: i16,     // 1e-2, -180..180°
}

pub struct UbxCfgEsfAlg {}

impl UbxCfgEsfAlg {
    pub fn new() -> UbxFrameWithData<Data> {
        UbxFrameWithData::new("UBX-CFG-ESFALG", UbxCID::new(CLS, ID))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{UbxFrameDeSerialize, UbxFrameSerialize};

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
        dut.from_bin(DATA.to_vec());
    }

    #[test]
    fn load() {
        const DATA: [u8; 12] = [
            0xff, 0xfe, 0xfd, 0xfc, 0x04, 0x03, 0x02, 0x01, 0x08, 0x07, 0x06, 0x05,
        ];
        let mut dut = UbxCfgEsfAlg::new();
        dut.from_bin(DATA.to_vec());

        assert_eq!(dut.data.bitfield, 0xfcfdfeffu32);
        assert_eq!(dut.data.yaw, 0x01020304u32);
        assert_eq!(dut.data.pitch, 0x0708i16);
        assert_eq!(dut.data.roll, 0x0506i16);
    }

    #[test]
    fn serialize() {
        let mut dut = UbxCfgEsfAlg::new();
        assert_eq!(dut.name, "UBX-CFG-ESFALG");
        dut.data.yaw = 180 as u32 * 100;
        dut.data.pitch = -45 as i16 * 100;
        dut.data.roll = 45 as i16 * 100;

        let data = dut.to_bin();
        assert_eq!(
            data[0..18],
            [
                0xb5, 0x62, 0x06, 0x56, 12, 0, 0x00, 0x00, 0x00, 0x00, 0x50, 0x46, 0x00, 0x00,
                0x6C, 0xEE, 0x94, 0x11
            ]
        );
    }
}
