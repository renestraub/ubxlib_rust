use crate::server_tty::ServerTty as ServerTty;

use crate::ubx_cfg_rate::UbxCfgRate as UbxCfgRate;
use crate::ubx_cfg_rate::UbxCfgRatePoll as UbxCfgRatePoll;

use crate::ubx_mon_ver::UbxMonVer as UbxMonVer;
use crate::ubx_mon_ver::UbxMonVerPoll as UbxMonVerPoll;



#[derive(Default)]
#[derive(Debug)]
pub struct GnssMgrConfig {
    pub update_rate: Option<u16>,
    pub imu_yaw: Option<f32>,
    pub imu_pitch: Option<f32>,
    pub imu_roll: Option<f32>,
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

pub struct GnssMgr {
    pub device_name: String,
}

impl GnssMgr {
    pub fn new(device: &str) -> Self {
        Self { device_name: String::from(device), }
    }

    pub fn version(&mut self) {
        // TODO: Not sure what this function shall do
        // create /run/gnss/gnss0.config
        let mut server = ServerTty::new(&self.device_name);

        let mut set = UbxMonVer::new();
        let poll = UbxMonVerPoll::new();

        server.poll(&poll, &mut set);
        println!("current settings {:?}", set);
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
        // TODO: Move to some open() function or even ctor
        // TODO: DOn't want to open/close for each request
        let mut server = ServerTty::new(&self.device_name);

        let mut set = UbxCfgRate::new();
        let poll = UbxCfgRatePoll::new();

        server.poll(&poll, &mut set);
        println!("current settings {:?}", set);

        println!("changing to {}", 1000/rate);
        set.meas_rate = 1000u16 / rate;
        println!("new settings {:?}", set);

        server.set(&set);
    }
}
