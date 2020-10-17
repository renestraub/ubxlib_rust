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
    fn from_bin(&mut self, data: &[u8]);
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
        UbxFrame::bytes(self.cid(), &data)
    }
}

impl<T> UbxFrameDeSerialize for UbxFrameWithData<T>
where
    T: DeserializeOwned,
{
    fn from_bin(&mut self, data: &[u8]) {
        self.data = bincode::deserialize(&data).unwrap();
    }
}

// Generic implementation for ubx poll frame
// - no payload data
// - only serialization is implemented
#[derive(Default, Debug)]
pub struct UbxFramePoll {
    pub name: &'static str,
    pub cid: UbxCID,
}

impl UbxFramePoll {
    pub fn new(name: &'static str, cid: UbxCID) -> Self {
        Self { name, cid }
    }
}

impl UbxFrameInfo for UbxFramePoll {
    fn name(&self) -> &'static str {
        self.name
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxFramePoll {
    fn to_bin(&self) -> Vec<u8> {
        const DATA: [u8; 0] = [];
        UbxFrame::bytes(self.cid(), &DATA)
    }
}

#[derive(Default)]
pub struct UbxFrame {
    pub cid: UbxCID,
    pub data: Vec<u8>,
}

impl UbxFrame {
    pub fn bytes(cid: UbxCID, data: &[u8]) -> Vec<u8> {
        let frame = Self {
            cid,
            data: data.to_vec(),
        };
        frame.serialize()
    }

    fn serialize(&self) -> Vec<u8> {
        let mut checksum = Checksum::new();
        let mut msg = Vec::<u8>::with_capacity(256);

        // Header Sync
        msg.push(0xb5);
        msg.push(0x62);

        // Message Class & Id
        let cls = self.cid.cls();
        msg.push(cls);
        checksum.add(cls);

        let id = self.cid.id();
        msg.push(id);
        checksum.add(id);

        // Length - 16 bit little endian
        let length = self.data.len();
        let l_low = ((length >> 0) & 0xFF) as u8;
        msg.push(l_low);
        checksum.add(l_low);

        let l_high = ((length >> 8) & 0xFF) as u8;
        msg.push(l_high);
        checksum.add(l_high);

        // Payload
        for d in &self.data {
            msg.push(*d);
            checksum.add(*d)
        }

        // Checksum
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
        let msg = UbxFrame::bytes(UbxCID::new(0x05, 0x01), &[1, 2].to_vec());
        assert_eq!(msg, [0xb5, 0x62, 0x05, 0x01, 0x02, 0x00, 1, 2, 11, 47]);
    }

    #[test]
    fn poll_mon_ver() {
        // Poll UBX-MON-VER: B5 62 0A 04 00 00 0E 34
        let msg = UbxFrame::bytes(UbxCID::new(0x0A, 0x04), &[].to_vec());
        assert_eq!(msg, [0xb5, 0x62, 0x0a, 0x04, 0x00, 0x00, 0x0e, 0x34]);
    }
}
