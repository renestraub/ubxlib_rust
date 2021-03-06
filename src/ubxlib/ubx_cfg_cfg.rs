use serde::Serialize;

use crate::ubxlib::cid::UbxCID;
use crate::ubxlib::frame::UbxFrameWithData;

const CLS: u8 = 0x06;
const ID: u8 = 0x09;

const MASK_ALL: u32 = 0x00001F1F;

#[derive(Default, Debug, Serialize)]
pub struct DataCfgCfg {
    pub clear_mask: u32,
    pub save_mask: u32,
    pub load_mask: u32,
}

impl DataCfgCfg {
    pub fn from(clear_mask: u32, save_mask: u32, load_mask: u32) -> Self {
        Self {
            clear_mask,
            save_mask,
            load_mask,
        }
    }
}

pub struct UbxCfgCfgAction {}

impl UbxCfgCfgAction {
    pub fn factory_reset() -> UbxFrameWithData<DataCfgCfg> {
        UbxFrameWithData::init(
            "UBX-CFG-CFG",
            UbxCID::new(CLS, ID),
            DataCfgCfg::from(MASK_ALL, 0, MASK_ALL),
        )
    }

    pub fn persist() -> UbxFrameWithData<DataCfgCfg> {
        UbxFrameWithData::init(
            "UBX-CFG-CFG",
            UbxCID::new(CLS, ID),
            DataCfgCfg::from(0, MASK_ALL, 0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ubxlib::frame::UbxFrameSerialize;

    #[test]
    fn factory_reset() {
        let dut = UbxCfgCfgAction::factory_reset();
        assert_eq!(dut.name, "UBX-CFG-CFG");
        let msg = dut.to_bin();
        assert_eq!(
            msg[0..18],
            [
                0xb5, 0x62, 0x06, 0x09, 12, 0, 0x1f, 0x1f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x1f, 0x1f, 0x00, 0x00
            ]
        );
    }

    #[test]
    fn persist() {
        let dut = UbxCfgCfgAction::persist();
        assert_eq!(dut.name, "UBX-CFG-CFG");
        let msg = dut.to_bin();
        assert_eq!(
            msg[..18],
            [
                0xb5, 0x62, 0x06, 0x09, 12, 0, 0x00, 0x00, 0x00, 0x00, 0x1f, 0x1f, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00
            ]
        );
    }
}
