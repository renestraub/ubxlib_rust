use crate::server_tty::ServerTty;

use crate::ubx_cfg_rate::{UbxCfgRate, UbxCfgRatePoll};
use crate::ubx_mon_ver::{UbxMonVer, UbxMonVerPoll};
use crate::ubx_cfg_nav5::{UbxCfgNav5, UbxCfgNav5Poll};
use crate::ubx_cfg_esfalg::{UbxCfgEsfAlg, UbxCfgEsfAlgPoll};


#[derive(Debug, Default)]
pub struct GnssMgrConfig {
    pub update_rate: Option<u16>,
    pub mode: Option<String>,
    // pub systems: (array of) Strings (or enums)  GPS;Galileo;Beidou;SBAS

    pub imu_yaw: Option<u16>,
    pub imu_pitch: Option<i16>,
    pub imu_roll: Option<i16>,

    pub vrp2antenna: Option<Xyz>,
    /*
    vrp2imu=
    */
}


#[derive(Debug, Default)]
pub struct Xyz {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}



// TODO: define information struct for version() method

// TODO: Implement !
// TODO: Return error code from each function

pub struct GnssMgr {
    pub device_name: String,

    server: ServerTty,
}

impl GnssMgr {
    pub fn new(device: &str) -> Self {
        Self { 
            device_name: String::from(device), 
            server: ServerTty::new(device),
        }
    }

    pub fn version(&mut self) {
        // TODO: Not sure what this function shall do
        // create /run/gnss/gnss0.config
        // let mut server = ServerTty::new(&self.device_name);

        let mut set = UbxMonVer::new();
        let poll = UbxMonVerPoll::new();

        self.server.poll(&poll, &mut set);
        println!("{:?}", set);
    }

    pub fn configure(&mut self, config: &GnssMgrConfig) {
        println!("configure");
        println!("device {}", self.device_name);
        println!("config {:?}", config);

        if config.update_rate.is_some() {
            let rate = config.update_rate.unwrap();
            // println!("applying update_rate {}", rate);
            self.set_update_rate(rate);
        }

        // TODO: Overly complicated with these string types...
        match &config.mode {
            Some(mode) => {
                match mode.as_str() {
                    "stationary" => self.set_dynamic_mode(2),
                    "vehicle" => self.set_dynamic_mode(4),
                    _ => (),
                }
            },
            _ => (),
        }

        // TODO: Combine IMU angles in a struct, this is ugly
        if config.imu_yaw.is_some() &&
            config.imu_pitch.is_some() &&
            config.imu_roll.is_some() {
            self.set_imu_angles(config.imu_yaw.unwrap(), config.imu_pitch.unwrap(), config.imu_roll.unwrap());
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
        let mut set = UbxCfgRate::new();
        let poll = UbxCfgRatePoll::new();

        self.server.poll(&poll, &mut set);
        println!("current settings {:?}", set);

        println!("changing to {}", 1000/rate);
        set.data.meas_rate = 1000u16 / rate;
        println!("new settings {:?}", set);

        self.server.set(&set);
    }

    fn set_dynamic_mode(&mut self, model: u8) {
        let mut set = UbxCfgNav5::new();
        let poll = UbxCfgNav5Poll::new();

        self.server.poll(&poll, &mut set);
        println!("current settings {:?}", set.data);

        set.data.dyn_model = model;
        println!("new settings {:?}", set.data);

        self.server.set(&set);
    }

    // TODO: provide arguments
    fn set_imu_angles(&mut self, yaw: u16, pitch: i16, roll: i16) {
        let mut set = UbxCfgEsfAlg::new();
        let poll = UbxCfgEsfAlgPoll::new();

        self.server.poll(&poll, &mut set);
        println!("current IMU settings {:?}", set.data);

        set.data.yaw = yaw as u32 * 100;
        set.data.pitch = pitch as i16 * 100;
        set.data.roll = roll as i16 * 100;
        println!("new IMU settings {:?}", set.data);

        self.server.set(&set);
    }
}
