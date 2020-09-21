use std::io::prelude::*;
use std::time::Duration;
use std::time::Instant;

use log::{debug, info, warn};
use serial::prelude::*;

use crate::cid::UbxCID;
use crate::frame::{UbxFrameDeSerialize, UbxFrameInfo, UbxFrameSerialize};
use crate::parser::{Packet, Parser};

pub struct ServerTty {
    device_name: String,
    parser: Parser,
    serial_port: Option<serial::SystemPort>,
}

impl ServerTty {
    pub fn new(device_name: &str) -> Self {
        let obj = Self {
            device_name: String::from(device_name),
            parser: Parser::new(),
            serial_port: None,
        };
        obj
    }

    // TODO: combine with open so that there is only one call to serial_port open()?
    pub fn detect_baudrate(&mut self) -> Result<usize, &'static str> {
        const BITRATES: [usize; 2] = [115200, 9600];

        self.serial_port = serial::open(&self.device_name).ok();
        if self.serial_port.is_none() {
            return Err("cannot open serial port");
        }

        for baud in BITRATES.iter() {
            info!("checking {} bps", baud);

            // configure port for desired bitrate
            match self.serial_port.as_mut() {
                Some(port) => {
                    let settings = serial::PortSettings {
                        baud_rate: serial::BaudRate::from_speed(*baud),
                        char_size: serial::Bits8,
                        parity: serial::ParityNone,
                        stop_bits: serial::Stop1,
                        flow_control: serial::FlowNone,
                    };

                    port.configure(&settings).unwrap();
                    port.set_timeout(Duration::from_millis(100)).unwrap();
                }
                _ => return Err("cannot configure serial port"),
            }

            // try to receive ubx or NMEA frames
            match self.scan() {
                true => {
                    return Ok(*baud);
                }
                _ => {
                    info!("bitrate {:?} not working", baud);
                    ();
                }
            }
        }

        // TODO: is serial port closed hereafter?

