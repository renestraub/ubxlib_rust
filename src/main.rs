mod config_file;
mod gnss_mgr;
mod neo_m8;
mod ubxlib;

use std::env;
use std::fs;
use std::os::unix::fs::FileTypeExt;
use std::path::Path;
use std::process::Command;

use env_logger::Builder;
use log::LevelFilter;

use clap::{crate_version, App, Arg, ArgMatches, SubCommand};

use crate::gnss_mgr::GnssMgr;

fn main() {
    let app = setup_arg_parse();
    let matches = app.get_matches();

    set_logger(&matches);

    let res = run_app(&matches);
    let ec = match res {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {}", err);
            1
        }
    };
    std::process::exit(ec);
}

fn run_app(matches: &ArgMatches) -> Result<(), String> {
    // unwrap must never fail here, as argument is checked by parser already
    let mut device_name: String = matches.value_of("device").unwrap().to_string();

    // complete devicename if it was given in short hand form, e.g. gnss instead of /dev/gnss0
    if !device_name.starts_with("/dev/") {
        device_name = format!("/dev/{}", device_name);
    }

    // Ensure port can be used (exists, not used by another process)
    check_port(&device_name)?;

    // Create GNSS Manager on specified device
    let mut gnss = GnssMgr::new(&device_name);

    // The "init" command checks the current bitrate and changes to 115200 if required.
    // all other subcommand use the modem at 115200.
    let bitrate = match matches.subcommand() {
        ("init", Some(_)) => None,
        _ => Some(115200 as u32),
    };

    gnss.prepare_port(bitrate)?;

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
    #[rustfmt::skip]
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
            .about("Initializes GNSS, sets bitrate"))

        .subcommand(SubCommand::with_name("config")
            .about("Configures GNSS modem")
            .arg(Arg::with_name("configfile")
                .short("f")
                .long("file")
                .value_name("CONFIGFILE")
                .help("Path to configuration file")))

        .subcommand(SubCommand::with_name("control")
            .about("Performs GNSS modem control function")
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

fn set_logger(matches: &ArgMatches) {
    // Parse logger options -v/-q (mutually exclusive)
    let mut builder = Builder::new();

    if matches.is_present("verbose") {
        builder.filter(None, LevelFilter::Debug).init();
    } else if matches.is_present("quiet") {
        builder.filter(None, LevelFilter::Warn).init();
    } else {
        builder.filter(None, LevelFilter::Info).init();
    }
}

fn check_port(device_name: &str) -> Result<(), String> {
    // Check that specified device exists
    if !Path::new(device_name).exists() {
        return Err(format!("device {} does not exist", device_name));
    }

    // Check that it's a character device (no block device)
    let meta = fs::metadata(device_name).map_err(|_| "cannot determine device type")?;
    let file_type = meta.file_type();
    if !file_type.is_char_device() {
        return Err(format!("device {} is not a character device", device_name));
    }

    // Ensure device is not in use
    let output = Command::new("fuser")
        .args(&[device_name])
        .output()
        .map_err(|e| format!("error executing fuser command ({:?})", e))?;
    if !output.stdout.is_empty() {
        let pid = String::from_utf8_lossy(&output.stdout);
        let pid = pid.trim();
        return Err(format!(
            "another process (PID:{}) is accessing the receiver",
            &pid
        ));
    }

    Ok(())
}
