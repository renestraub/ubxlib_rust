mod config_file;
mod checksum;
mod cid;
mod frame;
mod parser;
mod neo_m8;
mod gnss_mgr;
mod server_tty;
mod ubx_cfg_esfla;
mod ubx_cfg_esfalg;
mod ubx_cfg_nav5;
mod ubx_cfg_rate;
mod ubx_cfg_nmea;
mod ubx_cfg_rst;
mod ubx_cfg_cfg;
mod ubx_cfg_prt;
mod ubx_mon_ver;
mod ubx_upd_sos;
mod ubx_mga_init_time_utc;

use std::fs;
use std::os::unix::fs::FileTypeExt;
use std::env;
use std::path::Path;
use std::process::Command;

use clap::{crate_version, Arg, ArgMatches, App, SubCommand};

use crate::gnss_mgr::GnssMgr;


fn main() {
    env_logger::init();

    let app = setup_arg_parse();
    let matches = app.get_matches();

    let rc = run_app(matches);

    std::process::exit(match rc {
        Ok(_) => 0,
        Err(err) => { eprintln!("error: {}", err); 1 }
    });
}

fn run_app(matches: ArgMatches) -> Result<(), String> {
    // Parse logger options -v/-q (mutually exclusive)
    if matches.is_present("verbose") {
        println!("verbose");
    }
    if matches.is_present("quiet") {
        println!("quiet");
    }
    // TODO: See how we can map this to env_logger
    // Enable logger only below this check and modify ENV before?

    // Devicename
    // unwrap must never fail here, as argument is checked by parser already
    let device_name = matches.value_of("device").unwrap();  

    // Check that specified device exists
    let device_exists = Path::new(device_name).exists();
    if !device_exists {
        return Err(format!("Device {} does not exist", device_name).to_string());
    }

    // Check that it's a character device (or i.e. no block device)
    let meta = fs::metadata(device_name);
    match meta {
        Ok(m) => {
            let file_type = m.file_type();
            if !file_type.is_char_device() {
                return Err(format!("Device {} is not a character device", device_name).to_string());
            }
        },
        Err(_) => return Err(String::from("cannot determine device type")),
    }

    // Check if device is in use, if so abort
    let output = Command::new("fuser").args(&[device_name]).output();
    match output {
        Ok(o) => { 
            if o.stdout.len() > 0 {
                let pid = String::from_utf8_lossy(&o.stdout);
                let pid = pid.trim();
                return Err(format!("another process (PID:{}) is accessing the receiver", &pid).to_string());
            }
        },
        Err(e) => return Err(format!("error executing fuser command {:?}", e).to_string()),
    }

    let mut gnss = GnssMgr::new(device_name);

    // The "init" command checks the current bitrate and changes to 115200 if required.
    let bitrate = match matches.subcommand() {
        ("init", Some(_)) => {
            match gnss.prepare_port() {
                Ok(br) => br,
                Err(e) => return Err(format!("{}", e).to_string()),
            }
        },
        _ => 115200,
    };

    match gnss.open(bitrate) {
        Ok(x) => x,
        Err(e) => return Err(format!("error opening serial port {:?}", e).to_string()),
    }

    // Execute desired command, returns Result directly
    match matches.subcommand() {
        ("init", Some(m)) => gnss.run_init(m),
        ("config", Some(m)) => gnss.run_config(m),
        ("control", Some(m)) => gnss.run_control(m),
        ("sos", Some(m)) => gnss.run_sos(m),
        _ => Err("Unknown command".to_string()),
    }
}

fn setup_arg_parse() -> App<'static, 'static> {
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
    app
}
