use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;

use log::{debug, info, warn};
use clap::{ArgMatches};

use crate::neo_m8::NeoM8;
use crate::config_file::{GnssMgrConfig};


static CURRENT_FW_VER: &str = "ADR 4.31";


// TODO: Return error code from each function

pub struct GnssMgr {
    device_name: String,
    modem: NeoM8,
}

impl GnssMgr {
    pub fn new(device: &str) -> Self {
        Self {
            device_name: String::from(device),
            modem: NeoM8::new(device),
        }
    }

    pub fn prepare_port(&mut self) -> Result<usize, String> {
        // NOTE: To be calledonly  before GnssMgr object is instantiated
        // TODO: Refactor to use modem object of GnssMgr

        // Check bitrate and change to 115'200 if different
        #![allow(unused_mut)] // rustc incorectly complains about "mut"
        let mut bit_rate_current;
        
        // let mut detector = DetectBaudrate::new(&self.device_name);
        // let res = detector.exec();
        let res = self.modem.detect_baudrate();
        match res {
            Ok(bitrate) => {
                info!("detected bitrate {:?} bps", bitrate);
                bit_rate_current = bitrate;
            },
            Err(e) => // return Err(String::from(e)),
                return Err(format!("bitrate detection failed, {}", e).to_string()),
        }
    
        if bit_rate_current == 9600 {
            info!("changing bitrate from {} to 115200 bps", bit_rate_current);
    
            self.modem.open(bit_rate_current)?;
            self.modem.set_baudrate(115200);
            return Ok(115200);
        }
/*
        else if bit_rate_current == 115200 {
            info!("changing bitrate from {} to 9600 bps", bit_rate_current);
    
            let mut modem = NeoM8::new(device_name);
            modem.open(bit_rate_current);
            modem.set_baudrate(9600);
            return Ok(9600);
        }
        else {
            return Err("unsupported bitrate".to_string());
        }
*/
        else {
            return Ok(115200);
        }
    }

    pub fn open(&mut self, bitrate: usize)-> Result<(), &'static str> {
        self.modem.open(bitrate)
    }

    pub fn run_init(&mut self, _matches: &ArgMatches) -> Result<(), String> {
        // create /run/gnss/gnss0.config
        let runfile_path = Self::build_runfile_path(&self.device_name);
    
        // vendor is always "ublox" when using this library
        let mut info: HashMap<&str, String> = HashMap::new();
        info.insert("vendor", String::from("ublox"));
    
        // Get version information and ..
        self.modem.version(&mut info);
        debug!("{:?}", info);
    
        // .. create run file
        match Self::write_runfile(&runfile_path, &info) {
            Ok(_) => info!("GNSS run file {} created", &runfile_path),
            Err(_) => { warn!("Error creating run file"); }, // TODO: return code on error
        }
    
        // Change protocol to NMEA 4.1
        // set_nmea_protocol_version
        self.modem.set_nmea_protocol_version("4.1");
    
        Ok(())
    }
    
    pub fn run_config(&mut self, matches: &ArgMatches) -> Result<(), String> {
        // Check for optional config file name
        let configfile_path = matches.value_of("configfile");
        let configfile_path: String = match configfile_path {
            Some(path) => path.to_string(),                     // path to file specified
            _ => Self::build_configfile_path(&self.device_name),      // left away, compute from device name
        };
    
        info!("using configfile {}", configfile_path);
    
        // Get configuration from config file
        let mut config: GnssMgrConfig = Default::default();
        let _res = config.parse_config(&configfile_path)?;
    
        self.configure(&config);
    
        Ok(())
    }
    
    pub fn run_control(&mut self, matches: &ArgMatches) -> Result<(), String> {
        let action = matches.value_of("action").unwrap();
        debug!("control action {:?}", action);
    
        match action {
            "cold-start" => self.modem.cold_start(),
            "factory-reset" => self.modem.factory_reset(),
            "persist" => self.modem.persist(),
            _ => return Err("Unknown command".to_string()),
        }
    
        Ok(())
    }
    
    pub fn run_sos(&mut self, matches: &ArgMatches) -> Result<(), String> {
        let action = matches.value_of("action").unwrap();
        debug!("sos action {:?}", action);
    
        match action {
            "save" => self.modem.sos_save(),
            "clear" => {
                self.modem.set_assistance_time();
                self.modem.sos_clear();
            },
            _ => return Err("Unknown command".to_string()),
        }
    
        Ok(())
    }
    
    pub fn configure(&mut self, config: &GnssMgrConfig) {
        if config.update_rate.is_some() {
            let rate = config.update_rate.unwrap();
            self.modem.set_update_rate(rate);
        }

        // TODO: Overly complicated with these string types...
        match &config.mode {
            Some(mode) => {
                match mode.as_str() {
                    "stationary" => self.modem.set_dynamic_mode(2),
                    "vehicle" => self.modem.set_dynamic_mode(4),
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
            self.modem.set_imu_angles(config.imu_yaw.unwrap(), config.imu_pitch.unwrap(), config.imu_roll.unwrap());
        }

        // Lever Arms
        if config.vrp2antenna.is_some() {
            // TODO: replace 0 with a proper constant
            self.modem.set_lever_arm(0, &config.vrp2antenna.unwrap());
        }

        if config.vrp2imu.is_some() {
            // TODO: replace 1 with a proper constant
            self.modem.set_lever_arm(1, &config.vrp2imu.unwrap());
        }
    }

    fn build_runfile_path(path: &str) -> String {
        // Take devicename of form /dev/<name> to build /run/gnss/<name>.config
        let path = &path.replace("/dev/", "/run/gnss/");
        let mut path = String::from(path);
        path.push_str(".config");
        path
        //let owner = Path::new(&path);
        // path.as_ref()
    }
    
    fn write_runfile(path: &str, info: &HashMap<&str, String>) -> Result<(), &'static str> {
        let path = Path::new(path);
        // let display = path.display();
        let mut file = match File::create(&path) {
            Err(_) => return Err("Can't create GNSS run file"),
            Ok(file) => file,
        };
    
        let deprecated = if info["fw_ver"] != CURRENT_FW_VER {
            " (Deprecated)"
        }
        else {
            ""
        };
    
        let text = format!(
            "Vendor:                             {}\n\
            Model:                              {}\n\
            Firmware:                           {}{}\n\
            ubx-Protocol:                       {}\n\
            Supported Satellite Systems:        {}\n\
            Supported Augmentation Services:    {}\n\
            SW Version:                         {}\n\
            HW Version:                         {}\n",
            info["vendor"],
            info["model"],
            info["fw_ver"], deprecated,
            info["protocol"],
            info["systems"],
            info["augmentation"],
            info["sw_ver"],
            info["hw_ver"],
        );
    
        match file.write_all(text.as_bytes()) {
            Err(_) => Err("Can't write GNSS run file"),
            Ok(_) => Ok(()),
        }
    }
    
    // TODO: return Path instead of String
    fn build_configfile_path(path: &str) -> String {
        // Take devicename of form /dev/<name> to build /etc/gnss/<name>
        let path = &path.replace("/dev/", "/etc/gnss/");
        let mut path = String::from(path);
        path.push_str(".conf");
        path
    }
}
