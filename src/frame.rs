use std::fmt;

use crate::cid::UbxCID;
use crate::checksum::Checksum;


#[derive(Default)]
pub struct UbxFrame {
    pub cid: UbxCID,
    pub data: Vec::<u8>,
}


pub trait UbxFrameInfo {
    fn name(&self) -> String;
    fn cid(&self) -> UbxCID;
}

pub trait UbxFrameSerialize {
    fn to_bin(&self) -> Vec<u8>;
}

pub trait UbxFrameDeSerialize {
    fn from_bin(&mut self, data: Vec<u8>);
}


impl UbxFrame {
    #[cfg(test)]    // only for test, remove later?
    pub fn new() -> Self {
        //..Default::default()
        Self { 
            cid: UbxCID::new(0, 0),
            data: Vec::<u8>::new(),
        }
    }

    /*
    pub fn construct_empty(cid: UbxCID) -> Self {
        Self { 
            cid: cid,
            data: Vec::<u8>::new(),
        }
    }
    */

    pub fn construct(cid: UbxCID, data: Vec::<u8>) -> Self {
        Self { 
            cid: cid,
            data: data
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut checksum = Checksum::new();
        let mut msg = Vec::<u8>::new();

        msg.push(0xb5);
        msg.push(0x62);

        let cls = self.cid.cls();
        let id = self.cid.id();
        msg.push(cls);
        msg.push(id);
        checksum.add(cls);
        checksum.add(id);

        let length = self.data.len();
        msg.push(((length >> 0) & 0xFF) as u8);   // TODO: proper pack/unpack crate
        msg.push(((length >> 8) & 0xFF) as u8);   // TODO: there is surely one
        checksum.add(((length >> 0) & 0xFF) as u8);
        checksum.add(((length >> 8) & 0xFF) as u8);

        for d in &self.data {
            msg.push(*d);
            checksum.add(*d)
        }

        // let checksum = self._calc_checksum();
        let (cka, ckb) = checksum.value();
        msg.push(cka);
        msg.push(ckb);

        msg
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
