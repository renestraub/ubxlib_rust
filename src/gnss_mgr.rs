use std::collections::HashMap;

use crate::server_tty::ServerTty;

use crate::ubx_cfg_rate::{UbxCfgRate, UbxCfgRatePoll};
use crate::ubx_cfg_nmea::{UbxCfgNmea, UbxCfgNmeaPoll};
use crate::ubx_mon_ver::{UbxMonVer, UbxMonVerPoll};
use crate::ubx_cfg_nav5::{UbxCfgNav5, UbxCfgNav5Poll};
use crate::ubx_cfg_esfalg::{UbxCfgEsfAlg, UbxCfgEsfAlgPoll};

use crate::config_file::GnssMgrConfig;


// TODO: define information struct for version() method

// TODO: Implement !
// TODO: Return error code from each function

pub struct GnssMgr {
    pub device_name: String,

    server: ServerTty,
}

impl GnssMgr {
    // TODO: rename to open/create?
    // TODO: Result return code
    pub fn new(device: &str) -> Self {
        Self { 
            device_name: String::from(device), 
            server: ServerTty::new(device),
        }
    }

    pub fn version(&mut self, info: &mut HashMap<&str, String>) {
        let mut ver_result = UbxMonVer::new();
        let poll = UbxMonVerPoll::new();

        // TODO: error check missing for sure
        self.server.poll(&poll, &mut ver_result);
        // println!("{:?}", ver_result);

        // TODO: Don't assume fixed position for these entries
        let fw_ver = String::from(ver_result.hw_extension[1].trim_start_matches("FWVER="));
        let proto = String::from(ver_result.hw_extension[2].trim_start_matches("PROTVER="));
        let model = String::from(ver_result.hw_extension[3].trim_start_matches("MOD="));
        
        info.insert("model", model);
        info.insert("sw_ver", ver_result.sw_version);
        info.insert("hw_ver", ver_result.hw_version);
        info.insert("fw_ver", fw_ver);
        info.insert("protocol", proto);
        info.insert("systems", String::from(&ver_result.hw_extension[5]));
        info.insert("augmentation", String::from(&ver_result.hw_extension[6]));
    }

    pub fn configure(&mut self, config: &GnssMgrConfig) {
        println!("configure");
        println!("device {}", self.device_name);
        println!("config {:?}", config);

        if config.update_rate.is_some() {
            let rate = config.update_rate.unwrap();
            // println!("applying update_rate {}", rate);
            self.set_update_rate(rate);
        }

        // TODO: Overly complicated with these string types...
        match &config.mode {
            Some(mode) => {
                match mode.as_str() {
                    "stationary" => self.set_dynamic_mode(2),
                    "vehicle" => self.set_dynamic_mode(4),
                    _ => (),
                }
            },
            _ => (),
        }

        // TODO: Combine IMU angles in a struct, this is ugly
        if config.imu_yaw.is_some() &&
            config.imu_pitch.is_some() &&
            config.imu_roll.is_some() {
            self.set_imu_angles(config.imu_yaw.unwrap(), config.imu_pitch.unwrap(), config.imu_roll.unwrap());
        }
    }

    pub fn sos_save(&mut self) {
        println!("sos save");
        println!("device {}", self.device_name);
    }    

    pub fn sos_clear(&mut self) {
        println!("sos clear");
        println!("device {}", self.device_name);
    }    

    pub fn cold_start(&mut self) {
        println!("cold-start");
        println!("device {}", self.device_name);
    }    

    pub fn factory_reset(&mut self) {
        println!("factory-eset");
        println!("device {}", self.device_name);
    }    

    pub fn persist(&mut self) {
        println!("persist");
        println!("device {}", self.device_name);
    }

    fn set_update_rate(&mut self, rate: u16) {
        let mut set = UbxCfgRate::new();
        let poll = UbxCfgRatePoll::new();

        self.server.poll(&poll, &mut set);
        println!("current settings {:?}", set);

        println!("changing to {}", 1000/rate);
        set.data.meas_rate = 1000u16 / rate;
        println!("new settings {:?}", set);

        self.server.set(&set);
    }

    // TODO: make all these public?
    // TODO: Consider format of version parameter. The hex code is a bit too close to the UBX frame definition
    // enum ? string ?
    pub fn set_nmea_protocol_version(&mut self, version: u8) {
        let mut set = UbxCfgNmea::new();
        let poll = UbxCfgNmeaPoll::new();

        self.server.poll(&poll, &mut set);
        println!("current settings {:?}", set);

        println!("changing to {:02X}", version);
        set.data.nmea_version = version;
        println!("new settings {:?}", set);

        self.server.set(&set);
    }

    fn set_dynamic_mode(&mut self, model: u8) {
        let mut set = UbxCfgNav5::new();
        let poll = UbxCfgNav5Poll::new();

        self.server.poll(&poll, &mut set);
        println!("current settings {:?}", set.data);

        set.data.dyn_model = model;
        println!("new settings {:?}", set.data);

        self.server.set(&set);
    }

    // TODO: provide arguments
    fn set_imu_angles(&mut self, yaw: u16, pitch: i16, roll: i16) {
        let mut set = UbxCfgEsfAlg::new();
        let poll = UbxCfgEsfAlgPoll::new();

        self.server.poll(&poll, &mut set);
        println!("current IMU settings {:?}", set.data);

        set.data.yaw = yaw as u32 * 100;
        set.data.pitch = pitch as i16 * 100;
        set.data.roll = roll as i16 * 100;
        println!("new IMU settings {:?}", set.data);

        self.server.set(&set);
    }
}
