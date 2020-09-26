use serde::{Deserialize, Serialize};

use crate::cid::UbxCID;
use crate::frame::UbxFrameWithData;

const CLS: u8 = 0x06;
const ID: u8 = 0x08;


/*
pub trait UbxFrameDeSerialize2 {
    fn from_bin2(&mut self, data: &[u8] /*Vec<u8>*/);
}

impl<'a, T> UbxFrameDeSerialize2 for UbxFrameWithData<T>
where T: Deserialize<'a>
{
    fn from_bin2(&mut self, data: &[u8] /*Vec<u8>*/) {
        let mut data2: T;

        println!("data {:?}", data);

        self.data_frame = data.to_vec();
        // self.data = bincode::deserialize(&self.data_frame).unwrap();
    }
}
*/

/**************************************************************/
/*
pub struct UbxCfgRate { }
      
impl UbxCfgRate { 
    pub fn new() -> UbxFrameWithData<Data> {
        UbxFrameWithData::new("UBX-CFG-RATE", UbxCID::new(CLS, ID))
    }
}

impl UbxFrameDeSerialize for UbxFrameWithData<Data> {
    fn from_bin(&mut self, data: Vec<u8>) {
        self.data = bincode::deserialize(&data).unwrap();
    }
}

pub struct UbxCfgRatePoll { }
      
impl UbxCfgRatePoll { 
    pub fn new() -> UbxFrameWithData<DataPoll> {
        UbxFrameWithData::new("UBX-CFG-RATE-POLL", UbxCID::new(CLS, ID))
    }
}

/**************************************************************/

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct DataPoll {
}
*/

#[derive(Default, Debug, Serialize)]
pub struct DataPoll {
}

pub struct UbxCfgRatePoll { }
      
impl UbxCfgRatePoll { 
    pub fn new() -> UbxFrameWithData<DataPoll> {
        UbxFrameWithData::new("UBX-CFG-RATE-POLL", UbxCID::new(CLS, ID))
    }
}

pub struct UbxCfgRate { }
      
impl UbxCfgRate { 
    pub fn new() -> UbxFrameWithData<Data> {
        UbxFrameWithData::new("UBX-CFG-RATE", UbxCID::new(CLS, ID))
    }
}


/**************************************************************/
/*
pub struct UbxCfgRatePoll {
    pub name: &'static str,
    cid: UbxCID,
}

impl UbxCfgRatePoll {
    pub fn new() -> Self {
        Self {
            name: "UBX-CFG-RATE-POLL",
            cid: UbxCID::new(CLS, ID),
        }
    }
}

impl UbxFrameInfo for UbxCfgRatePoll {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxCfgRatePoll {
    fn to_bin(&self) -> Vec<u8> {
        UbxFrame::bytes(UbxCID::new(CLS, ID), [].to_vec())
    }
}
*/

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Data {
    pub meas_rate: u16, // Time elapsed between two measuremnts in ms
    pub nav_rate: u16,  // Number of measurements for NAV solution
    pub time_ref: u16,
}

/*
#[derive(Default, Debug)]
pub struct UbxCfgRate {
    pub name: &'static str,
    cid: UbxCID,
    pub data: Data,
}

impl UbxCfgRate {
    pub fn new() -> Self {
        Self {
            name: "UBX-CFG-RATE",
            cid: UbxCID::new(CLS, ID),
            ..Default::default()
        }
    }
}

impl UbxFrameInfo for UbxCfgRate {
    fn name(&self) -> String {
        String::from(self.name)
    }

    fn cid(&self) -> UbxCID {
        self.cid
    }
}

impl UbxFrameSerialize for UbxCfgRate {
    fn to_bin(&self) -> Vec<u8> {
        let data = bincode::serialize(&self.data).unwrap();
        UbxFrame::bytes(UbxCID::new(CLS, ID), data)
    }
}

impl UbxFrameDeSerialize for UbxCfgRate {
    fn from_bin(&mut self, data: Vec<u8>) {
        assert_eq!(data.len(), 6);
        self.data = bincode::deserialize(&data).unwrap();
    }
}
*/

#[cfg(test)]
mod tests {
    use crate::frame::{UbxFrameDeSerialize, UbxFrameSerialize};
    use super::*;

    #[test]
    fn cfg_rate_poll() {
        let dut = UbxCfgRatePoll::new();
        assert_eq!(dut.name, "UBX-CFG-RATE-POLL");
        let msg = dut.to_bin();
        assert_eq!(msg, [0xb5, 0x62, 0x06, 0x08, 0, 0, 14, 48]);
    }

    #[test]
    fn cfg_rate_load() {
        const DATA: [u8; 6] = [0xe8, 0x03, 0x01, 0x00, 0x34, 0x12];
        let mut dut = UbxCfgRate::new();
        dut.from_bin(DATA.to_vec());

        assert_eq!(dut.data.meas_rate, 1000);
        assert_eq!(dut.data.nav_rate, 1);
        assert_eq!(dut.data.time_ref, 0x1234);
    }

    #[test]
    #[should_panic]
    fn cfg_rate_load_too_few_values() {
        const DATA: [u8; 5] = [0xe8, 0x03, 0x01, 0x00, 0x34];
        let mut dut = UbxCfgRate::new();
        dut.from_bin(DATA.to_vec());

        assert_eq!(dut.data.meas_rate, 1000);
        assert_eq!(dut.data.nav_rate, 1);
        assert_eq!(dut.data.time_ref, 0x1234);
    }

    #[test]
    fn cfg_rate_change() {
        let mut dut = UbxCfgRate::new();
        assert_eq!(dut.data.meas_rate, 0);
        assert_eq!(dut.data.nav_rate, 0);
        assert_eq!(dut.data.time_ref, 0);

        dut.data.meas_rate = 1000;
        dut.data.nav_rate = 1;
        dut.data.time_ref = 0x1234;
        let data = dut.to_bin();
        assert_eq!(data[6..12], [0xE8, 3, 1, 0, 0x34, 0x12]);
    }

    #[test]
    fn cfg_rate_serialize() {
        let mut dut = UbxCfgRate::new();
        assert_eq!(dut.data.meas_rate, 0);
        assert_eq!(dut.data.nav_rate, 0);
        assert_eq!(dut.data.time_ref, 0);

        dut.data.meas_rate = 1000;
        dut.data.nav_rate = 1;
        dut.data.time_ref = 0x1234;

        let data = dut.to_bin();
        assert_eq!(
            data,
            [0xb5, 0x62, 0x06, 0x08, 0x06, 0x00, 0xE8, 0x03, 0x01, 0x00, 0x34, 0x12, 70, 177]
        );
    }

    #[test]
    fn cfg_rate_deserialize() {
        const DATA: [u8; 6] = [0xE8, 0x03, 0x01, 0x00, 0x34, 0x12];

        let mut dut = UbxCfgRate::new();
        dut.from_bin(DATA.to_vec());
        assert_eq!(dut.data.meas_rate, 1000);
        assert_eq!(dut.data.nav_rate, 1);
        assert_eq!(dut.data.time_ref, 0x1234);
    }
}
