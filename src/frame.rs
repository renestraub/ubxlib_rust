use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt;

use crate::checksum::Checksum;
use crate::cid::UbxCID;

pub trait UbxFrameInfo {
    fn name(&self) -> &'static str;
    fn cid(&self) -> UbxCID;
}

pub trait UbxFrameSerialize {
    fn to_bin(&self) -> Vec<u8>;
}

pub trait UbxFrameDeSerialize {
    fn from_bin(&mut self, data: Vec<u8>);
    // TODO: Consider data: &[u8]
}

// Generic implementation for ubx frames that can be directly (de)serialized
#[derive(Default, Debug)]
pub struct UbxFrameWithData<T> {
    pub name: &'static str,
    pub cid: UbxCID,
    pub data: T,
}

impl<T> UbxFrameWithData<T>
where
    T: Default,
{
    // TODO: cid or CLS, ID? Would allow to remove dependancy on UbxCID from frames
    pub fn new(name: &'static str, cid: UbxCID) -> Self {
        Self {
            name,
            cid,
            ..Default::default()
        }
    }

    pub fn init(name: &'static str, cid: UbxCID, data: T) -> Self {
        Self { name, cid, data }
    }
}

impl<T> UbxFrameInfo for UbxFrameWithData<T> {
    fn name(&self) -> &'static str {
        self.name
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl<T> UbxFrameSerialize for UbxFrameWithData<T>
where
    T: Serialize,
{
    fn to_bin(&self) -> Vec<u8> {
        let data = bincode::serialize(&self.data).unwrap();
        UbxFrame::bytes(self.cid(), data)
    }
}

impl<T> UbxFrameDeSerialize for UbxFrameWithData<T>
where
    T: DeserializeOwned,
{
    fn from_bin(&mut self, data: Vec<u8>) {
        self.data = bincode::deserialize(&data).unwrap();
    }
}

// TODO: Generic for polling frame
// no data
// no deserialize

#[derive(Default)]
pub struct UbxFrame {
    pub cid: UbxCID,
    pub data: Vec<u8>,
}

impl UbxFrame {
    pub fn construct(cid: UbxCID, data: Vec<u8>) -> Self {
        Self { cid, data }
    }

    pub fn bytes(cid: UbxCID, data: Vec<u8>) -> Vec<u8> {
        let frame = UbxFrame::construct(cid, data);
        let msg = frame.to_bytes();
        msg
    }

    fn to_bytes(&self) -> Vec<u8> {
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
        msg.push(((length >> 0) & 0xFF) as u8); // TODO: proper pack/unpack crate
        msg.push(((length >> 8) & 0xFF) as u8); // TODO: there is surely one
        checksum.add(((length >> 0) & 0xFF) as u8);
        checksum.add(((length >> 8) & 0xFF) as u8);

        for d in &self.data {
            msg.push(*d);
            checksum.add(*d)
        }

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
    fn ack_frame() {
        let dut = UbxFrame::construct(UbxCID::new(0x05, 0x01), [1, 2].to_vec());
        let msg = dut.to_bytes();
        assert_eq!(msg, [0xb5, 0x62, 0x05, 0x01, 0x02, 0x00, 1, 2, 11, 47]);
    }

    #[test]
    fn poll_mon_ver() {
        // Poll UBX-MON-VER: B5 62 0A 04 00 00 0E 34
        let dut = UbxFrame::construct(UbxCID::new(0x0A, 0x04), [].to_vec());
        let msg = dut.to_bytes();
        assert_eq!(msg, [0xb5, 0x62, 0x0a, 0x04, 0x00, 0x00, 0x0e, 0x34]);
    }
}
