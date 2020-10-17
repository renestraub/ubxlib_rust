use ini::{ini::Properties, Ini};
use log::info;
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Default)]
pub struct GnssMgrConfig {
    pub update_rate: Option<i32>,
    pub mode: Option<String>,
    pub systems: Option<Vec<String>>,
    pub imu_angles: Option<Angles>,
    pub vrp2antenna: Option<Xyz>,
    pub vrp2imu: Option<Xyz>,
}

impl GnssMgrConfig {
    // TODO: Better API would be with config file content as string
    // TODO: Makes it easier to test
    pub fn parse_config<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        // Import whole file, check for syntax errors
        let conf = Ini::load_from_file(path).map_err(|_err| "configuration file not found")?;

        Self::have_duplicates(&conf)?;

        // Get sections
        let sec_general = conf
            .section(Some("default"))
            .ok_or("Invalid configuration file format/version")?;

        let sec_navigation = conf
            .section(Some("navigation"))
            .ok_or("Invalid configuration file format/version")?;

        let sec_installation = conf
            .section(Some("installation"))
            .ok_or("Invalid configuration file format/version")?;

        // Check for version 2 format
        let _version = match sec_general.get("version") {
            Some("2") => 2,
            _ => return Err("Invalid configuration file format/version".to_string()),
        };

        // Update rate
        self.update_rate = Self::get_int(sec_general, "update-rate", |val| val >= 1 && val <= 2);

        // GNSS operation mode
        let valid_modes = vec!["stationary", "vehicle"];
        self.mode = Self::get_string(sec_navigation, "mode", |val| valid_modes.contains(&val));

        // Satellite systems
        let value_str = Self::get_string(sec_navigation, "systems", |_| true);
        self.systems = match value_str {
            Some(x) => Some(x.split(";").map(|s| s.to_string().to_lowercase()).collect()),
            _ => None,
        };

        // IMU Angles
        let imu_yaw = Self::get_int(sec_installation, "yaw", |val| val >= 0 && val <= 360);
        let imu_pitch = Self::get_int(sec_installation, "pitch", |val| val >= -90 && val <= 90);
        let imu_roll = Self::get_int(sec_installation, "roll", |val| val >= -180 && val <= 180);
        if imu_yaw.is_some() && imu_pitch.is_some() && imu_roll.is_some() {
            self.imu_angles = Angles::new(
                imu_yaw.unwrap() as u32,
                imu_pitch.unwrap() as i16,
                imu_roll.unwrap() as i16,
            );
        }

        // Lever Arms
        let value_str = Self::get_string(sec_installation, "vrp2antenna", |x| {
            Xyz::from_str(&x).is_some()
        });
        self.vrp2antenna = match value_str {
            Some(x) => Xyz::from_str(&x),
            _ => None,
        };

        let value_str =
            Self::get_string(sec_installation, "vrp2imu", |x| Xyz::from_str(&x).is_some());
        self.vrp2imu = match value_str {
            Some(x) => Xyz::from_str(&x),
            _ => None,
        };

        Ok(())
    }

    fn have_duplicates(conf: &Ini) -> Result<(), String> {
        let general_section_name = "general";
        let mut keys = HashSet::<String>::new();

        for (sec, prop) in conf.iter() {
            let section_name = sec.as_ref().unwrap_or(&general_section_name);
            for (key, _v) in prop.iter() {
                let fullname = format!("[{}] {}", section_name, key);
                if keys.contains(&fullname) {
                    return Err(format!("duplicate key {} detected", fullname));
                } else {
                    keys.insert(fullname);
                }
            }
        }
        
        Ok(())
    }

    fn get_int<F>(section: &Properties, keyname: &str, fn_check: F) -> Option<i32>
    where
        F: FnOnce(i32) -> bool,
    {
        let value = match section.get(keyname) {
            Some("") => {
                info!("no value for {} specified, ignoring", keyname);
                None
            }
            Some(val_str) => match val_str.parse::<i32>() {
                Ok(value) if fn_check(value) => {
                    // info!("using {} for {}", val_str, keyname);
                    info!("{}: {}", keyname, val_str);
                    Some(value)
                }
                Ok(_) | Err(_) => {
                    info!("invalid value {} for key {}", val_str, keyname);
                    None
                }
            },
            _ => {
                info!("key '{}' not defined", keyname);
                None
            }
        };
        value
    }

    fn get_string<F>(section: &Properties, keyname: &str, fn_check: F) -> Option<String>
    where
        F: FnOnce(&str) -> bool,
    {
        let value = match section.get(keyname) {
            Some("") => {
                info!("no value for {} specified, ignoring", keyname);
                None
            }
            Some(value) if fn_check(&value) => {
                info!("{}: {}", keyname, value);
                Some(String::from(value))
            }
            Some(value) => {
                info!("invalid value {} for key {}", value, keyname);
                None
            }
            _ => {
                info!("key '{}' not defined", keyname);
                None
            }
        };
        value
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Angles {
    pub yaw: u32,
    pub pitch: i16,
    pub roll: i16,
}

impl Angles {
    pub fn new(yaw: u32, pitch: i16, roll: i16) -> Option<Self> {
        Some(Self { yaw, pitch, roll })
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
        if text.is_empty() {
            return None;
        }

        let mut obj: Self = Default::default();
        let tokens: Vec<&str> = text.split(";").collect();
        if tokens.len() != 3 {
            return None;
        }

        let x = Xyz::parse_float(tokens[0]);
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
        assert!(float_same(uut.x, 1.0));
        assert!(float_same(uut.y, -2.2));
        assert!(float_same(uut.z, 333.3));

        let uut = Xyz::from_str("-11.1;22.2;-333.33");
        assert_eq!(uut.is_some(), true);
        let uut = uut.unwrap();
        assert!(float_same(uut.x, -11.1));
        assert!(float_same(uut.y, 22.2));
        assert!(float_same(uut.z, -333.33));
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

    fn float_same(a: f32, b: f32) -> bool {
        let delta = (a - b).abs();
        return delta < 0.01;
    }
}

#[cfg(test)]
mod file_and_format {
    use super::*;

    #[test]
    fn file_not_found() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/does_not_exists.conf");
        assert!(res.is_err());
    }

    #[test]
    fn no_default_section() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_no_default_section.conf");
        assert!(res.is_err());
    }

    #[test]
    fn no_version_info() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_no_version.conf");
        assert!(res.is_err());
    }

    #[test]
    fn wrong_version_info() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_no_version.conf");
        assert!(res.is_err());
    }

    #[test]
    fn duplicate() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_duplicate.conf");
        assert!(res.is_err());
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
        assert!(config.imu_angles.is_none());
    }

    #[test]
    fn no_value() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_imu_yaw_empty.conf");
        assert_eq!(res.is_ok(), true);
        assert!(config.imu_angles.is_none());
    }

    #[test]
    fn valid() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_imu_valid.conf");
        assert_eq!(res.is_ok(), true);
        assert!(config.imu_angles.is_some());

        let angles = config.imu_angles.unwrap();
        assert_eq!(angles.yaw, 180);
        assert_eq!(angles.pitch, -90);
        assert_eq!(angles.roll, 90);
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
        assert!(float_same(xyz.x, 1.0));
        assert!(float_same(xyz.y, 1.5));
        assert!(float_same(xyz.z, 0.3));
    }

    fn float_same(a: f32, b: f32) -> bool {
        let delta = (a - b).abs();
        return delta < 0.01;
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
