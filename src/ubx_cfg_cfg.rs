use serde::{Serialize};

use crate::cid::UbxCID;
use crate::frame::{UbxFrame, UbxFrameInfo, UbxFrameSerialize};


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
    // TODO: Remove load mask as not needed so far
    pub fn new(clear: u32, save: u32, load: u32) -> Self {
        Self {
            clear_mask: clear,
            save_mask: save,
            load_mask: load,
            ..Default::default()
        }
    }
}


#[derive(Default, Debug)]
pub struct UbxCfgCfgAction {
    pub name: &'static str,
    cid: UbxCID,
    pub data: Data,
}

impl UbxCfgCfgAction {
    pub fn factory_reset() -> Self {
        Self {
            name: "UBX-CFG-CFG",
            cid: UbxCID::new(CLS, ID),
            data: Data::new(MASK_ALL, 0, MASK_ALL),
        }
    }

    pub fn persist() -> Self {
        Self {
            name: "UBX-CFG-RST",
            cid: UbxCID::new(CLS, ID),
            data: Data::new(0, MASK_ALL, 0),
        }
    }

    fn save(&self) -> Vec<u8> {
        let data = bincode::serialize(&self.data).unwrap();
        assert!(data.len() == 12);
        data
    }
}

impl UbxFrameInfo for UbxCfgCfgAction {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxCfgCfgAction {
    fn to_bin(&self) -> Vec<u8> {
        // update binary data in frame
        let data = self.save();

        // construct a frame with correct CID and payload
        let frame = UbxFrame::construct(UbxCID::new(CLS, ID), data);
        let msg = frame.to_bytes();
        msg
        // TODO: Combine to one statement
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_reset() {
        let dut = UbxCfgCfgAction::factory_reset();
        let msg = dut.to_bin();
        assert_eq!(msg[0..18], [0xb5, 0x62, 0x06, 0x09, 12, 0, 0x1f, 0x1f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1f, 0x1f, 0x00, 0x00]);
        //                      B5      62    06    09  0D 00  FF      FB    00    00    00    00    00    00    FF    FF    00    00   17  2B 7E
    }

    #[test]
    fn persist() {
        let dut = UbxCfgCfgAction::persist();
        let msg = dut.to_bin();
        assert_eq!(msg[..18], [0xb5, 0x62, 0x06, 0x09, 12, 0, 0x00, 0x00, 0x00, 0x00, 0x1f, 0x1f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        //                       B5    62    06    09  0D 00    00    00    00    00    FF    FF    00    00    00    00    00    00   17   31 BF
    }
}
