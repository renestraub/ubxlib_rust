use serde::Serialize;

use crate::cid::UbxCID;
use crate::frame::UbxFrameWithData;

const CLS: u8 = 0x06;
const ID: u8 = 0x09;

const MASK_ALL: u32 = 0x00001F1F;

#[derive(Default, Debug, Serialize)]
pub struct Data {
    pub clear_mask: u32,
    pub save_mask: u32,
    pub load_mask: u32,
}

impl Data {
    pub fn new(clear: u32, save: u32, load: u32) -> Self {
        Self {
            clear_mask: clear,
            save_mask: save,
            load_mask: load,
            ..Default::default()
        }
    }
}

pub struct UbxCfgCfgAction { }

impl UbxCfgCfgAction { 
    pub fn factory_reset() -> UbxFrameWithData<Data> {
        UbxFrameWithData::init("UBX-CFG-CFG", UbxCID::new(CLS, ID), Data::new(MASK_ALL, 0, MASK_ALL))
    }

    pub fn persist() -> UbxFrameWithData<Data> {
        UbxFrameWithData::init("UBX-CFG-CFG", UbxCID::new(CLS, ID), Data::new(0, MASK_ALL, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::UbxFrameSerialize;

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
