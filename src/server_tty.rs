use std::time::Instant;
use std::time::Duration;

use crate::cid::UbxCID;
use crate::frame::{UbxFrameInfo, UbxFrameSerialize, UbxFrameDeSerialize};
use crate::parser::{Parser, Packet};

use std::io::prelude::*;
use serial::prelude::*;


// Port is configured for 115'200, 8N1
const SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate:    serial::Baud115200,
    char_size:    serial::Bits8,
    parity:       serial::ParityNone,
    stop_bits:    serial::Stop1,
    flow_control: serial::FlowNone,
};


pub struct ServerTty {
    // device_name: String,
    parser: Parser,
    serial_port: serial::SystemPort,
}

impl ServerTty {
    // TODO: Result return code to handle errors
    pub fn new(device_name: &str) -> Self {
        let mut obj = Self { 
            // device_name: String::from(device_name),
            parser: Parser::new(),
            serial_port: serial::open(&device_name).unwrap(),   // TODO: How do we do error check here?
        };

        // let mut port = serial::open(&self.device_name).unwrap(); // ?;
        obj.serial_port.configure(&SETTINGS).unwrap(); // ?;
        obj.serial_port.set_timeout(Duration::from_secs(1000)).unwrap(); //?;

        obj
    }    

    /*
    Poll a receiver status

    - sends the poll message
    - waits for receiver message with same class/id as poll message
    ((- retries in case no answer is received))
    */
    // TODO: Return code caller must handle
    pub fn poll<TPoll: UbxFrameInfo + UbxFrameSerialize, TAnswer: UbxFrameDeSerialize>(&mut self, frame_poll: &TPoll, frame_result: &mut TAnswer)
    {
        println!("polling {}", frame_poll.name());

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
            Err(e) => println!("poll: {}", e),    // TODO: Abort here? What about clear_filter()?
        }

        let payload = self.wait();
        match payload {
            Ok(packet) => { 
                // println!("ok {:?}", packet.data);   // ACK or NAK received
                frame_result.from_bin(packet.data);
            },
            // BUG: clear_filter call not executed
            Err(_) => println!("poll: timeout"),
        }

        self.parser.clear_filter();
    }

    /*
    Send a set message to modem and wait for acknowledge

    - creates bytes representation of set frame
    - sends set message to modem
    - waits for ACK/NAK
    */
    // TODO: Return code caller must handle
    pub fn set<TSet: UbxFrameSerialize + UbxFrameInfo>(&mut self, frame_set: &TSet) {
        println!("setting {}", frame_set.name());

        // Wait for ACK-ACK and ACK-NAK
        self.parser.empty_queue();
        self.parser.add_filter(UbxCID::new(0x05, 0x01));
        self.parser.add_filter(UbxCID::new(0x05, 0x00));

        // Get frame data (header, cls, id, len, payload, checksum a/b)
        let data = frame_set.to_bin();
        // println!("{:?}", data);
        let res = self.send(&data);
        match res {
            Ok(_) => (),
            Err(e) => println!("set: {}", e),    // TODO: Abort here? What about clear_filter()?
        }

        // Check proper response (ACK/NAK)
        let payload = self.wait();
        match payload {
            Ok(packet) => { 
                println!("ok {:?}", packet);
                // TODO: Check ACK/NAK and CLS, ID in ACK
                // f2.from_bin(packet.data);
            },
            Err(_) => println!("set: timeout"),
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
        println!("firing {}", frame_set.name());

        let data = frame_set.to_bin();
        // println!("{:?}", data);
        let res = self.send(&data);
        match res {
            Ok(_) => (),
            Err(e) => println!("set: {}", e),    // TODO: Abort here? What about clear_filter()?
        }
    }


    /*** Private ***/

    fn send(&mut self, data: &Vec<u8>) -> Result<(), &'static str> {
        // println!("{} bytes to send {:?}", data.len(), data);

        let res = self.serial_port.write(&data);
        match res {
            Ok(bytes_written) => {
                // println!("{} bytes written", bytes_written);
                if bytes_written == data.len() {
                    Ok(())
                }
                else {
                    Err("Write error, not all data written")
                }
            },
            Err(_) => Err("Write error"),
        }
    }

    fn wait(&mut self) -> Result<Packet, &'static str> {
        let mut read_buffer = [0u8; 1024];

        let start = Instant::now();
        let mut elapsed = start.elapsed();
        while elapsed.as_millis() < 3000 {
            // Read data 
            // TODO: Check why only 48 bytes are read at once
            let res = self.serial_port.read(&mut read_buffer[..]);
            match res {
                Ok(bytes_read) => { 
                    // println!("{} bytes read", bytes_read); 
                    let data = read_buffer[0..bytes_read].to_vec();
                    // println!("{:?}", data);
                    // process() places all decoded frames in response_queue
                    self.parser.process(&data);
                },
                Err(_) => (),   // no data, just continue
            }

            // Check if a packet could be decoded already
            let res = self.parser.packet();
            match res {
                Some(p) => { 
                    // println!("got desired packet {:?}", p);
                    return Ok(p);
                }
                _ => ()     // No packet decoded so far, no problem just continue
            }

            elapsed = start.elapsed();
        }

        Err("timeout")
    }
}
