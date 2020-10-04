use serde::Serialize;

use crate::cid::UbxCID;
use crate::frame::UbxFrameWithData;

const CLS: u8 = 0x09;
const ID: u8 = 0x14;


// TODO: UpdSosAction Enum

#[derive(Default, Debug, Serialize)]
pub struct UpdSosAction {
    pub cmd: u8,
    pub res1: [u8; 3],
}

impl UpdSosAction {
    pub fn new(cmd: u8) -> Self {
        Self {
            cmd,
            ..Default::default()
        }
    }
}

pub struct UbxUpdSosAction { }

impl UbxUpdSosAction { 
    pub fn backup() -> UbxFrameWithData<UpdSosAction> {
        UbxFrameWithData::init("UBX-UPD-SOS-ACTION", UbxCID::new(CLS, ID), UpdSosAction::new(0x00))
    }

    pub fn clear() -> UbxFrameWithData<UpdSosAction> {
        UbxFrameWithData::init("UBX-UPD-SOS-ACTION", UbxCID::new(CLS, ID), UpdSosAction::new(0x01))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::UbxFrameSerialize;

    #[test]
    fn backup() {
        let dut = UbxUpdSosAction::backup();
        let msg = dut.to_bin();
        assert_eq!(msg[..10], [0xb5, 0x62, 0x09, 0x14, 4, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn clear() {
        let dut = UbxUpdSosAction::clear();
        let msg = dut.to_bin();
        assert_eq!(msg[..10], [0xb5, 0x62, 0x09, 0x14, 4, 0, 1, 0, 0, 0]);
    }
}
