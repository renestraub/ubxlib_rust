use chrono::prelude::*;
use log::{debug};
use std::collections::HashMap;
use std::{thread, time};

use crate::config_file::Xyz;
use crate::error::Error;
use crate::server_tty::ServerTty;
use crate::ubx_cfg_cfg::UbxCfgCfgAction;
use crate::ubx_cfg_esfalg::{UbxCfgEsfAlg, UbxCfgEsfAlgPoll};
use crate::ubx_cfg_esfla::UbxCfgEsflaSet;
use crate::ubx_cfg_gnss::{UbxCfgGnss, UbxCfgGnssPoll, SystemName};
use crate::ubx_cfg_nav5::{UbxCfgNav5, UbxCfgNav5Poll};
use crate::ubx_cfg_nmea::{UbxCfgNmea, UbxCfgNmeaPoll};
use crate::ubx_cfg_prt::{UbxCfgPrtPoll, UbxCfgPrtUart};
use crate::ubx_cfg_rate::{UbxCfgRate, UbxCfgRatePoll};
use crate::ubx_cfg_rst::UbxCfgRstAction;
use crate::ubx_mga_init_time_utc::UbxMgaIniTimeUtc;
use crate::ubx_mon_ver::{UbxMonVer, UbxMonVerPoll};
use crate::ubx_upd_sos::UbxUpdSosAction;


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

    pub fn detect_baudrate(&mut self) -> Result<usize, Error> {
        const BITRATES: [usize; 2] = [115200, 9600];

        for baud in BITRATES.iter() {
            debug!("checking {} bps", baud);

            self.server.set_baudrate(*baud)?;

            // try to receive ubx or NMEA frames at configured bitrate
            match self.server.scan() {
                Ok(_) => {
                    return Ok(*baud);
                }
                Err(_) => {
                    debug!("bitrate {:?} not working", baud);
                    ();
                }
            }
        }

        Err(Error::BaudRateDetectionFailed)
    }

    pub fn configure(&mut self, bitrate: usize) -> Result<(), Error> {
        self.server.set_baudrate(bitrate)
    }

    pub fn version(&mut self, info: &mut HashMap<&str, String>) -> Result<(), Error> {
        let mut ver_result = UbxMonVer::new();
        let poll = UbxMonVerPoll::new();
        self.server.poll(&poll, &mut ver_result)?;
        debug!("{:#?}", ver_result);

        let fw_ver = ver_result.get_info("FWVER=");
        let proto = ver_result.get_info("PROTVER=");
        let model = ver_result.get_info("MOD=");

        info.insert("model", model);
        info.insert("sw_ver", ver_result.sw_version());
        info.insert("hw_ver", ver_result.hw_version());
        info.insert("fw_ver", fw_ver);
        info.insert("protocol", proto);
        info.insert("systems", String::from(&ver_result.get_ext(5)));
        info.insert("augmentation", String::from(&ver_result.get_ext(6)));

        Ok(())
    }

    pub fn sos_save(&mut self) -> Result<(), Error> {
        // Stop receiver
        let set = UbxCfgRstAction::stop();
        self.server.fire_and_forget(&set)?;

        // Stop is not acknowledged, give receiver time to execute
        // request before commanding next messages
        thread::sleep(time::Duration::from_millis(200));

        let set = UbxUpdSosAction::backup();
        self.server.set(&set)?;

        Ok(())
    }

    pub fn sos_clear(&mut self) -> Result<(), Error> {
        let set = UbxUpdSosAction::clear();
        self.server.set(&set)?;

        Ok(())
    }

    pub fn cold_start(&mut self) -> Result<(), Error> {
        let set = UbxCfgRstAction::cold_start();
        self.server.fire_and_forget(&set)?;

        // Cold Start is not acknowledged, give receiver time to boot
        // before commanding next message
        thread::sleep(time::Duration::from_millis(200));

        Ok(())
    }

    pub fn factory_reset(&mut self) -> Result<(), Error> {
        let set = UbxCfgCfgAction::factory_reset();
        self.server.fire_and_forget(&set)?;

        // Factory reset can lead to change of bitrate, no acknowledge can be received then
        // Give receiver time before commanding next message
        thread::sleep(time::Duration::from_millis(200));

        Ok(())
    }

    pub fn persist(&mut self) -> Result<(), Error> {
        let set = UbxCfgCfgAction::persist();
        self.server.set(&set)?;

        Ok(())
    }

    pub fn set_modem_baudrate(&mut self, baudrate: u32) -> Result<(), Error> {
        assert!(baudrate == 115200 || baudrate == 9600);

        let mut set = UbxCfgPrtUart::new();
        let poll = UbxCfgPrtPoll::new();
        self.server.poll(&poll, &mut set)?;

        if set.data.baudrate != baudrate {
            debug!("setting baudrate to {} bps", baudrate);
            set.data.baudrate = baudrate;
            debug!("new settings {:?}", set);

            self.server.fire_and_forget(&set)?;
            thread::sleep(time::Duration::from_millis(200));
        }

        Ok(())
    }

    pub fn set_update_rate(&mut self, rate_in_hz: u16) -> Result<(), Error> {
        assert!(rate_in_hz >= 1 && rate_in_hz <= 10);

        let mut set = UbxCfgRate::new();
        let poll = UbxCfgRatePoll::new();
        self.server.poll(&poll, &mut set)?;

        let new_time = 1000u16 / rate_in_hz;
        if set.data.meas_rate != new_time {
            debug!("setting update rate to {} ms", new_time);
            set.data.meas_rate = new_time;
            debug!("new settings {:?}", set);

            self.server.set(&set)?;
        }

        Ok(())
    }

    pub fn set_nmea_protocol_version(&mut self, version: &str) -> Result<(), Error> {
        let ubx_ver = match version {
            "4.0" => 0x40,
            "4.1" => 0x41,
            "4.11" => 0x4b,
            _ => panic!("invalid nmea protocol version"),
        };

        let mut set = UbxCfgNmea::new();
        let poll = UbxCfgNmeaPoll::new();
        self.server.poll(&poll, &mut set)?;

        if set.data.nmea_version != ubx_ver {
            debug!("setting NMEA protocol version to {}", version);
            set.data.nmea_version = ubx_ver;
            debug!("new settings {:?}", set);
            self.server.set(&set)?;
        }

        Ok(())
    }

    pub fn set_dynamic_mode(&mut self, model: u8) -> Result<(), Error> {
        assert!(model <= 10 && model != 1);

        let mut set = UbxCfgNav5::new();
        let poll = UbxCfgNav5Poll::new();
        self.server.poll(&poll, &mut set)?;

        if set.data.dyn_model != model {
            debug!("setting dynamic model to {}", model);
            set.data.dyn_model = model;
            debug!("new settings {:?}", set.data);
            self.server.set(&set)?;
        }

        Ok(())
    }

    pub fn set_systems(&mut self, systems: &Vec<String>) -> Result<(), Error> {
        let mut set = UbxCfgGnss::new();
        let poll = UbxCfgGnssPoll::new();
        self.server.poll(&poll, &mut set)?;

        // info!("setting navigation systems {:?}", systems);

        set.disable_all();
        if systems.contains(&String::from("gps")) {
            set.enable(SystemName::Gps);
        }
        if systems.contains(&String::from("sbas")) {
            set.enable(SystemName::Sbas);
        }
        if systems.contains(&String::from("galileo")) {
            set.enable(SystemName::Galileo);
        }
        if systems.contains(&String::from("beidou")) {
            set.enable(SystemName::Beidou);
        }
        if systems.contains(&String::from("imes")) {
            set.enable(SystemName::Imes);
        }
        if systems.contains(&String::from("qzss")) {
            set.enable(SystemName::Qzss);
        }
        if systems.contains(&String::from("glonass")) {
            set.enable(SystemName::Glonass);
        }

        // If an invalid system combination is selected this can fail.
        // Caller should handle this case.
        self.server.set(&set)?;

        Ok(())
    }

    pub fn set_imu_angles(&mut self, yaw: u16, pitch: i16, roll: i16) -> Result<(), Error> {
        assert!(yaw <= 360);
        assert!(pitch >= -90 && pitch <= 90);
        assert!(roll >= -180 && roll <= 180);

        let mut set = UbxCfgEsfAlg::new();
        let poll = UbxCfgEsfAlgPoll::new();
        self.server.poll(&poll, &mut set)?;

        set.data.yaw = yaw as u32 * 100;
        set.data.pitch = pitch as i16 * 100;
        set.data.roll = roll as i16 * 100;
        debug!("new IMU settings {:?}", set.data);

        self.server.set(&set)?;

        Ok(())
    }

    // TODO const for arm type 0 = VRP-to-Ant, 1 = VRP_to_IMU
    pub fn set_lever_arm(&mut self, armtype: u8, distances: &Xyz) -> Result<(), Error> {
        assert!(distances.x >= -30.0 && distances.x <= 30.0);
        assert!(distances.y >= -10.0 && distances.y <= 10.0);
        assert!(distances.z >= -10.0 && distances.z <= 10.0);

        let mut set = UbxCfgEsflaSet::new();
        set.data.version = 0;
        set.data.num_configs = 1;
        set.data.leverarm_type = armtype;
        set.data.leverarm_x = (distances.x * 100.0) as i16;
        set.data.leverarm_y = (distances.y * 100.0) as i16;
        set.data.leverarm_z = (distances.z * 100.0) as i16;
        debug!("new lever arm settings {:?}", set.data);

        self.server.set(&set)?;

        Ok(())
    }

    pub fn set_assistance_time(&mut self) -> Result<(), Error> {
        let utc: DateTime<Utc> = Utc::now();
        debug!("Setting GNSS time to {:?}", utc);

        let mut set = UbxMgaIniTimeUtc::new();
        set.set_date_time(&utc);

        self.server.fire_and_forget(&set)?;
        // MGA messages are not acked by default
        // Would have to enable with NAVX5 message

        Ok(())
    }
}
