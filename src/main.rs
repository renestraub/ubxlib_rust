// TODO: Is this defining of what files our exe is made?
mod checksum;
mod cid;
mod frame;
mod parser;
mod gnss_mgr;
mod server_tty;
mod ubx_cfg_rate;

use std::env;
use std::path::Path;

use crate::gnss_mgr::GnssMgr as GnssMgr;
use crate::gnss_mgr::GnssMgrConfig as GnssMgrConfig;

extern crate clap;
use clap::{crate_version, Arg, ArgMatches, App, SubCommand};

extern crate ini;
use ini::Ini;


fn main() {
    let app = App::new("gnss manager utility")
                // .version("0.1.0")
                .version(crate_version!())
                .about("Operates and configures u-blox NEO GNSS modems")
                .arg(Arg::with_name("verbose")
                    .short("v")
                    .conflicts_with("quiet")
                    .help("Be verbose, show debug output"))
                .arg(Arg::with_name("quiet")
                    .short("q")
                    .conflicts_with("verbose")
                    .help("Be quiet, only show warnings and errors"))

                    .arg(Arg::with_name("device")
                    .required(true)
                    .help("local serial device to which GNSS modem is connected (e.g. /dev/gnss0)"))

                .subcommand(SubCommand::with_name("config")
                    .about("Performs GNSS modem control function")
                    .arg(Arg::with_name("configfile")
                        .short("f")
                        .long("file")
                        .value_name("CONFIGFILE")
                        // .default_value("/etc/gnss/gnss0.conf")      // TODO: dynamic based on device name
                        .help("Path to configuration file")))

                .subcommand(SubCommand::with_name("control")
                    .about("Configures GNSS modem")
                    .arg(Arg::with_name("action")
                        .required(true)
                        .possible_values(&["cold-start", "persist", "factory-reset"])
                        .help("Selects action to perform")))
   
                .subcommand(SubCommand::with_name("sos")
                    .about("Save on shutdown operations")
                    .arg(Arg::with_name("action")
                        .required(true)
                        .possible_values(&["save", "clear"])
                        .help("Selects sos operation to perform")));
    
    let matches = app.get_matches();

    let rc = run_app(matches);
    std::process::exit(match rc {
        Ok(_) => 0,
        Err(err) => { eprintln!("error: {:?}", err); 1 }
    });
}


fn run_app(matches: ArgMatches) -> Result<(), String> {
    // println!("{:?}", matches);

    // Parse logger options -v/-q (mutually exclusive)
    if matches.is_present("verbose") {
        println!("verbose");
    }
    if matches.is_present("quiet") {
        println!("quiet");
    }

    // Devicename
    let device_name = matches.value_of("device").unwrap();  // unwrap must never fail here, as argument is required
    // println!("device_name {}", device_name);

    // Check that device exists (and is a TTY?)
    let device_exists = Path::new(device_name).exists();
    if !device_exists {
        return Err(format!("Device {} does not exist", device_name).to_string());
    }

    // TODO: create gnss-mgr here and provide to run_xx function instead of device_name
    // run gnss-mgr sos <action>
    let mut gnss = GnssMgr::new(device_name);

    // Check which subcommand was selected
    match matches.subcommand() {
        ("config", Some(m)) => run_config(m, &mut gnss),
        ("control", Some(m)) => run_control(m, &mut gnss),
        ("sos", Some(m)) => run_sos(m, &mut gnss),
        _ => Err("Unknown command".to_string()),
    }
}

fn run_config(matches: &ArgMatches, gnss: &mut GnssMgr) -> Result<(), String> {
    // println!("control {:?}", matches);

    // Check for optional config file name
    let configfile_path = matches.value_of("configfile");
    let configfile_path: String = match configfile_path {
        Some(path) => path.to_string(),                     // path to file specified
        _ => build_configfile_path(&gnss.device_name),      // left away, compute from device name
    };

    println!("configfile {}", configfile_path);

    // TODO: Have parse_config return config?
    let mut config: GnssMgrConfig = Default::default();
    let _res = parse_config(&configfile_path, &mut config)?;

    gnss.configure(&config);

    Ok(())
}

fn run_control(matches: &ArgMatches, gnss: &mut GnssMgr) -> Result<(), String> {
    println!("control {:?}", matches);

    let action = matches.value_of("action").unwrap();
    // println!("action {:?}", action);

    match action {
        "cold-start" => gnss.cold_start(),
        "factory-reset" => gnss.factory_reset(),
        "persist" => gnss.persist(),
        _ => return Err("Unknown command".to_string()),
    }

    Ok(())
}

fn run_sos(matches: &ArgMatches, gnss: &mut GnssMgr) -> Result<(), String> {
    println!("sos {:?}", matches);

    // Distinguish sos save, clear
    let action = matches.value_of("action").unwrap();
    // println!("action {:?}", action);

    match action {
        "save" => gnss.sos_save(),
        "clear" => gnss.sos_clear(),
        _ => return Err("Unknown command".to_string()),
    }

    Ok(())
}


fn build_configfile_path(path: &str) -> String {
    // Take devicename of form /dev/<name> to build /etc/gnss/<name>
    let path = &path.replace("/dev/", "/etc/gnss/");
    let mut path = String::from(path);
    path.push_str(".conf");
    println!("path {}", path);
    path
}

fn parse_config(path: &str, config: &mut GnssMgrConfig)  -> Result<(), String> {
    // Check if configfile exists
    let config_exists = Path::new(&path).exists();
    if !config_exists {
        return Err(format!("Configuration file {} not found", path).to_string());
    }

    // Import whole file, check for syntax errors
    // TODO: Proper error handling
    let conf = Ini::load_from_file(path).unwrap();

    let sec_general = conf.section(Some("default")).unwrap();
    let version = match sec_general.get("version") {
        Some("2") => 2,
        _ => return Err("Invalid configuration file format/version".to_string()),
    };
    println!("{:?}", version);

    // TODO: Combine in a nice getter with range check
    // Return Some(number) with valid content
    // or Err("....")
    let update_rate = match sec_general.get("update-rate") {
        Some("") => "3",    // TODO: Use 1 Hz if nothing specified
        Some(x) => x,
        _ => return Err("Invalid configuration file format/version".to_string()),
    };
    println!("{:?}", update_rate);
    let update_rate = match update_rate.parse::<i16>() {
        Ok(x) => x,
        _ => return Err("Invalid update-rate".to_string()),
    };
    println!("{:?}", update_rate);

    config.update_rate = Some(update_rate as u16);

    config.imu_yaw = Some(0.0);
    config.imu_pitch = Some(0.0);
    config.imu_yaw = Some(0.0);

    Ok(())
}
