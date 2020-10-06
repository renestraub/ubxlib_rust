use serde::Serialize;
use serde_repr::Serialize_repr;

use crate::cid::UbxCID;
use crate::frame::UbxFrameWithData;

const CLS: u8 = 0x09;
const ID: u8 = 0x14;

#[derive(Serialize_repr, Debug)]
#[repr(u8)]
pub enum Command {
    Backup = 0x00,
    Clear = 0x01,
}

impl Default for Command {
    fn default() -> Self {
        Command::Backup
    }
}

#[derive(Default, Debug, Serialize)]
pub struct UpdSosAction {
    pub cmd: Command,
    pub res1: [u8; 3],
}

impl UpdSosAction {
    pub fn new(cmd: Command) -> Self {
        Self {
            cmd,
            ..Default::default()
        }
    }
}

pub struct UbxUpdSosAction {}

impl UbxUpdSosAction {
    pub fn backup() -> UbxFrameWithData<UpdSosAction> {
        UbxFrameWithData::init(
            "UBX-UPD-SOS-ACTION",
            UbxCID::new(CLS, ID),
            UpdSosAction::new(Command::Backup),
        )
    }

    pub fn clear() -> UbxFrameWithData<UpdSosAction> {
        UbxFrameWithData::init(
            "UBX-UPD-SOS-ACTION",
            UbxCID::new(CLS, ID),
            UpdSosAction::new(Command::Clear),
        )
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
