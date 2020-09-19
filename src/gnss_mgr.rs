use std::{thread, time};
use std::collections::HashMap;
use log::{debug, info};
use chrono::prelude::*;

use crate::config_file::{GnssMgrConfig, Xyz};
use crate::server_tty::ServerTty;
use crate::ubx_cfg_rate::{UbxCfgRate, UbxCfgRatePoll};
use crate::ubx_cfg_nmea::{UbxCfgNmea, UbxCfgNmeaPoll};
use crate::ubx_mon_ver::{UbxMonVer, UbxMonVerPoll};
use crate::ubx_cfg_nav5::{UbxCfgNav5, UbxCfgNav5Poll};
use crate::ubx_cfg_rst::UbxCfgRstAction;
use crate::ubx_cfg_cfg::UbxCfgCfgAction;
use crate::ubx_cfg_prt::{UbxCfgPrtPoll, UbxCfgPrtUart};
use crate::ubx_cfg_esfalg::{UbxCfgEsfAlg, UbxCfgEsfAlgPoll};
use crate::ubx_cfg_esfla::UbxCfgEsflaSet;
use crate::ubx_upd_sos::UbxUpdSosAction;
use crate::ubx_mga_init_time_utc::UbxMgaIniTimeUtc;


// TODO: Split away neom8 driver module

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
    pub fn new(device: &str, bitrate: usize) -> Self {
        Self {
            device_name: String::from(device),
            server: ServerTty::new(device, bitrate),
        }
    }

    pub fn version(&mut self, info: &mut HashMap<&str, String>) {
        let mut ver_result = UbxMonVer::new();
        let poll = UbxMonVerPoll::new();

        // TODO: error check missing for sure
        self.server.poll(&poll, &mut ver_result);
        debug!("{:?}", ver_result);

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
        if config.update_rate.is_some() {
            let rate = config.update_rate.unwrap();
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
        // Stop receiver
        let set = UbxCfgRstAction::stop();
        self.server.fire_and_forget(&set);
        // Stop is not acknowledged, give receiver time to execute
        // request before commanding next messages
        thread::sleep(time::Duration::from_millis(200));

        let set = UbxUpdSosAction::backup();
        self.server.set(&set);
        info!("Saving receiver state successfully performed");
    }

    pub fn sos_clear(&mut self) {
        let set = UbxUpdSosAction::clear();
        self.server.set(&set);
        info!("Clearing receiver state successfully performed");
    }

    pub fn cold_start(&mut self) {
        let set = UbxCfgRstAction::cold_start();
        self.server.fire_and_forget(&set);

        info!("Cold boot of GNSS receiver triggered, let receiver start");

        // Cold Start is not acknowledged, give receiver time to boot
        // before commanding next message
        thread::sleep(time::Duration::from_millis(200));
    }

    pub fn factory_reset(&mut self) {
        let set = UbxCfgCfgAction::factory_reset();
        self.server.fire_and_forget(&set);

        info!("Reset GNSS receiver configuration to default, let receiver start with default config");

        // Factory reset can lead to change of bitrate, no acknowledge can be received then
        // Give receiver time before commanding next message
        thread::sleep(time::Duration::from_millis(200));
    }

    pub fn persist(&mut self) {
        info!("Persisting receiver configuration");

        let set = UbxCfgCfgAction::persist();
        self.server.set(&set);
    }

    pub fn set_baudrate(&mut self, baudrate: u32) {
        let mut set = UbxCfgPrtUart::new();
        let poll = UbxCfgPrtPoll::new();

        self.server.poll(&poll, &mut set);
        // debug!("current settings {:?}", set);

        if set.data.baudrate != baudrate {
            info!("setting baudrate {} bps", baudrate);
            set.data.baudrate = baudrate;
            debug!("new settings {:?}", set);

            self.server.fire_and_forget(&set);
            thread::sleep(time::Duration::from_millis(200));
        }
    }

    fn set_update_rate(&mut self, rate: u16) {
        let mut set = UbxCfgRate::new();
        let poll = UbxCfgRatePoll::new();

        self.server.poll(&poll, &mut set);
        // debug!("current settings {:?}", set);

        let new_time = 1000u16 / rate;
        if set.data.meas_rate != new_time {
            info!("setting update rate to {} ms", new_time);
            set.data.meas_rate = new_time;
            debug!("new settings {:?}", set);

            self.server.set(&set);
        }
    }

    // TODO: make all these public?
    // TODO: Consider format of version parameter. The hex code is a bit too close to the UBX frame definition
    // enum ? string ?
    pub fn set_nmea_protocol_version(&mut self, version: u8) {
        let mut set = UbxCfgNmea::new();
        let poll = UbxCfgNmeaPoll::new();

        self.server.poll(&poll, &mut set);
        // debug!("current settings {:?}", set);

        if set.data.nmea_version != version {
            info!("setting NMEA protocol version to 0x{:02X}", version);
            set.data.nmea_version = version;
            debug!("new settings {:?}", set);
            self.server.set(&set);
        }
    }

    fn set_dynamic_mode(&mut self, model: u8) {
        let mut set = UbxCfgNav5::new();
        let poll = UbxCfgNav5Poll::new();

        self.server.poll(&poll, &mut set);
        // debug!("current settings {:?}", set.data);

        if true || set.data.dyn_model != model {
            info!("setting dynamic model to {}", model);
            set.data.dyn_model = model;
            debug!("new settings {:?}", set.data);
            self.server.set(&set);
        }
    }

    fn set_imu_angles(&mut self, yaw: u16, pitch: i16, roll: i16) {
        assert!(yaw <= 360);
        assert!(pitch >= -90 && pitch <= 90);
        assert!(roll >= -180 && roll <= 180);

        let mut set = UbxCfgEsfAlg::new();
        let poll = UbxCfgEsfAlgPoll::new();

        self.server.poll(&poll, &mut set);
        debug!("current IMU settings {:?}", set.data);

        set.data.yaw = yaw as u32 * 100;
        set.data.pitch = pitch as i16 * 100;
        set.data.roll = roll as i16 * 100;
        debug!("new IMU settings {:?}", set.data);

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
        debug!("new lever arm settings {:?}", set.data);

        self.server.set(&set);
    }

    pub fn set_assistance_time(&mut self) {
        let utc: DateTime<Utc> = Utc::now();
        debug!("Setting GNSS time to {:?}", utc);

        let mut set = UbxMgaIniTimeUtc::new();
        set.set_date_time(&utc);

        self.server.fire_and_forget(&set);
        // MGA messages are not acked by default
        // Would have to enable with NAVX5 message
    }
}
