use serde::{Serialize, Deserialize};

use crate::cid::UbxCID;
use crate::frame::{UbxFrame, UbxFrameInfo, UbxFrameSerialize, UbxFrameDeSerialize};


const CLS: u8 = 0x06;
const ID: u8 = 0x00;

const PORTID_UART: u8 = 1;


#[derive(Default, Debug, Serialize)]
pub struct DataPoll {
    pub port_id: u8,
}

impl DataPoll {
    pub fn new() -> Self {
        Self {
            port_id: PORTID_UART,
        }
    }
}


#[derive(Default, Debug)]
pub struct UbxCfgPrtPoll {
    pub name: &'static str,
    cid: UbxCID,
    pub data: DataPoll,
}

impl UbxCfgPrtPoll {
    pub fn new() -> Self {
        Self {
            name: "UBX-CFG-PRT-POLL",
            cid: UbxCID::new(CLS, ID),
            data: DataPoll::new(),
        }
    }
}

impl UbxFrameInfo for UbxCfgPrtPoll {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxCfgPrtPoll {
    fn to_bin(&self) -> Vec<u8> {
        let data = bincode::serialize(&self.data).unwrap();
        let frame = UbxFrame::construct(UbxCID::new(CLS, ID), data);
        let msg = frame.to_bytes();
        msg
    }
}


#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Data {
    pub port_id: u8,
    pub res1: u8,
    pub tx_ready: u16,
    pub mode: u32,
    pub baudrate: u32,

    pub in_proto_mask: u16,
    pub out_proto_mask: u16,
    pub flags: u16,
    pub res2: [u8; 2],
}

/*
        self.f.add(U1('PortId'))
        self.f.add(Padding(1, 'res1'))
        self.f.add(X2('txReady'))
        self.f.add(X4_Mode('mode'))
        self.f.add(U4('baudRate'))

        self.f.add(X2_Proto('inProtoMask'))
        self.f.add(X2_Proto('outProtoMask'))
        self.f.add(X2('flags'))
        self.f.add(Padding(2, 'res2'))
*/

#[derive(Default, Debug)]
pub struct UbxCfgPrtUart {
    pub name: &'static str,
    cid: UbxCID,
    pub data: Data,
}

impl UbxCfgPrtUart {
    pub fn new() -> Self {
        Self {
            name: "UBX-CFG-PRT",
            cid: UbxCID::new(CLS, ID),
            ..Default::default()
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        assert!(data.len() == 20);
        self.data = bincode::deserialize(&data).unwrap();
    }

    pub fn save(&self) -> Vec<u8> {
        let data = bincode::serialize(&self.data).unwrap();
        assert!(data.len() == 20);
        data
    }
}

impl UbxFrameInfo for UbxCfgPrtUart {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxCfgPrtUart {
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

impl UbxFrameDeSerialize for UbxCfgPrtUart {
    fn from_bin(&mut self, data: Vec<u8>) {
        self.load(&data);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poll() {
        let dut = UbxCfgPrtPoll::new();
        assert_eq!(dut.name, "UBX-CFG-PRT-POLL");
        assert_eq!(dut.data.port_id, 1);
        let msg = dut.to_bin();
        assert_eq!(msg, [0xb5, 0x62, 0x06, 0x00, 1, 0, 1, 8, 34]);
    }

    #[test]
    fn set() {
        let mut dut = UbxCfgPrtUart::new();
        assert_eq!(dut.name, "UBX-CFG-PRT");
        dut.data.baudrate = 115200;
        let msg = dut.to_bin();
        assert_eq!(msg[0..6], [0xb5, 0x62, 0x06, 0x00, 20, 0]);
        assert_eq!(msg[6+8..6+12], [0, 194, 1, 0]);
    }
}
