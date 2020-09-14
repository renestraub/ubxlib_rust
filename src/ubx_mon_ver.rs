// use std::fmt;

use crate::cid::UbxCID as UbxCID;
use crate::frame::UbxFrame as UbxFrame;
use crate::frame::UbxFrameInfo as UbxFrameInfo;
use crate::frame::UbxFrameSerialize as UbxFrameSerialize;
use crate::frame::UbxFrameDeSerialize as UbxFrameDeSerialize;


const CLS: u8 = 0x0A;
const ID: u8 = 0x04;
// const NAME: &'static str = "UBX-MON-VER";

pub struct UbxMonVerPoll {
    pub name: &'static str,
    cid: UbxCID,
}


impl UbxMonVerPoll {
    pub fn new() -> Self {
        Self {
            name: "UBX-MON-VER-POLL",
            cid: UbxCID::new(CLS, ID), 
        }
    }
}

impl UbxFrameInfo for UbxMonVerPoll {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxMonVerPoll {
    fn to_bin(&self) -> Vec<u8> {
        // poll frame has no data to serialize
        // just build empty frame
        let frame = UbxFrame::construct(UbxCID::new(CLS, ID), [].to_vec());
        let msg = frame.to_bytes();
        msg
        // TODO: simplify
    }
}


#[derive(Default)]
#[derive(Debug)]
pub struct UbxMonVer {
    pub name: &'static str,
    cid: UbxCID,

    pub sw_version: String,
    pub hw_version: String,
    pub hw_extension: String,   // TODO: Should be array of Strings
}

impl UbxMonVer {
    pub fn new() -> Self {
        Self {
            name: "UBX-MON-VER",
            cid: UbxCID::new(CLS, ID),
            ..Default::default()
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        let bytes = data.len();
        println!("UbxMonVer, got {} bytes", bytes);
        assert!(bytes >= 40);

        // TODO: search simpler way, DRY
        let text = &data[0..30].to_vec();
        let text = String::from_utf8_lossy(text);
        self.sw_version = text.replace(|c: char| c == '\0', "");

        let text = &data[30..40].to_vec();
        let text = String::from_utf8_lossy(text);
        self.hw_version = text.replace(|c: char| c == '\0', "");

        if bytes > 40 {
            assert!((bytes - 40) % 30 == 0);

            let mut offset = 40;
            let size = 30;
            while offset < bytes {
                let text = &data[offset..offset+size].to_vec();
                let text = String::from_utf8_lossy(text);
                let text = text.replace(|c: char| c == '\0', "");
                println!("{}: {}", offset, text);

                offset += size;
            }
        }
    }
}

impl UbxFrameInfo for UbxMonVer {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameDeSerialize for UbxMonVer {
    fn from_bin(&mut self, data: Vec<u8>) {
        self.load(&data);
    }
}

/*
impl fmt::Debug for UbxMonVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UbxCfgRate")
        .field("cid", &self.cid)
        .field("measRate", &self.meas_rate)
        .field("navRate", &self.nav_rate)
        .field("timeRef", &self.time_ref)
        .finish()
    }
}
*/


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ubx_mon_ver_poll() {
        let dut = UbxMonVerPoll::new();
        assert_eq!(dut.name, "UBX-MON-VER-POLL");
        let msg = dut.to_bin();
        println!("message {:?}", msg);
        assert_eq!(msg, [0xb5, 0x62, 0x0A, 0x04, 0, 0, 14, 52]);
    }

    #[test]
    fn ubx_mon_ver() {
        let dut = UbxMonVer::new();
        assert_eq!(dut.name, "UBX-MON-VER");

        println!("message {:?}", dut);

        assert_eq!(dut.sw_version, "");
        assert_eq!(dut.hw_version, "");
    }
}
