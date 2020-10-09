use std::io::prelude::*;
use std::time::Duration;
use std::time::Instant;

use log::{debug, warn};
use serial::prelude::*;

use crate::cid::UbxCID;
use crate::error::Error;
use crate::frame::UbxFrame;
use crate::frame::{UbxFrameDeSerialize, UbxFrameInfo, UbxFrameSerialize};
use crate::parser_ubx::ParserUbx;
use crate::ubx_ack::UbxAck;

pub struct ServerTty {
    device_name: String,
    parser: ParserUbx,
    serial_port: Option<serial::SystemPort>,
    crc_error_cid: UbxCID,
    max_retries: usize,
    retry_delay_in_ms: u128,
}

impl ServerTty {
    pub fn new(device_name: &str) -> Self {
        let obj = Self {
            device_name: String::from(device_name),
            parser: ParserUbx::new(),
            serial_port: serial::open(device_name).ok(), // convert Result to Option value
            crc_error_cid: UbxCID::new(0x00, 0x02),
            max_retries: 5,
            retry_delay_in_ms: 3000,
        };
        obj
    }

    pub fn set_baudrate(&mut self, bitrate: usize) -> Result<(), Error> {
        debug!("opening {} with {} bps", self.device_name, bitrate);

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

                port.configure(&settings)
                    .map_err(|_err| Error::SerialPortConfigFailed)?;
                port.set_timeout(Duration::from_millis(100))
                    .map_err(|_err| Error::SerialPortConfigFailed)?;
                return Ok(());
            }
            _ => return Err(Error::SerialPortNotFound),
        }
    }

    pub fn scan(&mut self) -> Result<(), Error> {
        let port = self.serial_port.as_mut().unwrap();
        // TODO: move to (dummy) parser_nmea module
        let mut nmea_buffer = String::new(); // hold combined string from all received data

        let start = Instant::now();
        let mut elapsed = start.elapsed();
        let ubx_frames = self.parser.frames_received();

        self.parser.restart();
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
                debug!("ubx frames received");
                return Ok(());
            }

            let count = nmea_buffer.matches("GPGSV").count();
            if count >= 2 {
                debug!("NMEA frames received");
                return Ok(());
            }

            elapsed = start.elapsed();
        }

        Err(Error::ModemNotResponding)
    }

    /*
    Poll a receiver status

    - sends the poll message
    - waits for receiver message with same class/id as poll message
    - retries in case no answer is received
    */
    pub fn poll<TPoll: UbxFrameInfo + UbxFrameSerialize, TAnswer: UbxFrameDeSerialize>(
        &mut self,
        frame_poll: &TPoll,
        frame_result: &mut TAnswer,
    ) -> Result<(), Error> {
        debug!("polling {}", frame_poll.name());

        // We expect a response frame with the exact same CID
        let wait_cid = frame_poll.cid();
        self.parser.set_filter(wait_cid);

        // Serialize polling frame payload.
        // Only a few polling frames required payload, most come w/o.
        let data = frame_poll.to_bin();

        for retry in 0..self.max_retries {
            self.parser.empty_queue();
            self.send(&data)?;

            // Check if requested frame is received
            match self.wait() {
                Ok(packet) => {
                    debug!("result received {:?} {:?}", packet.cid, packet.data);
                    frame_result.from_bin(&packet.data);
                    return Ok(());
                }
                Err(_) => {
                    warn!("poll: timeout, retrying {}", retry + 1);
                }
            }
        }

        Err(Error::ModemNotResponding)
    }

    /*
    Send a set message to modem and wait for acknowledge

    - creates bytes representation of set frame
    - sends set message to modem
    - waits for ACK/NAK
    */
    pub fn set<TSet: UbxFrameSerialize + UbxFrameInfo>(
        &mut self,
        frame_set: &TSet,
    ) -> Result<(), Error> {
        debug!("setting {}", frame_set.name());

        // Wait for ACK-ACK / ACK-NAK
        let cids = [UbxCID::new(0x05, 0x00), UbxCID::new(0x05, 0x01)];
        self.parser.set_filters(&cids);

        // Get frame data (header, cls, id, len, payload, checksum a/b)
        let data = frame_set.to_bin();

        for retry in 0..self.max_retries {
            self.parser.empty_queue();
            self.send(&data)?;

            // Check proper response (ACK/NAK)
            match self.wait() {
                Ok(packet) => match self.check_ack_nak(&packet, frame_set.cid()) {
                    Ok(_) => return Ok(()),
                    Err(Error::ModemNAK) => return Err(Error::ModemNAK),
                    Err(_) => (), // Ignore other errors, as for example NAK for another request
                },
                Err(_) => {
                    warn!("set: timeout, retrying {}", retry + 1);
                }
            }
        }

        Err(Error::ModemNotResponding)
    }

    /*
    Send a set message to modem without waiting for a response
    (fire and forget)

    This method is typically used for commands that are not ACKed, i.e.
    - cold start
    - change baudrate
    */
    pub fn fire_and_forget<TSet: UbxFrameSerialize + UbxFrameInfo>(
        &mut self,
        frame_set: &TSet,
    ) -> Result<(), Error> {
        debug!("firing {}", frame_set.name());

        let data = frame_set.to_bin();
        self.send(&data)?;

        Ok(())
    }

    /*** Private ***/

    fn send(&mut self, data: &Vec<u8>) -> Result<(), Error> {
        // debug!("{} bytes to send {:?}", data.len(), data);
        let port = self.serial_port.as_mut().unwrap();
        let res = port.write(&data);
        match res {
            Ok(bytes_written) => {
                if bytes_written == data.len() {
                    Ok(())
                } else {
                    // debug!("write error, not all data written")
                    Err(Error::SerialPortSendFailed)
                }
            }
            // debug!("write error"),
            Err(_) => Err(Error::SerialPortSendFailed),
        }
    }

    fn wait(&mut self) -> Result<UbxFrame, Error> {
        let mut read_buffer = [0u8; 1024];
        let port = self.serial_port.as_mut().unwrap();

        let start = Instant::now();
        let mut elapsed = start.elapsed();

        self.parser.restart();
        while elapsed.as_millis() < self.retry_delay_in_ms {
            match port.read(&mut read_buffer[..]) {
                Ok(bytes_read) => {
                    let data = read_buffer[0..bytes_read].to_vec();
                    // process() places all decoded frames in response_queue
                    self.parser.process(&data);
                }
                Err(_) => (), // no data, just continue
            }

            // Check if a packet could be decoded already
            let res = self.parser.packet();
            match res {
                Some(p) => {
                    if p.cid != self.crc_error_cid {
                        return Ok(p);
                    } else {
                        warn!("checksum error in frame, discarding");
                    }
                }
                _ => (), // No packet decoded so far, no problem just continue
            }

            elapsed = start.elapsed();
        }

        Err(Error::ModemNotResponding)
    }

    fn check_ack_nak(&self, packet: &UbxFrame, set_cid: UbxCID) -> Result<(), Error> {
        let mut ack_nak = UbxAck::new(packet.cid.id());
        ack_nak.from_bin(&packet.data);
        // debug!("ack/nak {:?} - {:?}", ack_nak.ack_cid(), set_cid);

        if ack_nak.ack_cid() == set_cid {
            // CID in ACK/NAK frame matches our request
            match ack_nak.cid.id() {
                // TODO: Not nice to hard-code 0x01 here.. any better way?
                0x01 => Ok(()),
                0x00 => Err(Error::ModemNAK),
                _ => Err(Error::ModemNAK),
            }
        } else {
            // ACK/NAK must be for another request
            Err(Error::ModemUnexpectedAckNak)
        }
    }
}
