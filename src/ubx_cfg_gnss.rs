use serde::{Deserialize, Serialize};

use crate::cid::UbxCID;
use crate::frame::{UbxFrame, UbxFrameDeSerialize, UbxFrameInfo, UbxFrameSerialize};

const CLS: u8 = 0x06;
const ID: u8 = 0x3E;

#[derive(Copy, Clone)]
pub enum SystemName {
    Gps = 0,
    Sbas = 1,
    Galileo = 2,
    Beidou = 3,
    Imes = 4,
    Qzss = 5,
    Glonass = 6,
}

pub struct UbxCfgGnssPoll {
    pub name: &'static str,
    pub cid: UbxCID,
}

impl UbxCfgGnssPoll {
    pub fn new() -> Self {
        Self {
            name: "UBX-CFG-GNSS-POLL",
            cid: UbxCID::new(CLS, ID),
        }
    }
}

impl UbxFrameInfo for UbxCfgGnssPoll {
    fn name(&self) -> &'static str {
        self.name
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxCfgGnssPoll {
    fn to_bin(&self) -> Vec<u8> {
        UbxFrame::bytes(UbxCID::new(CLS, ID), &[].to_vec())
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Header {
    pub msg_ver: u8,
    pub num_trk_ch_hw: u8,
    pub num_trk_ch_use: u8,
    pub num_config_blocks: u8,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct CfgBlock {
    pub gnss_id: u8,
    pub res_trk_ch: u8,
    pub max_trk_ch: u8,
    pub res1: u8,
    pub flags: u32,
}

#[derive(Default, Debug)]
pub struct UbxCfgGnss {
    pub name: &'static str,
    pub cid: UbxCID,
    pub header: Header,
    pub configs: Vec<CfgBlock>,
}

impl UbxCfgGnss {
    pub fn new() -> Self {
        Self {
            name: "UBX-CFG-GNSS",
            cid: UbxCID::new(CLS, ID),
            ..Default::default()
        }
    }

    pub fn enable(&mut self, system: SystemName) {
        if let Some(cfg) = self.find_config(system) {
            cfg.flags |= 1;
        }
    }

    #[cfg(test)]
    pub fn disable(&mut self, system: SystemName) {
        if let Some(cfg) = self.find_config(system) {
            cfg.flags &= !1;
        }
    }

    pub fn disable_all(&mut self) {
        for cfg in &mut self.configs {
            cfg.flags &= !1;
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        // First read header to get number of config blocks that follow
        let bytes = data.len();
        assert!(bytes >= 4);
        self.header = bincode::deserialize(&data[0..4]).unwrap();
        assert!(self.header.num_config_blocks <= 8);

        // Then read configuration blocks
        if bytes > 8 {
            assert!((bytes - 4) % 8 == 0);

            let mut offset = 4;
            let size = 8;
            while offset < bytes {
                let cfg: CfgBlock = bincode::deserialize(&data[offset..offset + size]).unwrap();
                self.configs.push(cfg);

                offset += size;
            }
        }
    }

    fn save(&self) -> Vec<u8> {
        let mut data = bincode::serialize(&self.header).unwrap();
        // println!("{:?}", data);

        for cfg in &self.configs {
            let mut cfg_data = bincode::serialize(&cfg).unwrap();
            // println!("{:?}", x);
            data.append(&mut cfg_data);
        }
        data
    }

    fn find_config(&mut self, system: SystemName) -> Option<&mut CfgBlock> {
        self.configs
            .iter_mut()
            .find(|c| c.gnss_id as usize == system as usize)
    }
}

impl UbxFrameInfo for UbxCfgGnss {
    fn name(&self) -> &'static str {
        self.name
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxCfgGnss {
    fn to_bin(&self) -> Vec<u8> {
        let data = self.save();
        UbxFrame::bytes(UbxCID::new(CLS, ID), &data)
    }
}

impl UbxFrameDeSerialize for UbxCfgGnss {
    fn from_bin(&mut self, data: &[u8]) {
        self.load(&data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poll() {
        let dut = UbxCfgGnssPoll::new();
        assert_eq!(dut.name, "UBX-CFG-GNSS-POLL");
        let msg = dut.to_bin();
        assert_eq!(msg, [0xb5, 0x62, 0x06, 0x3e, 0, 0, 68, 210]);
    }

    #[test]
    fn header_load() {
        const DATA: [u8; 4] = [0x00, 26, 16, 1];
        let mut dut = UbxCfgGnss::new();
        dut.from_bin(&DATA);

        assert_eq!(dut.name, "UBX-CFG-GNSS");
        assert_eq!(dut.header.msg_ver, 0x00);
        assert_eq!(dut.header.num_trk_ch_hw, 26);
        assert_eq!(dut.header.num_trk_ch_use, 16);
        assert_eq!(dut.header.num_config_blocks, 1);
    }

    #[test]
    fn header_and_config_load() {
        const DATA: [u8; 12] = [0x00, 26, 16, 1, 2, 12, 8, 0, 0x00, 0x00, 0xFF, 0x00];
        let mut dut = UbxCfgGnss::new();
        dut.from_bin(&DATA);

        assert_eq!(dut.name, "UBX-CFG-GNSS");
        assert_eq!(dut.header.msg_ver, 0x00);
        assert_eq!(dut.header.num_trk_ch_hw, 26);
        assert_eq!(dut.header.num_trk_ch_use, 16);
        assert_eq!(dut.header.num_config_blocks, 1);

        let cfg = &dut.configs[0];
        assert_eq!(cfg.gnss_id, 2);
        assert_eq!(cfg.res_trk_ch, 12);
        assert_eq!(cfg.max_trk_ch, 8);
        assert_eq!(cfg.flags, 0x00ff0000);
    }

    #[test]
    fn header_and_multiple_config_load() {
        const DATA: [u8; 20] = [
            0x00, 26, 16, 1, 2, 12, 8, 0, 0x00, 0x00, 0xFF, 0x00, 3, 4, 2, 0, 0x01, 0x00, 0xAA,
            0x00,
        ];
        let mut dut = UbxCfgGnss::new();
        dut.from_bin(&DATA);

        assert_eq!(dut.name, "UBX-CFG-GNSS");
        assert_eq!(dut.header.msg_ver, 0x00);
        assert_eq!(dut.header.num_trk_ch_hw, 26);
        assert_eq!(dut.header.num_trk_ch_use, 16);
        assert_eq!(dut.header.num_config_blocks, 1);

        let cfg = &dut.configs[0];
        assert_eq!(cfg.gnss_id, 2);
        assert_eq!(cfg.res_trk_ch, 12);
        assert_eq!(cfg.max_trk_ch, 8);
        assert_eq!(cfg.flags, 0x00ff0000);

        let cfg = &dut.configs[1];
        assert_eq!(cfg.gnss_id, 3);
        assert_eq!(cfg.res_trk_ch, 4);
        assert_eq!(cfg.max_trk_ch, 2);
        assert_eq!(cfg.flags, 0x00aa0001);
    }

    #[test]
    fn enable_disable() {
        const DATA: [u8; 20] = [
            0x00, 26, 16, 1, 3, 12, 8, 0, 0x00, 0x00, 0xFF, 0x00, 2, 4, 2, 0, 0x01, 0x00, 0xAA,
            0x00,
        ];
        let mut dut = UbxCfgGnss::new();
        dut.from_bin(&DATA);

        dut.enable(SystemName::Beidou);
        let cfg = &dut.configs[0];
        assert_eq!(cfg.flags & 1, 1);

        dut.disable(SystemName::Beidou);
        let cfg = &dut.configs[0];
        assert_eq!(cfg.flags & 1, 0);

        dut.disable(SystemName::Galileo);
        let cfg = &dut.configs[1];
        assert_eq!(cfg.flags & 1, 0);

        dut.enable(SystemName::Galileo);
        let cfg = &dut.configs[1];
        assert_eq!(cfg.flags & 1, 1);

        dut.disable_all();
        let cfg = &dut.configs[0];
        assert_eq!(cfg.flags & 1, 0);
        let cfg = &dut.configs[1];
        assert_eq!(cfg.flags & 1, 0);
    }

    #[test]
    fn serialize() {
        const DATA: [u8; 20] = [
            0x00, 26, 16, 1, 3, 12, 8, 0, 0x00, 0x00, 0xFF, 0x00, 2, 4, 2, 0, 0x01, 0x00, 0xAA,
            0x00,
        ];
        const DATA_SERIALIZED: [u8; 20] = [
            0x00, 26, 16, 1, 3, 12, 8, 0, 0x01, 0x00, 0xFF, 0x00, 2, 4, 2, 0, 0x00, 0x00, 0xAA,
            0x00,
        ];

        let mut dut = UbxCfgGnss::new();
        dut.from_bin(&DATA);

        dut.enable(SystemName::Beidou);
        dut.disable(SystemName::Galileo);

        let res = dut.to_bin();
        assert_eq!(res.len(), 6 + 4 + 16 + 2);
        assert_eq!(res[6..26], DATA_SERIALIZED);
    }
}
