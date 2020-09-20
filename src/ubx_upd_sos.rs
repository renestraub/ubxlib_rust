use serde::Serialize;

use crate::cid::UbxCID;
use crate::frame::{UbxFrame, UbxFrameInfo, UbxFrameSerialize};

const CLS: u8 = 0x09;
const ID: u8 = 0x14;

/*
pub struct UbxUpdSosPoll {
    pub name: &'static str,
    cid: UbxCID,
}

impl UbxUpdSosPoll {
    pub fn new() -> Self {
        Self {
            name: "UBX-UPD-SOS-POLL",
            cid: UbxCID::new(CLS, ID),
        }
    }
}

impl UbxFrameInfo for UbxUpdSosPoll {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxUpdSosPoll {
    fn to_bin(&self) -> Vec<u8> {
        let frame = UbxFrame::construct(UbxCID::new(CLS, ID), [].to_vec());
        let msg = frame.to_bytes();
        msg
    }
}


#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Data {
    pub cmd: u8,
    pub res1: [u8; 3],
    pub response: u8,
    pub res2: [u8; 3],
}


#[derive(Default, Debug)]
pub struct UbxUpdSos {
    pub name: &'static str,
    cid: UbxCID,
    pub data: Data,
}

impl UbxUpdSos {
    pub fn new() -> Self {
        Self {
            name: "UBX-UPD-SOS",
            cid: UbxCID::new(CLS, ID),
            ..Default::default()
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        assert!(data.len() == 20);
        self.data = bincode::deserialize(&data).unwrap();
    }
}

impl UbxFrameInfo for UbxUpdSos {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxUpdSos {
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

impl UbxFrameDeSerialize for UbxUpdSos {
    fn from_bin(&mut self, data: Vec<u8>) {
        self.load(&data);
    }
}
*/

#[derive(Default, Debug, Serialize)]
pub struct DataAction {
    pub cmd: u8,
    pub res1: [u8; 3],
}

impl DataAction {
    pub fn new(cmd: u8) -> Self {
        Self {
            cmd: cmd,
            ..Default::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct UbxUpdSosAction {
    pub name: &'static str,
    cid: UbxCID,
    pub data: DataAction,
}

impl UbxUpdSosAction {
    pub fn backup() -> Self {
        Self {
            name: "UBX-UPD-SOS-ACTION",
            cid: UbxCID::new(CLS, ID),
            data: DataAction::new(0x00),
            ..Default::default()
        }
    }

    pub fn clear() -> Self {
        Self {
            name: "UBX-UPD-SOS-ACTION",
            cid: UbxCID::new(CLS, ID),
            data: DataAction::new(0x01),
            ..Default::default()
        }
    }
}

impl UbxFrameInfo for UbxUpdSosAction {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxUpdSosAction {
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
