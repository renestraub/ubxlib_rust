use ini::Ini;
use log::info;

#[derive(Debug, Default)]
pub struct GnssMgrConfig {
    pub update_rate: Option<u16>,
    pub mode: Option<String>,
    pub systems: Option<Vec<String>>,
    pub imu_yaw: Option<u16>,
    pub imu_pitch: Option<i16>,
    pub imu_roll: Option<i16>,

    pub vrp2antenna: Option<Xyz>,
    pub vrp2imu: Option<Xyz>,
}

impl GnssMgrConfig {
    pub fn parse_config(&mut self, path: &str) -> Result<(), String> {
        // Import whole file, check for syntax errors
        let conf = match Ini::load_from_file(path) {
            Ok(c) => c,
            _ => return Err(format!("configuration file {} not found", path).to_string()),
        };

        // Check for version 2 format
        let sec_general = match conf.section(Some("default")) {
            Some(sec) => sec,
            _ => return Err("Invalid configuration file format/version".to_string()),
        };

        let _version = match sec_general.get("version") {
            Some("2") => 2,
            _ => return Err("Invalid configuration file format/version".to_string()),
        };

        // TODO: Combine in a nice getter with range check
        // Return Some(number) with valid content
        // or Err("....")
        let keyname = "update-rate";
        let value = match sec_general.get(keyname) {
            Some("") => {
                info!("no value for {} specified, ignoring", keyname);
                None
            }
            Some(x) => match x.parse::<u16>() {
                Ok(y) if y <= 2 => {
                    info!("using {} for {}", x, keyname);
                    Some(y)
                }
                Ok(_) | Err(_) => {
                    info!("invalid value {} for key {}", x, keyname);
                    None
                }
            },
            _ => {
                info!("key '{}' not defined", keyname);
                None
            }
        };
        self.update_rate = value;

        let sec_navigation = match conf.section(Some("navigation")) {
            Some(sec) => sec,
            _ => return Err("Invalid configuration file format/version".to_string()),
        };

        let keyname = "mode";
        let valid_args = vec!["stationary", "vehicle"];
        let value = match sec_navigation.get(keyname) {
            Some("") => {
                info!("no value for {} specified, ignoring", keyname);
                None
            }
            Some(x) if valid_args.contains(&x) => {
                info!("using {} for {}", x, keyname);
                Some(String::from(x))
            }
            _ => {
                info!("key '{}' not defined", keyname);
                None
            }
        };
        self.mode = value;

        let keyname = "systems";
        let value = match sec_navigation.get(keyname) {
            Some("") => {
                info!("no value for {} specified, ignoring", keyname);
                None
            }
            Some(x) => {
                info!("using {} for {}", x, keyname);
                let res: Vec<String> = x.split(";").map(|s| s.to_string().to_lowercase()).collect();
                // TODO: println!("split {:?}", res);
                Some(res)
            }
            _ => {
                info!("key '{}' not defined", keyname);
                None
            }
        };
        self.systems = value;


        let sec_installation = match conf.section(Some("installation")) {
            Some(sec) => sec,
            _ => return Err("Invalid configuration file format/version".to_string()),
        };

        let keyname = "yaw";
        let value = match sec_installation.get(keyname) {
            Some("") => {
                info!("no value for {} specified, ignoring", keyname);
                None
            }
            Some(x) => match x.parse::<u16>() {
                Ok(y) if y <= 360 => {
                    info!("using {} for {}", x, keyname);
                    Some(y)
                }
                Ok(_) | Err(_) => {
                    info!("invalid value {} for key {}", x, keyname);
                    None
                }
            },
            _ => {
                info!("key '{}' not defined", keyname);
                None
            }
        };
        self.imu_yaw = value;

        let keyname = "pitch";
        let value = match sec_installation.get(keyname) {
            Some("") => {
                info!("no value for {} specified, ignoring", keyname);
                None
            }
            Some(x) => match x.parse::<i16>() {
                Ok(y) if y >= -90 && y <= 90 => {
                    info!("using {} for {}", x, keyname);
                    Some(y)
                }
                Ok(_) | Err(_) => {
                    info!("invalid value {} for key {}", x, keyname);
                    None
                }
            },
            _ => {
                info!("key '{}' not defined", keyname);
                None
            }
        };
        self.imu_pitch = value;

        let keyname = "roll";
        let value = match sec_installation.get(keyname) {
            Some("") => {
                info!("no value for {} specified, ignoring", keyname);
                None
            }
            Some(x) => match x.parse::<i16>() {
                Ok(y) if y >= -180 && y <= 180 => {
                    info!("using {} for {}", x, keyname);
                    Some(y)
                }
                Ok(_) | Err(_) => {
                    info!("invalid value {} for key {}", x, keyname);
                    None
                }
            },
            _ => {
                info!("key '{}' not defined", keyname);
                None
            }
        };
        self.imu_roll = value;

        let keyname = "vrp2antenna";
        let value = match sec_installation.get(keyname) {
            Some("") => {
                info!("no value for {} specified, ignoring", keyname);
                None
            }
            Some(x) => {
                info!("using {} for {}", x, keyname);
                Xyz::from_str(&x)
            }
            _ => {
                info!("key '{}' not defined", keyname);
                None
            }
        };
        self.vrp2antenna = value;

        let keyname = "vrp2imu";
        let value = match sec_installation.get(keyname) {
            Some("") => {
                info!("no value for {} specified, ignoring", keyname);
                None
            }
            Some(x) => {
                info!("using {} for {}", x, keyname);
                Xyz::from_str(&x)
            }
            _ => {
                info!("key '{}' not defined", keyname);
                None
            }
        };
        self.vrp2imu = value;

        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Xyz {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Xyz {
    pub fn from_str(text: &str) -> Option<Self> {
        // TODO: Add limit checks

        if text.is_empty() {
            // TODO: Go with Result<Self, str> ? so we can return an error message
            // Or log error message here directly?
            return None;
        }

        let mut obj: Self = Default::default();
        let tokens: Vec<&str> = text.split(";").collect();
        if tokens.len() != 3 {
            return None;
        }

        // debug!("x is {}", tokens[0]);
        // debug!("y is {}", tokens[1]);
        // debug!("z is {}", tokens[2]);

        let x = Xyz::parse_float(tokens[0]); // TODO: check here and return with error message (.or_else, ...)
        let y = Xyz::parse_float(tokens[1]);
        let z = Xyz::parse_float(tokens[2]);
        if x.is_ok() && y.is_ok() && z.is_ok() {
            obj.x = x.unwrap();
            obj.y = y.unwrap();
            obj.z = z.unwrap();
            Some(obj)
        } else {
            None
        }
    }

    fn parse_float(text: &str) -> Result<f32, &'static str> {
        match text.trim().parse::<f32>() {
            Ok(res) => Ok(res),
            _ => Err("Conversion error"),
        }
    }
}

#[cfg(test)]
mod xyz_reader {
    use super::*;

    #[test]
    fn empty_string() {
        let uut = Xyz::from_str("");
        assert_eq!(uut.is_none(), true);
    }

    #[test]
    fn ok() {
        let uut = Xyz::from_str("1.0;-2.2;333.3");
        assert_eq!(uut.is_some(), true);
        let uut = uut.unwrap();
        assert_eq!(uut.x, 1.0); // TODO: Check float comparison
        assert_eq!(uut.y, -2.2);
        assert_eq!(uut.z, 333.3);

        let uut = Xyz::from_str("-11.1;22.2;-333.33");
        assert_eq!(uut.is_some(), true);
        let uut = uut.unwrap();
        assert_eq!(uut.x, -11.1);
        assert_eq!(uut.y, 22.2);
        assert_eq!(uut.z, -333.33);
    }

    #[test]
    fn ok_with_spaces() {
        let uut = Xyz::from_str("1.25;   -2.5; 3.75");
        assert_eq!(uut.is_some(), true);
        let uut = uut.unwrap();
        assert_eq!(uut.x, 1.25);
        assert_eq!(uut.y, -2.5);
        assert_eq!(uut.z, 3.75);
    }

    #[test]
    fn value_missing() {
        let uut = Xyz::from_str("1.0;;2.0");
        assert_eq!(uut.is_none(), true);
    }

    #[test]
    fn invalid_separators() {
        let uut = Xyz::from_str("1.0;2.0,3.0");
        assert_eq!(uut.is_none(), true);
    }

    /*
    #[test]
    fn out_of_range() {
        let uut = "1.0,222,3.0";
        assert_eq!(1, 0);
    }
    */
}

#[cfg(test)]
mod file_and_format {
    use super::*;

    #[test]
    fn file_not_found() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/does_not_exists.conf");
        assert_eq!(res.is_err(), true);
    }

    #[test]
    fn no_default_section() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_no_default_section.conf");
        assert_eq!(res.is_err(), true);
    }

