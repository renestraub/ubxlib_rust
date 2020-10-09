use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

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

#[derive(Default, Debug, Serialize)]
pub struct DataPoll {}

pub struct UbxUpdSosPoll {}

impl UbxUpdSosPoll {
    pub fn new() -> UbxFrameWithData<DataPoll> {
        UbxFrameWithData::new("UBX-UPD-SOS-POLL", UbxCID::new(CLS, ID))
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct DataResponse {
    pub cmd: u8, // shall be 0x02
    pub res1: [u8; 3],
    pub response: Response,
    pub res2: [u8; 3],
}

pub struct UbxUpdSos {}

impl UbxUpdSos {
    pub fn new() -> UbxFrameWithData<DataResponse> {
        UbxFrameWithData::new("UBX-UPD-SOS", UbxCID::new(CLS, ID))
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
mod action {
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

#[cfg(test)]
mod response {
    use super::*;
    use crate::frame::{UbxFrameDeSerialize, UbxFrameSerialize};

    #[test]
    fn poll() {
        let dut = UbxUpdSosPoll::new();
        assert_eq!(dut.name, "UBX-UPD-SOS-POLL");
        let msg = dut.to_bin();
        assert_eq!(msg, [0xb5, 0x62, 0x09, 0x14, 0, 0, 29, 96]);
    }

    #[test]
    fn deserialize() {
        const DATA: [u8; 8] = [0x03, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00];
        let mut dut = UbxUpdSos::new();
        assert_eq!(dut.name, "UBX-UPD-SOS");
        dut.from_bin(&DATA);

        assert_eq!(dut.data.cmd, 0x03);
        assert_eq!(dut.data.response, Response::Restored);
    }
}
