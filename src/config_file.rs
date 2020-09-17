use std::path::Path;
use ini::Ini;


#[derive(Debug, Default)]
pub struct GnssMgrConfig {
    pub update_rate: Option<u16>,
    pub mode: Option<String>,
    // pub systems: (array of) Strings (or enums)  GPS;Galileo;Beidou;SBAS

    pub imu_yaw: Option<u16>,
    pub imu_pitch: Option<i16>,
    pub imu_roll: Option<i16>,

    pub vrp2antenna: Option<Xyz>,
    pub vrp2imu: Option<Xyz>,
}


#[derive(Debug, Default)]
#[derive(Clone, Copy)]
pub struct Xyz {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}


impl GnssMgrConfig {
    pub fn parse_config(&mut self, path: &str)  -> Result<(), String> {
        // Check if configfile exists
        let config_exists = Path::new(&path).exists();
        if !config_exists {
            return Err(format!("Configuration file {} not found", path).to_string());
        }

        // Import whole file, check for syntax errors
        // TODO: Proper error handling
        let conf = Ini::load_from_file(path).unwrap();

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
            Some("") => { println!("no value for {} specified, ignoring", keyname); None },
            Some(x) => {
                match x.parse::<u16>() {
                    Ok(y) if y <= 2 => { println!("using {} for {}", x, keyname); Some(y) },
                    Ok(_) | Err(_) => { println!("invalid value {} for key {}", x, keyname); None },
                }
            },
            _ => { println!("key '{}' not defined", keyname); None },
        };
        self.update_rate = value;

        let sec_navigation = match conf.section(Some("navigation")) {
            Some(sec) => sec,
            _ => return Err("Invalid configuration file format/version".to_string()),
        };

        let keyname = "mode";
        let valid_args = vec!["stationary", "vehicle"]; 
        let value = match sec_navigation.get(keyname) {
            Some("") => { println!("no value for {} specified, ignoring", keyname); None },
            Some(x) if valid_args.contains(&x) => { println!("using {} for {}", x, keyname); Some(String::from(x)) },
            _ => { println!("key '{}' not defined", keyname); None },
        };
        self.mode = value;



        let sec_installation = match conf.section(Some("installation")) {
            Some(sec) => sec,
            _ => return Err("Invalid configuration file format/version".to_string()),
        };

        let keyname = "yaw";
        let value = match sec_installation.get(keyname) {
            Some("") => { println!("no value for {} specified, ignoring", keyname); None },
            Some(x) => {
                match x.parse::<u16>() {
                    Ok(y) if y <= 360 => { println!("using {} for {}", x, keyname); Some(y) },
                    Ok(_) | Err(_) => { println!("invalid value {} for key {}", x, keyname); None },
                }
            },
            _ => { println!("key '{}' not defined", keyname); None },
        };
        self.imu_yaw = value;

        let keyname = "pitch";
        let value = match sec_installation.get(keyname) {
            Some("") => { println!("no value for {} specified, ignoring", keyname); None },
            Some(x) => {
                match x.parse::<i16>() {
                    Ok(y) if y >= -90 && y <= 90 => { println!("using {} for {}", x, keyname); Some(y) },
                    Ok(_) | Err(_) => { println!("invalid value {} for key {}", x, keyname); None },
                }
            },
            _ => { println!("key '{}' not defined", keyname); None },
        };
        self.imu_pitch = value;

        let keyname = "roll";
        let value = match sec_installation.get(keyname) {
            Some("") => { println!("no value for {} specified, ignoring", keyname); None },
            Some(x) => {
                match x.parse::<i16>() {
                    Ok(y) if y >= -180 && y <= 180 => { println!("using {} for {}", x, keyname); Some(y) },
                    Ok(_) | Err(_) => { println!("invalid value {} for key {}", x, keyname); None },
                }
            },
            _ => { println!("key '{}' not defined", keyname); None },
        };
        self.imu_roll = value;



        let keyname = "vrp2antenna";
        let value = match sec_installation.get(keyname) {
            Some("") => { println!("no value for {} specified, ignoring", keyname); None },
            Some(x) => { println!("using {} for {}", x, keyname); Xyz::from_str(&x) },
            _ => { println!("key '{}' not defined", keyname); None },
        };
        self.vrp2antenna = value;

        let keyname = "vrp2imu";
        let value = match sec_installation.get(keyname) {
            Some("") => { println!("no value for {} specified, ignoring", keyname); None },
            Some(x) => { println!("using {} for {}", x, keyname); Xyz::from_str(&x) },
            _ => { println!("key '{}' not defined", keyname); None },
        };
        self.vrp2imu = value;

