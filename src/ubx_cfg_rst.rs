use serde::Serialize;

use crate::cid::UbxCID;
use crate::frame::{UbxFrame, UbxFrameInfo, UbxFrameSerialize};

const CLS: u8 = 0x06;
const ID: u8 = 0x04;

use serde_repr::Serialize_repr;

#[derive(Serialize_repr, Debug)]
#[repr(u16)]
pub enum BbrMask {
    HotStart = 0x0000,
    // WarmStart = 0x0001,
    ColdStart = 0xFFFF,
}

impl Default for BbrMask {
    fn default() -> Self {
        BbrMask::ColdStart
    }
}

#[derive(Serialize_repr, Debug)]
#[repr(u8)]
pub enum ResetMode {
    ImmediateHwReset = 0x00,
    SwReset = 0x01,
    // HwReset = 0x04,
    Stop = 0x08,
    // Start = 0x09,
}

impl Default for ResetMode {
    fn default() -> Self {
        ResetMode::ImmediateHwReset
    }
}

#[derive(Default, Debug, Serialize)]
pub struct Data {
    pub nav_bbr_mask: BbrMask,
    pub reset_mode: ResetMode,
    pub res1: u8,
}

impl Data {
    pub fn new(mask: BbrMask, reset_mode: ResetMode) -> Self {
        Self {
            nav_bbr_mask: mask,
            reset_mode: reset_mode,
            ..Default::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct UbxCfgRstAction {
    pub name: &'static str,
    cid: UbxCID,
    pub data: Data,
}

impl UbxCfgRstAction {
    pub fn cold_start() -> Self {
        Self {
            name: "UBX-CFG-RST",
            cid: UbxCID::new(CLS, ID),
            data: Data::new(BbrMask::ColdStart, ResetMode::SwReset),
        }
    }

    pub fn stop() -> Self {
        Self {
            name: "UBX-CFG-RST",
            cid: UbxCID::new(CLS, ID),
            data: Data::new(BbrMask::HotStart, ResetMode::Stop),
        }
    }
}

impl UbxFrameInfo for UbxCfgRstAction {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxCfgRstAction {
    fn to_bin(&self) -> Vec<u8> {
        let data = bincode::serialize(&self.data).unwrap();
        assert_eq!(data.len(), 4);
        UbxFrame::bytes(UbxCID::new(CLS, ID), data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cold_start() {
        let dut = UbxCfgRstAction::cold_start();
        let msg = dut.to_bin();
        assert_eq!(
            msg,
            [0xb5, 0x62, 0x06, 0x04, 4, 0, 0xFF, 0xFF, 0x01, 0, 13, 95]
        );
    }

    #[test]
    fn stop() {
        let dut = UbxCfgRstAction::stop();
        let msg = dut.to_bin();
        assert_eq!(
            msg,
            [0xb5, 0x62, 0x06, 0x04, 4, 0, 0x00, 0x00, 0x08, 0, 22, 116]
        );
    }
}
