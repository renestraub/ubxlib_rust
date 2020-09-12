use std::fmt;

use crate::cid::UbxCID as UbxCID;
use crate::checksum::Checksum as Checksum;

pub struct UbxFrame {
    cid: UbxCID,
    data: Vec::<u8>,
}

impl UbxFrame {
    pub fn new() -> Self {
        Self { 
            cid: UbxCID::new(0, 0),
            data: Vec::<u8>::new(),
        }
    }

    pub fn construct(cid: UbxCID, data: Vec::<u8>) -> Self {
        Self { 
            cid: cid,
            data: data
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut msg = Vec::<u8>::new();

        msg.push(0xb5);
        msg.push(0x62);

        let cls = self.cid.cls();
        let id = self.cid.id();
        msg.push(cls);
        msg.push(id);

        let length = self.data.len();
        msg.push(((length >> 0) & 0xFF) as u8);   // TODO: proper pack/unpack crate
        msg.push(((length >> 8) & 0xFF) as u8);   // TODO: there is surely one

        for d in &self.data {
            msg.push(*d);
        }

        let checksum = self._calc_checksum();
        let (cka, ckb) = checksum.value();
        msg.push(cka);
        msg.push(ckb);

        msg
    }

    fn _calc_checksum(&self) -> Checksum {
        // TODO: Duplicates code from to_bytes() -> combine
        let mut checksum = Checksum::new();

        let cls = self.cid.cls();
        let id = self.cid.id();
        checksum.add(cls);
        checksum.add(id);

        let length = self.data.len();
        checksum.add(((length >> 0) & 0xFF) as u8);
        checksum.add(((length >> 8) & 0xFF) as u8);

        for d in &self.data {
            checksum.add(*d)
        }

        checksum
    }
}

impl fmt::Debug for UbxFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Frame")
         .field("cid", &self.cid)
         .field("len", &self.data.len())
         .field("data", &self.data)
         .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dummy_frame() {
        let dut = UbxFrame::new();
        let msg = dut.to_bytes();
        println!("message {:?}", msg);
        assert_eq!(msg, [0xb5, 0x62, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn ack_frame() {
        let dut = UbxFrame::construct(UbxCID::new(0x05, 0x01), [1, 2].to_vec());
        let msg = dut.to_bytes();
        println!("{:?}", dut);
        assert_eq!(msg, [0xb5, 0x62, 0x05, 0x01, 0x02, 0x00, 1, 2, 11, 47]);
    }

    #[test]
    fn poll_mon_ver() {
        // Poll UBX-MON-VER: B5 62 0A 04 00 00 0E 34 
        let dut = UbxFrame::construct(UbxCID::new(0x0A, 0x04), [].to_vec());
        println!("{:?}", dut);
        let msg = dut.to_bytes();
        assert_eq!(msg, [0xb5, 0x62, 0x0a, 0x04, 0x00, 0x00, 0x0e, 0x34]);
    }
}
