// TODO: Is this defining of what files our exe is made?
mod checksum;
mod cid;
mod frame;
mod parser;
mod ubx_cfg_rate;

// use crate::frame::UbxFrame as UbxFrame;
// use crate::cid::UbxCID as UbxCID;

use crate::ubx_cfg_rate::UbxCfgRate as UbxCfgRate;
use crate::ubx_cfg_rate::UbxCfgRatePoll as UbxCfgRatePoll;

use crate::frame::UbxFrameInfo as UbxFrameInfo;
use crate::frame::UbxFrameSerialize as UbxFrameSerialize;


fn main() {
    let mut p = parser::Parser::new();
    p.process(&[1, 2, 3].to_vec());
    let _pkt = p.packet();

    let f1 = UbxCfgRatePoll::new();
    let name = f1.name();
    println!("UbxCfgRatePoll Name {}", name);

    let mut f2 = UbxCfgRate::new();
    let name = f2.name();
    f2.meas_rate = 500;
    f2.nav_rate = 1;
    f2.time_ref = 0;
    println!("UbxCfgRate Name {}", name);

    frame_info(&f1);
    frame_info(&f2);

    frame_data(&f1);
    frame_data(&f2);
}


fn frame_info<F>(f: &F)
where F: UbxFrameInfo {
    println!("UbxFrameInfo name is: {}", f.name());
    println!("cls is: {}", f.cls());
    println!("id is: {}", f.id());
}


// NOTE: Prototype for server_tty send() method
fn frame_data<F>(f: &F)
where F: UbxFrameSerialize {
    let data = f.to_bin();
    println!("{:?}", data);
}