        Err("cannot detect bitrate")
    }

    pub fn open(&mut self, bitrate: usize) -> Result<(), &'static str> {
        info!("opening {} with {} bps", self.device_name, bitrate);

        // self.serial_port = serial::open(&self.device_name)?; // TODO: check ?
        self.serial_port = serial::open(&self.device_name).ok();
        if self.serial_port.is_none() {
            return Err("cannot open serial port");
        }

        // configure port for desired bitrate
        match self.serial_port.as_mut() {
            Some(port) => {
                let settings = serial::PortSettings {
                    baud_rate: serial::BaudRate::from_speed(bitrate),
                    char_size: serial::Bits8,
                    parity: serial::ParityNone,
                    stop_bits: serial::Stop1,
                    flow_control: serial::FlowNone,
                };

                port.configure(&settings).unwrap();
                port.set_timeout(Duration::from_millis(100)).unwrap();
                return Ok(());
            }
            _ => return Err("cannot configure serial port"),
        }
    }

    /*
    Poll a receiver status

    - sends the poll message
    - waits for receiver message with same class/id as poll message
    ((- retries in case no answer is received))
    */
    // TODO: Return code caller must handle
    pub fn poll<TPoll: UbxFrameInfo + UbxFrameSerialize, TAnswer: UbxFrameDeSerialize>(
        &mut self,
        frame_poll: &TPoll,
        frame_result: &mut TAnswer,
    ) -> Result<(), &'static str> {
        debug!("polling {}", frame_poll.name());

        // We expect a response frame with the exact same CID
        let wait_cid = frame_poll.cid();
        self.parser.empty_queue();
        self.parser.add_filter(wait_cid);

        // Serialize polling frame payload.
        // Only a few polling frames required payload, most come w/o.
        let data = frame_poll.to_bin();
        let res = self.send(&data);
        match res {
            Ok(_) => (),
            Err(e) => {
                warn!("poll: {}", e);
                return Err(e);    // TODO: What about clear_filter()?
            },
        }

        let payload = self.wait();
        match payload {
            Ok(packet) => {
                debug!("result received {:?}", packet.data);
                frame_result.from_bin(packet.data);
            }
            // BUG: clear_filter call not executed
            Err(e) => { 
                warn!("poll: timeout");
                return Err(e);    // TODO: What about clear_filter()?
            },
        }
        self.parser.clear_filter();

        Ok(())
    }

    /*
    Send a set message to modem and wait for acknowledge

    - creates bytes representation of set frame
    - sends set message to modem
    - waits for ACK/NAK
    */
    // TODO: Return code caller must handle
    pub fn set<TSet: UbxFrameSerialize + UbxFrameInfo>(&mut self, frame_set: &TSet) {
        debug!("setting {}", frame_set.name());

        // Wait for ACK-ACK and ACK-NAK
        self.parser.empty_queue();
        self.parser.add_filter(UbxCID::new(0x05, 0x01));
        self.parser.add_filter(UbxCID::new(0x05, 0x00));

        // Get frame data (header, cls, id, len, payload, checksum a/b)
        let data = frame_set.to_bin();
        // debug!("{:?}", data);
        let res = self.send(&data);
        match res {
            Ok(_) => (),
            Err(e) => warn!("set: {}", e), // TODO: Abort here? What about clear_filter()?
        }

        // Check proper response (ACK/NAK)
        let payload = self.wait();
        match payload {
            Ok(packet) => {
                debug!("ACK/NAK received {:?}", packet);
                // TODO: Check ACK/NAK and CLS, ID in ACK
                // packet.from_bin(packet.data);
            }
            Err(_) => warn!("set: timeout"),
        }

        self.parser.clear_filter();
    }

    /*
    Send a set message to modem without waiting for a response
    (fire and forget)

    This method is typically used for commands that are not ACKed, i.e.
    - cold start
    - change baudrate
    */
    pub fn fire_and_forget<TSet: UbxFrameSerialize + UbxFrameInfo>(&mut self, frame_set: &TSet) {
        debug!("firing {}", frame_set.name());

        let data = frame_set.to_bin();
        // debug!("{:?}", data);
        let res = self.send(&data);
        match res {
            Ok(_) => (),
            Err(e) => warn!("set: {}", e), // TODO: Abort here? What about clear_filter()?
        }
    }

    /*** Private ***/

    fn send(&mut self, data: &Vec<u8>) -> Result<(), &'static str> {
        // debug!("{} bytes to send {:?}", data.len(), data);
        let port = self.serial_port.as_mut().unwrap();

        let res = port.write(&data);
        match res {
            Ok(bytes_written) => {
                // debug!("{} bytes written", bytes_written);
                if bytes_written == data.len() {
                    Ok(())
                } else {
                    Err("Write error, not all data written")
                }
            }
            Err(_) => Err("Write error"),
        }
    }

    fn wait(&mut self) -> Result<Packet, &'static str> {
        let mut read_buffer = [0u8; 1024];
        let port = self.serial_port.as_mut().unwrap();

        let start = Instant::now();
        let mut elapsed = start.elapsed();
        while elapsed.as_millis() < 3000 {
            // Read data
            let res = port.read(&mut read_buffer[..]);
            match res {
                Ok(bytes_read) => {
                    // debug!("{} bytes read", bytes_read);
                    let data = read_buffer[0..bytes_read].to_vec();
                    debug!("{:?}", data);

                    // process() places all decoded frames in response_queue
                    self.parser.process(&data);
                }
                Err(_) => (), // no data, just continue
            }

            // Check if a packet could be decoded already
            let res = self.parser.packet();
            match res {
                Some(p) => {
                    // debug!("got desired packet {:?}", p);
                    return Ok(p);
                }
                _ => (), // No packet decoded so far, no problem just continue
            }

            elapsed = start.elapsed();
        }

        Err("timeout")
    }

    // TODO: result<bool, string>
    fn scan(&mut self) -> bool {
        let port = self.serial_port.as_mut().unwrap();
        // TODO: move to (dummy) parser_nmea module
        let mut nmea_buffer = String::new(); // hold combined string from all received data

        let start = Instant::now();
        let mut elapsed = start.elapsed();
        let ubx_frames = self.parser.frames_received();

        while elapsed.as_millis() < 2000 {
            let mut read_buffer = [0u8; 1024];
            let res = port.read(&mut read_buffer[..]);
            match res {
                Ok(bytes_read) => {
                    let data = read_buffer[0..bytes_read].to_vec();
                    // debug!("{:?}", data);
                    self.parser.process(&data);

                    // TODO: move to (dummy) parser_nmea module
                    let nmea = std::str::from_utf8(&read_buffer[0..bytes_read]);
                    if nmea.is_ok() {
                        // debug!("{:?}", nmea);
                        nmea_buffer.push_str(&nmea.unwrap());
                        // debug!("{:?}", nmea_buffer);
                    }
                }
                Err(_) => (), // no data, just continue
            }

            let _res = self.parser.packet();
            if self.parser.frames_received() - ubx_frames > 2 {
                info!("ubx frames received");
                return true;
            }

            let count = nmea_buffer.matches("GPGSV").count();
            if count >= 2 {
                info!("NMEA frames received");
                return true;
            }

            elapsed = start.elapsed();
        }
        false
    }
}
