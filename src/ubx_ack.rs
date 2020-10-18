use serde::Deserialize;

use crate::cid::UbxCID;
use crate::frame::UbxFrameWithData;

pub const CLS_ACK: u8 = 0x05;
pub const ID_ACK: u8 = 0x01;
pub const ID_NAK: u8 = 0x00;

#[derive(Default, Debug, Deserialize)]
pub struct DataAck {
    pub cls_id: u8,
    pub msg_id: u8,
}

pub struct UbxAck {}

impl UbxAck {
    pub fn from(id: u8) -> UbxFrameWithData<DataAck> {
        match id {
            ID_NAK => UbxFrameWithData::new("UBX-ACK-NAK", UbxCID::new(CLS_ACK, ID_NAK)),
            ID_ACK => UbxFrameWithData::new("UBX-ACK-ACK", UbxCID::new(CLS_ACK, ID_ACK)),
            _ => panic!("illegal UBX-ACK id {}", id),
        }
    }
}

impl UbxFrameWithData<DataAck> {
    pub fn ack_cid(&self) -> UbxCID {
        UbxCID::new(self.data.cls_id, self.data.msg_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::UbxFrameDeSerialize;

    #[test]
    fn poll() {
        let dut = UbxAck::from(ID_ACK);
        assert_eq!(dut.name, "UBX-ACK-ACK");

        let dut = UbxAck::from(ID_NAK);
        assert_eq!(dut.name, "UBX-ACK-NAK");
    }

    #[test]
    #[should_panic]
    fn poll_fail() {
        let _dut = UbxAck::from(0x33);
    }

    #[test]
    fn deserialize() {
        const DATA: [u8; 2] = [0x20, 0x04];
        let mut dut = UbxAck::from(ID_ACK);
        dut.from_bin(&DATA);

        assert_eq!(dut.data.cls_id, 0x20);
        assert_eq!(dut.data.msg_id, 0x04);
    }
}