    #[test]
    fn no_version_info() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_no_version.conf");
        assert_eq!(res.is_err(), true);
    }

    #[test]
    fn wrong_version_info() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_no_version.conf");
        assert_eq!(res.is_err(), true);
    }
}

#[cfg(test)]
mod update_rate {
    use super::*;

    #[test]
    fn key_missing() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_no_update_rate.conf");
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.update_rate.is_none(), true);
    }

    #[test]
    fn no_value() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_update_rate_empty.conf");
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.update_rate.is_none(), true);
    }

    #[test]
    fn value_ok() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_update_rate_ok.conf");
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.update_rate, Some(2));
    }

    #[test]
    fn value_too_high() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_update_rate_too_high.conf");
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.update_rate.is_none(), true);
    }

    #[test]
    fn syntax_error() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_update_rate_syntax_error.conf");
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.update_rate.is_none(), true);
    }
}

#[cfg(test)]
mod mode {
    use super::*;
    #[test]
    fn key_missing() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_no_mode.conf");
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.mode.is_none(), true);
    }

    #[test]
    fn no_value() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_mode_empty.conf");
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.mode.is_none(), true);
    }

    #[test]
    fn vehicle() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_mode_vehicle.conf");
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.mode, Some(String::from("vehicle")));
    }

    #[test]
    fn stationary() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_mode_stationary.conf");
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.mode, Some(String::from("stationary")));
    }

    #[test]
    fn unknown() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_mode_unknown.conf");
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.mode.is_none(), true);
    }
}

