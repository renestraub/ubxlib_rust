// TODO: Is this defining of what files our exe is made?
mod checksum;
mod cid;
mod frame;
mod parser;
mod ubx_cfg_rate;

use crate::cid::UbxCID as UbxCID;

use crate::ubx_cfg_rate::UbxCfgRate as UbxCfgRate;
use crate::ubx_cfg_rate::UbxCfgRatePoll as UbxCfgRatePoll;

// use crate::frame::UbxFrame as UbxFrame;
use crate::frame::UbxFrameInfo as UbxFrameInfo;
use crate::frame::UbxFrameSerialize as UbxFrameSerialize;

use crate::parser::Parser as Parser;
use crate::parser::Packet as Packet;


extern crate clap;
extern crate ini;

use std::time::Instant;
use std::path::Path;
use clap::{crate_version, Arg, ArgMatches, App, SubCommand};
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
        Some("") => "1",    // TODO: Use 1 Hz if nothing specified
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


#[derive(Default)]
#[derive(Debug)]
struct GnssMgrConfig {
    update_rate: Option<u16>,
    imu_yaw: Option<f32>,
    imu_pitch: Option<f32>,
    imu_roll: Option<f32>,
    // TODO: Add missing items
    /*
    mode=stationary, vehicle
    systems=GPS;Galileo;Beidou;SBAS
    vrp2antenna=
    vrp2imu=
    */
}


// TODO: Implement !
// TODO: Return error code from each function

struct GnssMgr {
    pub device_name: String,
}

impl GnssMgr {
    pub fn new(device: &str) -> Self {
        Self { device_name: String::from(device), }
    }

    pub fn configure(&mut self, config: &GnssMgrConfig) {
        println!("configure");
        println!("device {}", self.device_name);
        println!("config {:?}", config);

        if config.update_rate.is_some() {
            let rate = config.update_rate.unwrap();
            println!("applying update_rate {}", rate);
            self.set_update_rate(rate);
        }
    }

    pub fn sos_save(&mut self) {
        println!("sos save");
        println!("device {}", self.device_name);
    }    

    pub fn sos_clear(&mut self) {
        println!("sos clear");
        println!("device {}", self.device_name);
    }    

    pub fn cold_start(&mut self) {
        println!("cold-start");
        println!("device {}", self.device_name);
    }    

    pub fn factory_reset(&mut self) {
        println!("factory-eset");
        println!("device {}", self.device_name);
    }    

    pub fn persist(&mut self) {
        println!("persist");
        println!("device {}", self.device_name);
    }
    
    
    fn set_update_rate(&mut self, rate: u16) {
        let poll = UbxCfgRatePoll::new();
        // let name = poll.name();
        frame_info(&poll);
    
        let mut set = UbxCfgRate::new();
        set.meas_rate = 1000u16 / rate;
        set.nav_rate = 1;
        set.time_ref = 0;
        frame_info(&set);
    
        let mut server = ServerTty::new();
        server.set(&set);
    }
}


fn frame_info<F>(f: &F)
where F: UbxFrameInfo {
    println!("UbxFrameInfo name is: {}", f.name());
    println!("cls is: {}", f.cls());
    println!("id is: {}", f.id());
}




struct ServerTty {
    // serial port
    parser: Parser,
}

impl ServerTty {
    pub fn new() -> Self {
        Self { 
            parser: Parser::new(),
        }
    }    

    // NOTE: Prototype for server_tty send() method
    fn set<F>(&mut self, f: &F)
    where F: UbxFrameSerialize {
        println!("set");

        // message.pack();
        let data = f.to_bin();
        // println!("{:?}", data);

        self.parser.add_filter(UbxCID::new(0x05, 0x01));    // ACK-ACK
        self.parser.add_filter(UbxCID::new(0x05, 0x00));    // ACK-NAK
        // self.parser.add_filter(UbxCID::new(0x13, 0x40));    // TODO: Remove Test frame

        // self.send();
        self.wait();

        self.parser.clear_filter();
    }

    fn wait(&mut self) {
        const FRAME_1: [u8; 32] = [0xB5, 0x62, 0x13, 0x40, 0x18, 0x00, 0x10, 0x00, 0x00, 0x12, 0xE4, 0x07, 0x09, 0x05, 
        0x06, 0x28, 0x30, 0x00, 0x40, 0x28, 0xEF, 0x0C, 0x0A, 0x00, 0x00, 0x00, 
        0x00, 0x00, 0x00, 0x00, 0x51, 0xAC];

        self.parser.process(&FRAME_1.to_vec());

        let start = Instant::now();
        let mut elapsed = start.elapsed();
        while elapsed.as_millis() < 3000 {

            let res = self.parser.packet();
            match res {
                Some(p) => { println!("got packet {:?}", p); break; }  // TODO: construct frame from packet
                _ => ()
            }

            elapsed = start.elapsed();
            // println!("{:?}", elapsed.as_millis());
        }
    }
}

/*
extern crate serial;

use std::env;
use std::io;
// use std::time::Duration;

use std::io::prelude::*;
use serial::prelude::*;

const SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate:    serial::Baud9600,
    char_size:    serial::Bits8,
    parity:       serial::ParityNone,
    stop_bits:    serial::Stop1,
    flow_control: serial::FlowNone,
};


fn main() {
    loop {
        match test() {
            Ok(_) =>  (), //println!("all good"),
            Err(e) => println!("errored {:?}", e),
        }
    }
}


fn test() -> Result<(), io::Error> {
    let mut port = serial::open("/dev/gnss0")?;

    port.configure(&SETTINGS)?;
    // port.set_timeout(Duration::from_secs(1000))?;

    // let mut buf: Vec<u8> = Vec::with_capacity(256);
    let mut read_buffer = [0u8; 1024];
    // let mut read_buffer = Vec::new();
    // println!("buf {:?}", read_buffer);

    let count = port.read(&mut read_buffer[..]);
    match count {
        Ok(n) => { 
            println!("{} bytes read", n); 
            let data = read_buffer[0..n].to_vec();
            println!("{:?}", data)
        },
        Err(_) => println!("no data"),
    }

    Ok(())
}

*/
