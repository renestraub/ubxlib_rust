// TODO: Is this defining of what files our exe is made?
mod config_file;
mod checksum;
mod cid;
mod frame;
mod parser;
mod gnss_mgr;
mod server_tty;
mod ubx_cfg_esfalg;
mod ubx_cfg_nav5;
mod ubx_cfg_rate;
mod ubx_cfg_nmea;
mod ubx_cfg_rst;
mod ubx_mon_ver;

use std::env;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use std::{thread, time};
use std::collections::HashMap;

use crate::gnss_mgr::GnssMgr;
use crate::config_file::GnssMgrConfig;

use clap::{crate_version, Arg, ArgMatches, App, SubCommand};


static CURRENT_FW_VER: &str = "ADR 4.31";


fn main() {
    let app = App::new("gnss manager utility")
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

                .subcommand(SubCommand::with_name("init")
                    .about("Initializes GNSS"))

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

    // Check that device exists
    // TODO: required, as GnssMgr::new will test it as well
    let device_exists = Path::new(device_name).exists();
    if !device_exists {
        return Err(format!("Device {} does not exist", device_name).to_string());
    }

    // TODO: Check return code
    let mut gnss = GnssMgr::new(device_name);

    for l in 1..2 {
        println!("*** {} **************************************************", l);
        thread::sleep(time::Duration::from_millis(250));

        // Check which subcommand was selected
        match matches.subcommand() {
            ("init", Some(m)) => run_init(m, &mut gnss),
            ("config", Some(m)) => run_config(m, &mut gnss),
            ("control", Some(m)) => run_control(m, &mut gnss),
            ("sos", Some(m)) => run_sos(m, &mut gnss),
            _ => Err("Unknown command".to_string()),
        }.unwrap();
    }
/*
    // Check which subcommand was selected
    match matches.subcommand() {
        ("init", Some(m)) => run_init(m, &mut gnss),
        ("config", Some(m)) => run_config(m, &mut gnss),
        ("control", Some(m)) => run_control(m, &mut gnss),
        ("sos", Some(m)) => run_sos(m, &mut gnss),
        _ => Err("Unknown command".to_string()),
    }
*/
    Ok(())
}


fn run_init(_matches: &ArgMatches, gnss: &mut GnssMgr) -> Result<(), String> {
    // TODO:
    // Check bitrate, change if required





    // create /run/gnss/gnss0.config
    let runfile_path = build_runfile_path(&gnss.device_name);

    // vendor is always "ublox" when using this library
    let mut info: HashMap<&str, String> = HashMap::new();
    info.insert("vendor", String::from("ublox"));

    gnss.version(&mut info);
    // println!("{:?}", info);

    match write_runfile(&runfile_path, &info) {
        Ok(_) => println!("GNSS run file created"),
        Err(_) => { println!("Error creating run file"); }, // TODO: return code on error
    }

    // Change protocol to NMEA 4.1
    gnss.set_nmea_protocol_version(0x41);

    Ok(())
}


fn run_config(matches: &ArgMatches, gnss: &mut GnssMgr) -> Result<(), String> {
    // Check for optional config file name
    let configfile_path = matches.value_of("configfile");
    let configfile_path: String = match configfile_path {
        Some(path) => path.to_string(),                     // path to file specified
        _ => build_configfile_path(&gnss.device_name),      // left away, compute from device name
    };

    // println!("configfile {}", configfile_path);

    // Get configuration from config file
    let mut config: GnssMgrConfig = Default::default();
    let _res = config.parse_config(&configfile_path)?;

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

// TODO: return Path instead of String
fn build_configfile_path(path: &str) -> String {
    // Take devicename of form /dev/<name> to build /etc/gnss/<name>
    let path = &path.replace("/dev/", "/etc/gnss/");
    let mut path = String::from(path);
    path.push_str(".conf");
    path
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
