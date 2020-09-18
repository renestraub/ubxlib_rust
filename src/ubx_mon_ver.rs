use crate::cid::UbxCID;
use crate::frame::{UbxFrame, UbxFrameInfo, UbxFrameSerialize, UbxFrameDeSerialize};


const CLS: u8 = 0x0A;
const ID: u8 = 0x04;


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
        let frame = UbxFrame::construct(UbxCID::new(CLS, ID), [].to_vec());
        let msg = frame.to_bytes();
        msg
        // TODO: simplify
    }
}


#[derive(Default, Debug)]
pub struct UbxMonVer {
    pub name: &'static str,
    cid: UbxCID,

    pub sw_version: String,
    pub hw_version: String,
    pub hw_extension: Vec<String>,
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
        assert!(bytes >= 40);

        self.sw_version = UbxMonVer::extract_string(&data[0..30]);
        self.hw_version = UbxMonVer::extract_string(&data[30..40]);

        if bytes > 40 {
            assert!((bytes - 40) % 30 == 0);

            let mut offset = 40;
            let size = 30;
            while offset < bytes {
                let text = UbxMonVer::extract_string(&data[offset..offset+size]);
                self.hw_extension.push(text);

                offset += size;
            }
        }
    }

    fn extract_string(data: &[u8]) -> String {
        String::from_utf8_lossy(&data).replace(|c: char| c == '\0', "")
    }

    // TODO: Decoder function for "FWVER=" etc.
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ubx_mon_ver_poll() {
        let dut = UbxMonVerPoll::new();
        assert_eq!(dut.name, "UBX-MON-VER-POLL");
        let msg = dut.to_bin();
        assert_eq!(msg, [0xb5, 0x62, 0x0A, 0x04, 0, 0, 14, 52]);
    }

    #[test]
    fn ubx_mon_ver() {
        let dut = UbxMonVer::new();
        assert_eq!(dut.name, "UBX-MON-VER");
        assert_eq!(dut.sw_version, "");
        assert_eq!(dut.hw_version, "");
    }
}
