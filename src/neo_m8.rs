use chrono::prelude::*;
use log::{debug, info};
use std::collections::HashMap;
use std::{thread, time};

use crate::config_file::Xyz;
use crate::server_tty::ServerTty;
use crate::ubx_cfg_cfg::UbxCfgCfgAction;
use crate::ubx_cfg_esfalg::{UbxCfgEsfAlg, UbxCfgEsfAlgPoll};
use crate::ubx_cfg_esfla::UbxCfgEsflaSet;
use crate::ubx_cfg_nav5::{UbxCfgNav5, UbxCfgNav5Poll};
use crate::ubx_cfg_nmea::{UbxCfgNmea, UbxCfgNmeaPoll};
use crate::ubx_cfg_prt::{UbxCfgPrtPoll, UbxCfgPrtUart};
use crate::ubx_cfg_rate::{UbxCfgRate, UbxCfgRatePoll};
use crate::ubx_cfg_rst::UbxCfgRstAction;
use crate::ubx_mga_init_time_utc::UbxMgaIniTimeUtc;
use crate::ubx_mon_ver::{UbxMonVer, UbxMonVerPoll};
use crate::ubx_upd_sos::UbxUpdSosAction;

// TODO: Return error code from each function

pub struct NeoM8 {
    pub device_name: String,
    server: ServerTty,
}

impl NeoM8 {
    pub fn new(device: &str) -> Self {
        Self {
            device_name: String::from(device),
            server: ServerTty::new(device),
        }
    }

    pub fn detect_baudrate(&mut self) -> Result<usize, &'static str> {
        self.server.detect_baudrate()
    }

    pub fn open(&mut self, bitrate: usize) -> Result<(), &'static str> {
        self.server.open(bitrate)
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

        info!(
            "Reset GNSS receiver configuration to default, let receiver start with default config"
        );

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
        assert!(baudrate == 115200 || baudrate == 9600);

        let mut set = UbxCfgPrtUart::new();
        let poll = UbxCfgPrtPoll::new();

        self.server.poll(&poll, &mut set);
        // debug!("current settings {:?}", set);

        if set.data.baudrate != baudrate {
            info!("setting baudrate to {} bps", baudrate);
            set.data.baudrate = baudrate;
            debug!("new settings {:?}", set);

            self.server.fire_and_forget(&set);
            thread::sleep(time::Duration::from_millis(200));
        }
    }

    pub fn set_update_rate(&mut self, rate_in_hz: u16) {
        assert!(rate_in_hz >= 1 && rate_in_hz <= 10);

        let mut set = UbxCfgRate::new();
        let poll = UbxCfgRatePoll::new();

        self.server.poll(&poll, &mut set);
        // debug!("current settings {:?}", set);

        let new_time = 1000u16 / rate_in_hz;
        if set.data.meas_rate != new_time {
            info!("setting update rate to {} ms", new_time);
            set.data.meas_rate = new_time;
            debug!("new settings {:?}", set);

            self.server.set(&set);
        }
    }

    // TODO: Consider format of version parameter. The hex code is a bit too close to the UBX frame definition
    // enum ? string ?
    pub fn set_nmea_protocol_version(&mut self, version: &str) {
        let ubx_ver = match version {
            "4.0" => 0x40,
            "4.1" => 0x41,
            "4.11" => 0x4b,
            _ => panic!("invalid nmea protocol version"),
        };

        let mut set = UbxCfgNmea::new();
        let poll = UbxCfgNmeaPoll::new();

        self.server.poll(&poll, &mut set);
        // debug!("current settings {:?}", set);

        if set.data.nmea_version != ubx_ver {
            info!("setting NMEA protocol version to {}", version);
            set.data.nmea_version = ubx_ver;
            debug!("new settings {:?}", set);
            self.server.set(&set);
        }
    }

    pub fn set_dynamic_mode(&mut self, model: u8) {
        assert!(model <= 10 && model != 1);

        let mut set = UbxCfgNav5::new();
        let poll = UbxCfgNav5Poll::new();

        self.server.poll(&poll, &mut set);
        // debug!("current settings {:?}", set.data);

        if set.data.dyn_model != model {
            info!("setting dynamic model to {}", model);
            set.data.dyn_model = model;
            debug!("new settings {:?}", set.data);
            self.server.set(&set);
        }
    }

    pub fn set_imu_angles(&mut self, yaw: u16, pitch: i16, roll: i16) {
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
    pub fn set_lever_arm(&mut self, armtype: u8, distances: &Xyz) {
        let mut set = UbxCfgEsflaSet::new();

        assert!(distances.x >= -30.0 && distances.x <= 30.0);
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
