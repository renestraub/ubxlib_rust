use serde::Serialize;
use serde_repr::Serialize_repr;

use crate::cid::UbxCID;
use crate::frame::UbxFrameWithData;

const CLS: u8 = 0x06;
const ID: u8 = 0x04;

#[derive(Serialize_repr, Debug)]
#[repr(u16)]
pub enum BbrMask {
    HotStart = 0x0000,
    _WarmStart = 0x0001,
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
    _HwReset = 0x04,
    Stop = 0x08,
    _Start = 0x09,
}

impl Default for ResetMode {
    fn default() -> Self {
        ResetMode::ImmediateHwReset
    }
}

#[derive(Default, Debug, Serialize)]
pub struct DataCfgRst {
    pub nav_bbr_mask: BbrMask,
    pub reset_mode: ResetMode,
    pub res1: u8,
}

impl DataCfgRst {
    pub fn from(nav_bbr_mask: BbrMask, reset_mode: ResetMode) -> Self {
        Self {
            nav_bbr_mask,
            reset_mode,
            ..Default::default()
        }
    }
}

pub struct UbxCfgRstAction {}

impl UbxCfgRstAction {
    pub fn cold_start() -> UbxFrameWithData<DataCfgRst> {
        UbxFrameWithData::init(
            "UBX-CFG-RST",
            UbxCID::new(CLS, ID),
            DataCfgRst::from(BbrMask::ColdStart, ResetMode::SwReset),
        )
    }

    pub fn stop() -> UbxFrameWithData<DataCfgRst> {
        UbxFrameWithData::init(
            "UBX-CFG-RST",
            UbxCID::new(CLS, ID),
            DataCfgRst::from(BbrMask::HotStart, ResetMode::Stop),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::UbxFrameSerialize;

    #[test]
    fn cold_start() {
        let dut = UbxCfgRstAction::cold_start();
        assert_eq!(dut.name, "UBX-CFG-RST");
        let msg = dut.to_bin();
        assert_eq!(
            msg,
            [0xb5, 0x62, 0x06, 0x04, 4, 0, 0xFF, 0xFF, 0x01, 0, 13, 95]
        );
    }

    #[test]
    fn stop() {
        let dut = UbxCfgRstAction::stop();
        assert_eq!(dut.name, "UBX-CFG-RST");
        let msg = dut.to_bin();
        assert_eq!(
            msg,
            [0xb5, 0x62, 0x06, 0x04, 4, 0, 0x00, 0x00, 0x08, 0, 22, 116]
        );
    }
}
