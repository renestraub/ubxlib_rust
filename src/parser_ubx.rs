/*
 * Parser that tries to extract UBX frames from arbitrary byte streams
 *
 * Byte streams can also be NMEA or other frames. Unfortunately,
 * u-blox frame header also appears in NMEA frames (e.g. version
 * information). Such data will be detected by a checksum error
 */

use std::collections::HashSet;
use std::collections::VecDeque;

use log::{debug, warn};

use crate::checksum::Checksum;
use crate::cid::UbxCID;
use crate::frame::UbxFrame;

pub struct ParserUbx {
    crc_error_cid: UbxCID,
    rx_queue: VecDeque<UbxFrame>,
    wait_cids: HashSet<UbxCID>,
    checksum: Checksum,

    frames_rx: usize,

    state: State,
    msg_class: u8,
    msg_id: u8,
    msg_len: usize,
    msg_data: Vec<u8>,
    cka: u8,
    ckb: u8,
    ofs: usize,
}

const MAX_MESSAGE_LENGTH: usize = 1000;

#[derive(Debug)]
enum State {
    Init,
    Sync1,
    Class,
    ID,
    Len1,
    Len2,
    Data,
    CRC1,
    CRC2,
}

impl ParserUbx {
    pub fn new() -> Self {
        let mut obj = Self {
            crc_error_cid: UbxCID::new(0x00, 0x02),
            rx_queue: VecDeque::with_capacity(10),
            wait_cids: HashSet::<UbxCID>::new(),
            checksum: Checksum::new(),
            frames_rx: 0,
            state: State::Init,
            msg_class: 0,
            msg_id: 0,
            msg_len: 0,
            msg_data: Vec::with_capacity(1024),
            cka: 0,
            ckb: 0,
            ofs: 0,
        };
        obj._reset();
        obj
    }

    pub fn restart(&mut self) {
        // debug!("restarting from state {:?}", self.state);
        self.state = State::Init;
    }

    pub fn frames_received(&self) -> usize {
        return self.frames_rx;
    }

    pub fn set_filter(&mut self, cid: UbxCID) {
        self.set_filters(&[cid]);
    }

    pub fn set_filters(&mut self, cids: &[UbxCID]) {
        // Build HashSet from provided array
        let cid_set: HashSet<UbxCID> = cids.iter().cloned().collect();
        self.wait_cids = cid_set;
        debug!("acceptance CID {:?}", self.wait_cids);
    }

    pub fn empty_queue(&mut self) {
        self.rx_queue.clear();
    }

    pub fn packet(&mut self) -> Option<UbxFrame> {
        self.rx_queue.pop_front() // Some(UbxFrame) or None
    }

    // TODO: Change to plain array instead of vector
    pub fn process(&mut self, data: &Vec<u8>) {
        for byte in data.iter() {
            self.process_byte(*byte);
        }
    }

    pub fn process_byte(&mut self, data: u8) {
        match self.state {
            State::Init => self.state_init(data),
            State::Sync1 => self.state_sync(data),
            State::Class => self.state_class(data),
            State::ID => self.state_id(data),
            State::Len1 => self.state_len1(data),
            State::Len2 => self.state_len2(data),
            State::Data => self.state_data(data),
            State::CRC1 => self.state_crc1(data),
            State::CRC2 => self.state_crc2(data),
        }
    }

    fn state_init(&mut self, data: u8) {
        if data == 0xB5 {
            self.state = State::Sync1;
        }
    }

    fn state_sync(&mut self, data: u8) {
        if data == 0x62 {
            self._reset();
            self.state = State::Class;
        } else {
            self.state = State::Init;
        }
    }

    fn state_class(&mut self, data: u8) {
        self.msg_class = data;
        self.checksum.add(data);
        self.state = State::ID;
    }

    fn state_id(&mut self, data: u8) {
        self.msg_id = data;
        self.checksum.add(data);
        self.state = State::Len1;
    }

    fn state_len1(&mut self, data: u8) {
        self.msg_len = data as usize;
        self.checksum.add(data);
        self.state = State::Len2;
    }

