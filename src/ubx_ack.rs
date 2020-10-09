use serde::Deserialize;

use crate::cid::UbxCID;
use crate::frame::UbxFrameWithData;

const CLS: u8 = 0x05;
const ID_ACK: u8 = 0x01;
const ID_NAK: u8 = 0x00;

#[derive(Default, Debug, Deserialize)]
pub struct Data {
    pub cls_id: u8,
    pub msg_id: u8,
}

pub struct UbxAck {}

impl UbxAck {
    pub fn new(id: u8) -> UbxFrameWithData<Data> {
        match id {
            0x00 => UbxFrameWithData::new("UBX-ACK-NAK", UbxCID::new(CLS, ID_NAK)),
            0x01 => UbxFrameWithData::new("UBX-ACK-ACK", UbxCID::new(CLS, ID_ACK)),
            _ => panic!("illegal UBX-ACK id"),
        }
    }
}

impl UbxFrameWithData<Data> {
    pub fn ack_cid(&self) -> UbxCID {
        return UbxCID::new(self.data.cls_id, self.data.msg_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::UbxFrameDeSerialize;

    #[test]
    fn poll() {
        let dut = UbxAck::new(ID_ACK);
        assert_eq!(dut.name, "UBX-ACK-ACK");

        let dut = UbxAck::new(ID_NAK);
        assert_eq!(dut.name, "UBX-ACK-NAK");
    }

    #[test]
    fn deserialize() {
        const DATA: [u8; 2] = [0x20, 0x04];
        let mut dut = UbxAck::new(ID_ACK);
        dut.from_bin(&DATA);

        assert_eq!(dut.data.cls_id, 0x20);
        assert_eq!(dut.data.msg_id, 0x04);
    }
}