        Ok(())
    }
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
        let tokens:Vec<&str>= text.split(";").collect();
        if tokens.len() != 3 {
            return None;
        }

        println!("x is {}",tokens[0]);
        println!("y is {}",tokens[1]);
        println!("z is {}",tokens[2]);

        let x = Xyz::parse_float(tokens[0]);    // TODO: check here and return with error message (.or_else, ...)
        let y = Xyz::parse_float(tokens[1]);
        let z = Xyz::parse_float(tokens[2]);
        if x.is_ok() && y.is_ok() && z.is_ok() {
            obj.x = x.unwrap();
            obj.y = y.unwrap();
            obj.z = z.unwrap();
            Some(obj)
        }
        else {
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
        println!("{:?}", uut);
        assert_eq!(uut.is_none(), true);
    }

    #[test]
    fn ok() {
        let uut = Xyz::from_str("1.0;-2.2;333.3");
        println!("{:?}", uut);
        assert_eq!(uut.is_some(), true);
        let uut = uut.unwrap();
        assert_eq!(uut.x, 1.0); // TODO: Check float comparison
        assert_eq!(uut.y, -2.2);
        assert_eq!(uut.z, 333.3);

        let uut = Xyz::from_str("-11.1;22.2;-333.33");
        println!("{:?}", uut);
        assert_eq!(uut.is_some(), true);
        let uut = uut.unwrap();
        assert_eq!(uut.x, -11.1);
        assert_eq!(uut.y, 22.2);
        assert_eq!(uut.z, -333.33);
    }

    #[test]
    fn ok_with_spaces() {
        let uut = Xyz::from_str("1.25;   -2.5; 3.75");
        println!("{:?}", uut);
        assert_eq!(uut.is_some(), true);
        let uut = uut.unwrap();
        assert_eq!(uut.x, 1.25);
        assert_eq!(uut.y, -2.5);
        assert_eq!(uut.z, 3.75);
    }

    #[test]
    fn value_missing() {
        let uut = Xyz::from_str("1.0;;2.0");
        println!("{:?}", uut);
        assert_eq!(uut.is_none(), true);
    }

    #[test]
    fn invalid_separators() {
        let uut = Xyz::from_str("1.0;2.0,3.0");
        println!("{:?}", uut);
        assert_eq!(uut.is_none(), true);
    }

    /*
    #[test]
    fn out_of_range() {
        let uut = "1.0,222,3.0";
        println!("{:?}", uut);
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
        println!("{:?}", res);
        assert_eq!(res.is_err(), true);
    }

    #[test]
    fn no_version_info() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_no_version.conf");
        // println!("{:?}", res);
        assert_eq!(res.is_err(), true);
    }

    #[test]
    fn wrong_version_info() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_no_version.conf");
        // println!("{:?}", res);
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
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.update_rate.is_none(), true);
    }

    #[test]
    fn no_value() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_update_rate_empty.conf");
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.update_rate.is_none(), true);
    }

    #[test]
    fn value_ok() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_update_rate_ok.conf");
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.update_rate, Some(2));
    }

    #[test]
    fn value_too_high() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_update_rate_too_high.conf");
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.update_rate.is_none(), true);
    }

    #[test]
    fn syntax_error() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_update_rate_syntax_error.conf");
        println!("{:?}", res);
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
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.mode.is_none(), true);
    }

    #[test]
    fn no_value() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_mode_empty.conf");
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.mode.is_none(), true);
    }

    #[test]
    fn vehicle() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_mode_vehicle.conf");
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.mode, Some(String::from("vehicle")));
    }

    #[test]
    fn stationary() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_mode_stationary.conf");
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.mode, Some(String::from("stationary")));
    }

    #[test]
    fn unknown() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_mode_unknown.conf");
        println!("{:?}", res);
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
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.imu_yaw.is_none(), true);
    }

    #[test]
    fn no_value() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_imu_yaw_empty.conf");
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.imu_yaw.is_none(), true);
    }

    #[test]
    fn yaw_ok() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_imu_yaw_ok.conf");
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.imu_yaw, Some(182));
        assert_eq!(config.imu_pitch, None);
        assert_eq!(config.imu_roll, None);
    }

    #[test]
    fn pitch_ok() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_imu_pitch_ok.conf");
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.imu_yaw, None);
        assert_eq!(config.imu_pitch, Some(-45));
        assert_eq!(config.imu_roll, None);
    }

    #[test]
    fn roll_ok() {
        let mut config: GnssMgrConfig = Default::default();
        let res = config.parse_config("test_files/gnss0_imu_roll_ok.conf");
        println!("{:?}", res);
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
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);

        let xyz = config.vrp2antenna.unwrap();
        assert_eq!(xyz.x, 1.0);
        assert_eq!(xyz.y, 1.5);
        assert_eq!(xyz.z, 0.3);
    }
}