    fn state_len2(&mut self, data: u8) {
        self.msg_len = self.msg_len + (data as usize * 256);
        // debug!("length {:?}", self.msg_len);
        self.checksum.add(data);

        if self.msg_len == 0 {
            self.state = State::CRC1;
        } else if self.msg_len > MAX_MESSAGE_LENGTH {
            warn!("invalid msg len {}", self.msg_len);
            self.state = State::Init;
        } else {
            self.ofs = 0;
            self.state = State::Data;
        }
    }

    fn state_data(&mut self, data: u8) {
        self.msg_data.push(data);
        // debug!("vec {:?}", self.msg_data);
        self.checksum.add(data);
        self.ofs += 1;

        if self.ofs == self.msg_len {
            self.state = State::CRC1;
        }
    }

    fn state_crc1(&mut self, data: u8) {
        self.cka = data;
        self.state = State::CRC2;
    }

    fn state_crc2(&mut self, data: u8) {
        self.ckb = data;

        // if checksum matches received checksum ..
        if self.checksum.matches(self.cka, self.ckb) {
            // debug!("checksum is ok");
            self.frames_rx += 1;

            // .. and frame passes filter ..
            let cid = UbxCID::new(self.msg_class, self.msg_id);
            // debug!("cid {:?}", cid);
            if self.wait_cids.contains(&cid) {
                // .. send CID and data as tuple to server
                // TODO: Here comes the fun part.
                // We have to copy over the buffer to the packet by value
                let packet = UbxFrame {
                    cid: cid,
                    data: self.msg_data.clone(),
                };
                self.rx_queue.push_back(packet);
            } else {
                debug!("no match - dropping {:?}, {} bytes", cid, self.msg_len);
            }
        } else {
            warn!("checksum error in frame, discarding");
            // debug!("computed {:?}", self.checksum);
            // debug!("{self.msg_class:02x} {self.msg_id:02x} {binascii.hexlify(self.msg_data)}')
            let crc_error_message = UbxFrame {
                cid: self.crc_error_cid,
                data: vec![],
            };
            self.rx_queue.push_back(crc_error_message);
        }

        self.state = State::Init;
    }

