use std::{thread, time};
use std::collections::HashMap;

use crate::config_file::{GnssMgrConfig, Xyz};
use crate::server_tty::ServerTty;
use crate::ubx_cfg_rate::{UbxCfgRate, UbxCfgRatePoll};
use crate::ubx_cfg_nmea::{UbxCfgNmea, UbxCfgNmeaPoll};
use crate::ubx_mon_ver::{UbxMonVer, UbxMonVerPoll};
use crate::ubx_cfg_nav5::{UbxCfgNav5, UbxCfgNav5Poll};
use crate::ubx_cfg_rst::UbxCfgRstAction;
use crate::ubx_cfg_esfalg::{UbxCfgEsfAlg, UbxCfgEsfAlgPoll};
use crate::ubx_cfg_esfla::UbxCfgEsflaSet;



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

        // TODO: Sat Systems



        // TODO: Combine IMU angles in a struct, this is ugly
        if config.imu_yaw.is_some() &&
            config.imu_pitch.is_some() &&
            config.imu_roll.is_some() {
            self.set_imu_angles(config.imu_yaw.unwrap(), config.imu_pitch.unwrap(), config.imu_roll.unwrap());
        }

        // Lever Arms
        if config.vrp2antenna.is_some() {
            // TODO: replace 0 with a proper constant
            self.set_lever_arm(0, &config.vrp2antenna.unwrap());
        }

        if config.vrp2imu.is_some() {
            // TODO: replace 1 with a proper constant
            self.set_lever_arm(1, &config.vrp2imu.unwrap());
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

        let mut set = UbxCfgRstAction::new();
        set.cold_start();
        self.server.fire_and_forget(&set);

        // Cold Start is not acknowledged, give receiver time to boot
        // before commanding next messages
        thread::sleep(time::Duration::from_millis(200));
    }    

    pub fn factory_reset(&mut self) {
        println!("factory-reset");
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

    // TODO const for arm type 0 = VRP-to-Ant, 1 = VRP_to_IMU
    fn set_lever_arm(&mut self, armtype: u8, distances: &Xyz) {
        let mut set = UbxCfgEsflaSet::new();

        assert!(distances.x >= -20.0 && distances.x <= 20.0);
        assert!(distances.y >= -10.0 && distances.y <= 10.0);
        assert!(distances.z >= -10.0 && distances.z <= 10.0);

        set.data.version = 0;
        set.data.num_configs = 1;
        set.data.leverarm_type = armtype; 
        set.data.leverarm_x = (distances.x * 100.0) as i16;
        set.data.leverarm_y = (distances.y * 100.0) as i16;
        set.data.leverarm_z = (distances.z * 100.0) as i16;
        println!("new lever arm settings {:?}", set.data);

        self.server.set(&set);
    }
}
