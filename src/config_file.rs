use std::path::Path;
use ini::Ini;

use crate::gnss_mgr::GnssMgrConfig;


pub fn parse_config(path: &str, config: &mut GnssMgrConfig)  -> Result<(), String> {
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
    config.update_rate = value;

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
    config.mode = value;

    /*
    config.imu_yaw = Some(0.0);
    config.imu_pitch = Some(0.0);
    config.imu_yaw = Some(0.0);
    */
    Ok(())
}


#[cfg(test)]
mod file_and_format {
    use super::*;
   
    #[test]
    fn file_not_found() {
        let mut config: GnssMgrConfig = Default::default();
        let res = parse_config("test_files/does_not_exists.conf", &mut config);
        assert_eq!(res.is_err(), true);
    }

    #[test]
    fn no_default_section() {
        let mut config: GnssMgrConfig = Default::default();
        let res = parse_config("test_files/gnss0_no_default_section.conf", &mut config);
        println!("{:?}", res);
        assert_eq!(res.is_err(), true);
    }

    #[test]
    fn no_version_info() {
        let mut config: GnssMgrConfig = Default::default();
        let res = parse_config("test_files/gnss0_no_version.conf", &mut config);
        // println!("{:?}", res);
        assert_eq!(res.is_err(), true);
    }

    #[test]
    fn wrong_version_info() {
        let mut config: GnssMgrConfig = Default::default();
        let res = parse_config("test_files/gnss0_no_version.conf", &mut config);
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
        let res = parse_config("test_files/gnss0_no_update_rate.conf", &mut config);
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.update_rate.is_none(), true);
    }

    #[test]
    fn no_value() {
        let mut config: GnssMgrConfig = Default::default();
        let res = parse_config("test_files/gnss0_update_rate_empty.conf", &mut config);
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.update_rate.is_none(), true);
    }

    #[test]
    fn value_ok() {
        let mut config: GnssMgrConfig = Default::default();
        let res = parse_config("test_files/gnss0_update_rate_ok.conf", &mut config);
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.update_rate, Some(2));
    }

    #[test]
    fn value_too_high() {
        let mut config: GnssMgrConfig = Default::default();
        let res = parse_config("test_files/gnss0_update_rate_too_high.conf", &mut config);
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.update_rate.is_none(), true);
    }

    #[test]
    fn syntax_error() {
        let mut config: GnssMgrConfig = Default::default();
        let res = parse_config("test_files/gnss0_update_rate_syntax_error.conf", &mut config);
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
        let res = parse_config("test_files/gnss0_no_mode.conf", &mut config);
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.mode.is_none(), true);
    }

    #[test]
    fn no_value() {
        let mut config: GnssMgrConfig = Default::default();
        let res = parse_config("test_files/gnss0_mode_empty.conf", &mut config);
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.mode.is_none(), true);
    }

    #[test]
    fn vehicle() {
        let mut config: GnssMgrConfig = Default::default();
        let res = parse_config("test_files/gnss0_mode_vehicle.conf", &mut config);
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.mode, Some(String::from("vehicle")));
    }

    #[test]
    fn stationary() {
        let mut config: GnssMgrConfig = Default::default();
        let res = parse_config("test_files/gnss0_mode_stationary.conf", &mut config);
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.mode, Some(String::from("stationary")));
    }

    #[test]
    fn unknown() {
        let mut config: GnssMgrConfig = Default::default();
        let res = parse_config("test_files/gnss0_mode_unknown.conf", &mut config);
        println!("{:?}", res);
        assert_eq!(res.is_ok(), true);
        assert_eq!(config.mode.is_none(), true);
    }
}