    fn _reset(&mut self) {
        self.msg_class = 0;
        self.msg_id = 0;
        self.msg_len = 0;
        self.msg_data.clear();
        self.cka = 0;
        self.ckb = 0;
        self.ofs = 0;
        self.checksum.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /* B5 62 13 40 18 00 10 00 00 12 E4 07 09 05 06 28 30 00 40 28 EF 0C 0A 00 00 00 00 00 00 00 51 AC   */
    /* hdr  | <--                                 checksum                                  --> | chksum */
    const FRAME_1: [u8; 32] = [
        0xB5, 0x62, 0x13, 0x40, 0x18, 0x00, 0x10, 0x00, 0x00, 0x12, 0xE4, 0x07, 0x09, 0x05, 0x06,
        0x28, 0x30, 0x00, 0x40, 0x28, 0xEF, 0x0C, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x51, 0xAC,
    ];

    #[test]
    #[should_panic]
    fn no_frames() {
        let mut uut = ParserUbx::new();
        let res = uut.packet(); // None
        let _packet = res.unwrap(); // should panic
    }

    #[test]
    fn process_byte() {
        let mut uut = ParserUbx::new();
        uut.set_filter(UbxCID::new(0x13, 0x40));
        for byte in FRAME_1.iter() {
            uut.process_byte(*byte);
        }

        let res = uut.packet();
        let packet = res.unwrap(); // panics if None
        assert_eq!(packet.cid, UbxCID::new(0x13, 0x40));
    }

    #[test]
    fn process_array() {
        let mut uut = ParserUbx::new();
        uut.set_filter(UbxCID::new(0x13, 0x40));
        uut.process(&FRAME_1.to_vec());

        let res = uut.packet();
        let packet = res.unwrap(); // panics if None
        assert_eq!(packet.cid, UbxCID::new(0x13, 0x40));
    }

    #[test]
    fn passes_filter() {
        let mut uut = ParserUbx::new();
        uut.set_filter(UbxCID::new(0x13, 0x40));
        uut.process(&FRAME_1.to_vec());

        let res = uut.packet();
        let packet = res.unwrap(); // panics if None
        assert_eq!(packet.cid, UbxCID::new(0x13, 0x40));
    }

    #[test]
    fn dropped_cls() {
        let mut uut = ParserUbx::new();
        uut.set_filter(UbxCID::new(0x12, 0x40));
        uut.process(&FRAME_1.to_vec());

        let res = uut.packet();
        assert_eq!(res.is_none(), true);
    }

    #[test]
    fn dropped_id() {
        let mut uut = ParserUbx::new();
        uut.set_filter(UbxCID::new(0x13, 0x41));
        uut.process(&FRAME_1.to_vec());

        let res = uut.packet();
        assert_eq!(res.is_none(), true);
    }

    #[test]
    fn multiple_filters() {
        let mut uut = ParserUbx::new();
        let cids = [
            UbxCID::new(0x12, 0x12),
            UbxCID::new(0x13, 0x40),
            UbxCID::new(0xFF, 0x00),
            UbxCID::new(0xFF, 0x00),
        ];
        uut.set_filters(&cids);
        uut.process(&FRAME_1.to_vec());

        let res = uut.packet();
        let packet = res.unwrap(); // panics if None
        assert_eq!(packet.cid, UbxCID::new(0x13, 0x40));
    }

    #[test]
    fn change_filter() {
        let mut uut = ParserUbx::new();
        uut.set_filter(UbxCID::new(0x13, 0x40));
        uut.process(&FRAME_1.to_vec());
        let res = uut.packet();
        assert_eq!(res.is_some(), true);

        uut.set_filter(UbxCID::new(0x00, 0x00));
        uut.process(&FRAME_1.to_vec());
        let res = uut.packet();
        assert_eq!(res.is_none(), true);
    }

    #[test]
    fn crc_error() {
        /* B5 62 13 40 18 00 10 00 00 12 E4 07 09 05 06 28 30 00 40 28 EF 0C 0A 00 00 00 00 00 00 00 51 AC   */
        /* hdr  | <--                                 checksum                                  --> | chksum */
        let frame: [u8; 32] = [
            0xB5,
            0x62,
            0x13,
            0x40,
            0x18,
            0x00,
            0x10,
            0x00,
            0x00,
            0x12,
            0xE4,
            0x07,
            0x09,
            0x05,
            0x06,
            0x28,
            0x30,
            0x00,
            0x40,
            0x28,
            0xEF,
            0x0C,
            0x0A,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x51,
            0xAC + 1,
        ];
        let mut uut = ParserUbx::new();
        uut.set_filter(UbxCID::new(0x13, 0x40));
        uut.process(&frame.to_vec());
        let res = uut.packet(); // crc packet
        assert_eq!(res.is_some(), true);
        let packet = res.unwrap(); // panics if None
        assert_eq!(packet.cid, UbxCID::new(0x00, 0x02));
    }

    #[test]
    fn invalid_length() {
        let frame: [u8; 32] = [
            0xB5, 0x62, 0x13, 0x40, 0xe9, 0x03, 0x10, 0x00, 0x00, 0x12, 0xE4, 0x07, 0x09, 0x05,
            0x06, 0x28, 0x30, 0x00, 0x40, 0x28, 0xEF, 0x0C, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x51, 0xAC,
        ];
        let mut uut = ParserUbx::new();
        uut.set_filter(UbxCID::new(0x13, 0x40));
        uut.process(&frame.to_vec());
        let res = uut.packet(); // Should be None because frame is too long (MAX_MESSAGE_LENGTH)
        assert_eq!(res.is_none(), true);
    }

    #[test]
    fn filters() {
        let mut uut = ParserUbx::new();

        uut.set_filters(&[UbxCID::new(0x05, 0x01), UbxCID::new(0x05, 0x00)]);
        assert_eq!(uut.wait_cids.len(), 2);
        assert!(uut.wait_cids.contains(&UbxCID::new(0x05, 0x00)));
        assert!(uut.wait_cids.contains(&UbxCID::new(0x05, 0x01)));

        let cids = [UbxCID::new(0x05, 0x00), UbxCID::new(0x05, 0x01)];
        uut.set_filters(&cids);
        assert_eq!(uut.wait_cids.len(), 2);
        assert!(uut.wait_cids.contains(&UbxCID::new(0x05, 0x00)));
        assert!(uut.wait_cids.contains(&UbxCID::new(0x05, 0x01)));
    }
}