#[cfg(test)]
mod imu_angles {
    use super::*;

    #[test]
    fn key_missing() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_no_imu_yaw.conf");
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.imu_yaw.is_none(), true);
    }

    #[test]
    fn no_value() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_imu_yaw_empty.conf");
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.imu_yaw.is_none(), true);
    }

    #[test]
    fn yaw_ok() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_imu_yaw_ok.conf");
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.imu_yaw, Some(182));
        assert_eq!(config.imu_pitch, None);
        assert_eq!(config.imu_roll, None);
    }

    #[test]
    fn pitch_ok() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_imu_pitch_ok.conf");
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.imu_yaw, None);
        assert_eq!(config.imu_pitch, Some(-45));
        assert_eq!(config.imu_roll, None);
    }

    #[test]
    fn roll_ok() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_imu_roll_ok.conf");
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.imu_yaw, None);
        assert_eq!(config.imu_pitch, None);
        assert_eq!(config.imu_roll, Some(45));
    }
}

#[cfg(test)]
mod vrp_antenna {
    use super::*;

    #[test]
    fn ok() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_vrp_antenna_ok.conf");
        assert_eq!(res.is_ok(), true);

        let xyz = config.vrp2antenna.unwrap();
        assert_eq!(xyz.x, 1.0);
        assert_eq!(xyz.y, 1.5);
        assert_eq!(xyz.z, 0.3);
    }
}

#[cfg(test)]
mod sat_systems {
    use super::*;

    #[test]
    fn ok() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_systems_ok.conf");
        assert_eq!(res.is_ok(), true);

        let systems = config.systems.unwrap();
        assert!(systems.contains(&String::from("gps")));
        assert!(systems.contains(&String::from("galileo")));
        assert!(systems.contains(&String::from("beidou")));
        assert!(systems.contains(&String::from("sbas")));
        assert!(!systems.contains(&String::from("qzss")));
        assert!(!systems.contains(&String::from("glonass")));
    }
}
