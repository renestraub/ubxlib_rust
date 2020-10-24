use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::ubxlib::cid::UbxCID;
use crate::ubxlib::frame::{UbxFramePoll, UbxFrameWithData};

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

#[derive(Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum Response {
    Unknown = 0,
    RestoreFailed = 1,
    Restored = 2,
    NotRestoredNoBackup = 3,
}

impl Default for Response {
    fn default() -> Self {
        Response::Unknown
    }
}

pub struct UbxUpdSosPoll {}

impl UbxUpdSosPoll {
    pub fn create() -> UbxFramePoll {
        UbxFramePoll::new("UBX-UPD-SOS-POLL", UbxCID::new(CLS, ID))
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct DataUpdSosResponse {
    pub cmd: u8, // shall be 0x02
    pub res1: [u8; 3],
    pub response: Response,
    pub res2: [u8; 3],
}

pub struct UbxUpdSos {}

impl UbxUpdSos {
    pub fn create() -> UbxFrameWithData<DataUpdSosResponse> {
        UbxFrameWithData::new("UBX-UPD-SOS", UbxCID::new(CLS, ID))
    }
}

#[derive(Default, Debug, Serialize)]
pub struct DataUpdSosAction {
    pub cmd: Command,
    pub res1: [u8; 3],
}

impl DataUpdSosAction {
    pub fn from(cmd: Command) -> Self {
        Self {
            cmd,
            ..Default::default()
        }
    }
}

pub struct UbxUpdSosAction {}

impl UbxUpdSosAction {
    pub fn backup() -> UbxFrameWithData<DataUpdSosAction> {
        UbxFrameWithData::init(
            "UBX-UPD-SOS-ACTION",
            UbxCID::new(CLS, ID),
            DataUpdSosAction::from(Command::Backup),
        )
    }

    pub fn clear() -> UbxFrameWithData<DataUpdSosAction> {
        UbxFrameWithData::init(
            "UBX-UPD-SOS-ACTION",
            UbxCID::new(CLS, ID),
            DataUpdSosAction::from(Command::Clear),
        )
    }
}

#[cfg(test)]
mod action {
    use super::*;
    use crate::ubxlib::frame::UbxFrameSerialize;

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

#[cfg(test)]
mod response {
    use super::*;
    use crate::ubxlib::frame::{UbxFrameDeSerialize, UbxFrameSerialize};

    #[test]
    fn poll() {
        let dut = UbxUpdSosPoll::create();
        assert_eq!(dut.name, "UBX-UPD-SOS-POLL");
        let msg = dut.to_bin();
        assert_eq!(msg, [0xb5, 0x62, 0x09, 0x14, 0, 0, 29, 96]);
    }

    #[test]
    fn deserialize() {
        const DATA: [u8; 8] = [0x03, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00];
        let mut dut = UbxUpdSos::create();
        assert_eq!(dut.name, "UBX-UPD-SOS");
        dut.from_bin(&DATA);

        assert_eq!(dut.data.cmd, 0x03);
        assert_eq!(dut.data.response, Response::Restored);
    }
}
