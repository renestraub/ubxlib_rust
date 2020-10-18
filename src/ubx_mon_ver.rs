use serde::Deserialize;

use crate::cid::UbxCID;
use crate::frame::{UbxFrameDeSerialize, UbxFrameInfo};
use crate::frame::{UbxFramePoll, UbxFrameWithData};

const CLS: u8 = 0x0A;
const ID: u8 = 0x04;

pub struct UbxMonVerPoll {}

impl UbxMonVerPoll {
    pub fn create() -> UbxFramePoll {
        UbxFramePoll::new("UBX-MON-VER-POLL", UbxCID::new(CLS, ID))
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct MonVer {
    pub sw_version: String,
    pub hw_version: String,
    pub hw_extension: Vec<String>,
}

#[derive(Debug)]
pub struct UbxMonVer {
    frame: UbxFrameWithData<MonVer>,
}

impl UbxMonVer {
    pub fn new() -> Self {
        Self {
            frame: UbxFrameWithData::new("UBX-MON-VER", UbxCID::new(CLS, ID)),
        }
    }

    pub fn hw_version(&self) -> String {
        self.frame.data.hw_version.clone()
    }

    pub fn sw_version(&self) -> String {
        self.frame.data.sw_version.clone()
    }

    pub fn get_info(&self, key: &str) -> String {
        for info in &self.frame.data.hw_extension {
            if info.starts_with(key) {
                let result = String::from(info);
                let result = result.trim_start_matches(key);
                return result.to_string();
            }
        }
        String::from("")
    }

    pub fn get_ext(&self, index: usize) -> String {
        if index < self.frame.data.hw_extension.len() {
            let result = self.frame.data.hw_extension[index].clone();
            return result;
        }
        String::from("")
    }

    pub fn load(&mut self, data: &[u8]) {
        let bytes = data.len();
        assert!(bytes >= 40);

        self.frame.data.sw_version = UbxMonVer::extract_string(&data[0..30]);
        self.frame.data.hw_version = UbxMonVer::extract_string(&data[30..40]);

        if bytes > 40 {
            assert!((bytes - 40) % 30 == 0);

            let mut offset = 40;
            let size = 30;
            while offset < bytes {
                let text = UbxMonVer::extract_string(&data[offset..offset + size]);
                self.frame.data.hw_extension.push(text);

                offset += size;
            }
        }
    }

    fn extract_string(data: &[u8]) -> String {
        // Version strings are zero padded, remove these and return String
        String::from_utf8_lossy(&data).replace(|c: char| c == '\0', "")
    }
}

impl UbxFrameInfo for UbxMonVer {
    fn name(&self) -> &'static str {
        self.frame.name()
    }

    fn cid(&self) -> UbxCID {
        self.frame.cid()
    }
}

impl UbxFrameDeSerialize for UbxMonVer {
    fn from_bin(&mut self, data: &[u8]) {
        self.load(&data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{UbxFrameDeSerialize, UbxFrameSerialize};

    #[test]
    fn poll() {
        let dut = UbxMonVerPoll::create();
        assert_eq!(dut.name, "UBX-MON-VER-POLL");
        let msg = dut.to_bin();
        assert_eq!(msg, [0xb5, 0x62, 0x0A, 0x04, 0, 0, 14, 52]);
    }

    #[test]
    fn new() {
        let dut = UbxMonVer::new();
        assert_eq!(dut.name(), "UBX-MON-VER");
        assert_eq!(dut.sw_version(), "");
        assert_eq!(dut.hw_version(), "");
    }

    #[test]
    fn deserialize() {
        const DATA: [u8; 250] = [
            69, 88, 84, 32, 67, 79, 82, 69, 32, 51, 46, 48, 49, 32, 40, 49, 101, 99, 57, 51, 102,
            41, 0, 0, 0, 0, 0, 0, 0, 0, 48, 48, 48, 56, 48, 48, 48, 48, 0, 0, 82, 79, 77, 32, 66,
            65, 83, 69, 32, 51, 46, 48, 49, 32, 40, 49, 48, 55, 56, 56, 56, 41, 0, 0, 0, 0, 0, 0,
            0, 0, 70, 87, 86, 69, 82, 61, 65, 68, 82, 32, 52, 46, 50, 49, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 80, 82, 79, 84, 86, 69, 82, 61, 49, 57, 46, 50, 48, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 77, 79, 68, 61, 78, 69, 79, 45, 77, 56, 76, 45,
            48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 70, 73, 83, 61, 48, 120, 69, 70,
            52, 48, 49, 53, 32, 40, 49, 48, 48, 49, 49, 49, 41, 0, 0, 0, 0, 0, 0, 0, 0, 0, 71, 80,
            83, 59, 71, 76, 79, 59, 71, 65, 76, 59, 66, 68, 83, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 83, 66, 65, 83, 59, 73, 77, 69, 83, 59, 81, 90, 83, 83, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let mut dut = UbxMonVer::new();
        dut.from_bin(&DATA);
        assert_eq!(dut.sw_version(), "EXT CORE 3.01 (1ec93f)");
        assert_eq!(dut.hw_version(), "00080000");
        assert_eq!(dut.get_info("PROTVER="), "19.20");
        assert_eq!(dut.get_ext(5), "GPS;GLO;GAL;BDS");
        assert_eq!(dut.get_ext(7), "");
    }
}
