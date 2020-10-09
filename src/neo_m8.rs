use chrono::prelude::*;
use log::debug;
use std::collections::HashMap;
use std::{thread, time};

use crate::config_file::{Angles, Xyz};
use crate::error::Error;
use crate::server_tty::ServerTty;
use crate::ubx_cfg_cfg::UbxCfgCfgAction;
use crate::ubx_cfg_esfalg::{UbxCfgEsfAlg, UbxCfgEsfAlgPoll};
use crate::ubx_cfg_esfla::{LeverArmType, UbxCfgEsflaSet};
use crate::ubx_cfg_gnss::{SystemName, UbxCfgGnss, UbxCfgGnssPoll};
use crate::ubx_cfg_nav5::{UbxCfgNav5, UbxCfgNav5Poll};
use crate::ubx_cfg_nmea::{UbxCfgNmea, UbxCfgNmeaPoll};
use crate::ubx_cfg_prt::{UbxCfgPrtPoll, UbxCfgPrtUart};
use crate::ubx_cfg_rate::{UbxCfgRate, UbxCfgRatePoll};
use crate::ubx_cfg_rst::UbxCfgRstAction;
use crate::ubx_mga_init_time_utc::UbxMgaIniTimeUtc;
use crate::ubx_mon_ver::{UbxMonVer, UbxMonVerPoll};
use crate::ubx_upd_sos::{Response, UbxUpdSos, UbxUpdSosAction, UbxUpdSosPoll};

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

    pub fn sos_check(&mut self) -> Result<(), Error> {
        let mut set = UbxUpdSos::new();
        let poll = UbxUpdSosPoll::new();
        self.server.poll(&poll, &mut set)?;
        debug!("SoS State reported is {:?}", set.data.response);

        match set.data.response {
            Response::Restored => Ok(()),
            Response::NotRestoredNoBackup => Err(Error::ModemNobackup),
            _ => Err(Error::ModemBackupRestoreFailed),
        }
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
        if baudrate != 115200 && baudrate != 9600 {
            return Err(Error::InvalidArgument);
        }

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
        if rate_in_hz < 1 || rate_in_hz > 10 {
            return Err(Error::InvalidArgument);
        }

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
            _ => return Err(Error::InvalidArgument),
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

    pub fn set_dynamic_mode(&mut self, model: &str) -> Result<(), Error> {
        let model = match model {
            "stationary" => 2,
            "vehicle" => 4,
            _ => return Err(Error::InvalidArgument),
        };

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

    pub fn set_imu_angles(&mut self, angles: Angles) -> Result<(), Error> {
        if angles.yaw > 360 || angles.pitch.abs() > 90 || angles.roll.abs() > 180 {
            return Err(Error::InvalidArgument);
        }

        let mut set = UbxCfgEsfAlg::new();
        let poll = UbxCfgEsfAlgPoll::new();
        self.server.poll(&poll, &mut set)?;

        set.data.yaw = angles.yaw * 100;
        set.data.pitch = angles.pitch * 100;
        set.data.roll = angles.roll * 100;
        debug!("new IMU settings {:?}", set.data);

        self.server.set(&set)?;

        Ok(())
    }

    pub fn set_lever_arm(&mut self, armtype: LeverArmType, distances: &Xyz) -> Result<(), Error> {
        if distances.x.abs() > 30.0 || distances.y.abs() > 10.0 || distances.z.abs() > 10.0 {
            return Err(Error::InvalidArgument);
        }

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
