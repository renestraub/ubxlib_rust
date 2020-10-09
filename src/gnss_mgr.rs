use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use clap::ArgMatches;
use log::{debug, info};

use crate::config_file::GnssMgrConfig;
use crate::error::Error;
use crate::neo_m8::NeoM8;
use crate::ubx_cfg_esfla::LeverArmType;

static CURRENT_FW_VER: &str = "ADR 4.31";

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

    pub fn prepare_port(&mut self, _bitrate: Option<u32>) -> Result<(), String> {
        if _bitrate.is_none() {
            // Check bitrate and change to 115'200 if different
            info!("detecting current bitrate");

            let bit_rate_current = match self.modem.detect_baudrate() {
                Ok(bitrate) => bitrate,
                Err(e) => return Err(format!("bitrate detection failed ({})", e)),
            };

            info!("detected bitrate {:?} bps", bit_rate_current);
            if bit_rate_current != 115200 {
                info!("changing bitrate from {} to 115200 bps", bit_rate_current);
                self.modem
                    .configure(bit_rate_current)
                    .map_err(|err| err.to_string())?;
                self.modem
                    .set_modem_baudrate(115200)
                    .map_err(|err| err.to_string())?;
            }
        }

        self.modem
            .configure(115200)
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    pub fn run_init(&mut self, _matches: &ArgMatches) -> Result<(), String> {
        // vendor is always "ublox" when using this library
        let mut info: HashMap<&str, String> = HashMap::new();
        info.insert("vendor", String::from("ublox"));

        // Get version information and ..
        info!("getting modem information");
        self.modem
            .version(&mut info)
            .map_err(|e| format!("can't get modem information ({})", e))?;

        // .. create run file
        let runfile_path = Self::build_runfile_path(&self.device_name);
        Self::write_runfile(&runfile_path, &info)
            .map_err(|e| format!("can't create run file ({})", e))?;

        info!("GNSS run file {} created", runfile_path.display());

        // Change protocol to NMEA 4.1
        info!("setting nmea version");
        self.modem
            .set_nmea_protocol_version("4.1")
            .map_err(|e| format!("can't set NMEA protocol version ({})", e))?;

        Ok(())
    }

    pub fn run_config(&mut self, matches: &ArgMatches) -> Result<(), String> {
        // Check for optional config file name
        let configfile_path = matches.value_of("configfile");
        let configfile_path = match configfile_path {
            Some(path) => PathBuf::from(path), // path to file specified
            _ => Self::build_configfile_path(&self.device_name), // left away, compute from device name
        };

        info!("using configfile {}", configfile_path.display());

        // Get configuration from config file
        let mut config: GnssMgrConfig = Default::default();
        let _res = config.parse_config(&configfile_path)?;

        info!("configuring modem");

        self.configure(&config)
            .map_err(|e| format!("configuration failed ({})", e))?;

        Ok(())
    }

    pub fn run_control(&mut self, matches: &ArgMatches) -> Result<(), String> {
        let action = matches.value_of("action").unwrap();
        debug!("control action {:?}", action);

        match action {
            "cold-start" => {
                info!("Cold boot of GNSS receiver triggered, let receiver start");
                self.modem.cold_start().map_err(|err| err.to_string())?
            }
            "factory-reset" => {
                info!(
                    "Reset GNSS receiver configuration to default, let receiver start with default config"
                );
                self.modem.factory_reset().map_err(|err| err.to_string())?
            }
            "persist" => {
                info!("Persisting receiver configuration");
                self.modem.persist().map_err(|err| err.to_string())?
            }
            _ => return Err("Unknown command".to_string()),
        };

        Ok(())
    }

    pub fn run_sos(&mut self, matches: &ArgMatches) -> Result<(), String> {
        let action = matches.value_of("action").unwrap();
        debug!("sos action {:?}", action);

        match action {
            "save" => {
                self.modem.sos_save().map_err(|err| err.to_string())?;
                info!("Saving receiver state successfully performed");
                Some(())
            }
            "clear" => {
                self.modem
                    .set_assistance_time()
                    .map_err(|err| err.to_string())?;

                match self.modem.sos_check() {
                    Ok(_) => {
                        self.modem.sos_clear().map_err(|err| err.to_string())?;
                        info!("Clearing receiver state successfully performed");
                    }
                    Err(Error::ModemNobackup) => {
                        info!("No backup found");
                    }
                    Err(_) => {
                        info!("Problem with backup, clearing state");
                        self.modem.sos_clear().map_err(|err| err.to_string())?;
                    }
                }
                Some(())
            }
            _ => return Err("Unknown command".to_string()),
        };

        Ok(())
    }

    fn configure(&mut self, config: &GnssMgrConfig) -> Result<(), String> {
        /*
         * Configure modem as defined by config
         * - Elements that are set (Some(x)) are applied, others are left as is.
         * - Operations must work. On the first error the method aborts.
         */
        if let Some(rate) = config.update_rate {
            self.modem
                .set_update_rate(rate as u16)
                .map_err(|err| err.to_string())?;
        }

        if let Some(mode) = &config.mode {
            // TODO: Move this decoding logic into set_dynamic_mode()?
            match mode.as_str() {
                "stationary" => {
                    self.modem
                        .set_dynamic_mode(2)
                        .map_err(|err| err.to_string())?;
                }
                "vehicle" => {
                    self.modem
                        .set_dynamic_mode(4)
                        .map_err(|err| err.to_string())?;
                }
                _ => return Err(format!("invalid mode {}", mode)),
            }
        }

        // Set Satellite systems
        if let Some(systems) = &config.systems {
            match self.modem.set_systems(systems) {
                Ok(_) => (),
                Err(Error::ModemNAK) => {
                    // warn!("failed to configure satellite systems {:?}", systems)
                    return Err(format!("invalid systems combination {:?}", systems))
                }
                Err(e) => return Err(e.to_string()),
            }
        }

        // IMU Orientation
        if let Some(angles) = config.imu_angles {
            self.modem
                .set_imu_angles(angles)
                .map_err(|err| err.to_string())?;
        }

        // Lever Arms
        if let Some(xyz) = config.vrp2antenna {
            self.modem
                .set_lever_arm(LeverArmType::VRPtoAntenna, &xyz)
                .map_err(|err| err.to_string())?;
        }

        if let Some(xyz) = config.vrp2imu {
            self.modem
                .set_lever_arm(LeverArmType::VRPtoIMU, &xyz)
                .map_err(|err| err.to_string())?;
        }

        Ok(())
    }

    fn write_runfile(path: &Path, info: &HashMap<&str, String>) -> Result<(), String> {
        let deprecated = if info["fw_ver"] != CURRENT_FW_VER {
            " (Deprecated)"
        } else {
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
            info["fw_ver"],
            deprecated,
            info["protocol"],
            info["systems"],
            info["augmentation"],
            info["sw_ver"],
            info["hw_ver"],
        );

        // let path = Path::new(path);
        let parent = path.parent().unwrap();

        fs::create_dir_all(parent)
            .map_err(|_err| format!("can't create GNSS run file folder {}", parent.display()))?;

        let mut file = File::create(&path)
            .map_err(|_err| format!("can't create GNSS run file {}", path.display()))?;
        file.write_all(text.as_bytes())
            .map_err(|_err| format!("can't write GNSS run file"))?;

        Ok(())
    }

    fn build_runfile_path(path: &str) -> PathBuf {
        // Take devicename of form /dev/<name> to build /run/gnss/<name>.config
        let path = &path.replace("/dev/", "/run/gnss/");
        let mut path = PathBuf::from(path);
        path.set_extension("config");
        path.to_path_buf()
    }

    fn build_configfile_path(path: &str) -> PathBuf {
        let path = &path.replace("/dev/", "/etc/gnss/");
        let mut path = PathBuf::from(path);
        path.set_extension("conf");
        path.to_path_buf()
    }
}
