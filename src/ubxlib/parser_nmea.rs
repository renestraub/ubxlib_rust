/*
 * Parser that tries to read NMEA drames from arbitrary byte stream
 *
 * $GNRMC,155215.00,A,4719.13883,N,00758.44996,E,0.259,,171020,2.47,E,A*3E\r\n
 */

use log::debug;

pub struct ParserNmea {
    frames_rx: usize,
    state: State,
    msg_data: Vec<u8>,
    checksum: u8,
    checksum_data: u8,
}

#[derive(Debug)]
enum State {
    WaitSync,
    Data,
    ChkSum1,
    ChkSum2,
    LineEnd,
}

impl ParserNmea {
    pub fn new() -> Self {
        let mut obj = Self {
            frames_rx: 0,
            state: State::WaitSync,
            msg_data: Vec::with_capacity(1024),
            checksum: 0,
            checksum_data: 0,
        };
        obj._reset();
        obj
    }

    pub fn frames_received(&self) -> usize {
        self.frames_rx
    }

    pub fn process(&mut self, data: &[u8]) {
        for byte in data.iter() {
            self.process_byte(*byte);
        }
    }

    pub fn process_byte(&mut self, data: u8) {
        // global state change when sync character is seen
        if data as char == '$' {
            self._reset();
            self.state = State::Data;
        } else {
            match self.state {
                State::WaitSync => self.state_wait_sync(data),
                State::Data => self.state_data(data),
                State::ChkSum1 => self.state_chksum1(data),
                State::ChkSum2 => self.state_chksum2(data),
                State::LineEnd => self.state_lineend(data),
            }
        }
    }

    fn state_wait_sync(&mut self, data: u8) {
        if data as char == '$' {
            self._reset();
            self.state = State::Data;
        }
    }

    fn state_data(&mut self, data: u8) {
        if data as char == '*' {
            self.state = State::ChkSum1;
        } else {
            self.msg_data.push(data);
            self.checksum_data ^= data;
        }
    }

    fn state_chksum1(&mut self, data: u8) {
        self.checksum = Self::_to_bin(data) << 4;
        self.state = State::ChkSum2;
    }

    fn state_chksum2(&mut self, data: u8) {
        self.checksum |= Self::_to_bin(data);

        if self.checksum == self.checksum_data {
            self.frames_rx += 1;

            let nmea = std::str::from_utf8(&self.msg_data);
            if nmea.is_ok() {
                debug!("{:?}", nmea.unwrap());
            }
        } else {
            debug!("Checksum error {} - {}", self.checksum, self.checksum_data);
        }

        self.state = State::LineEnd;
    }

    fn state_lineend(&mut self, data: u8) {
        if data as char == '\n' {
            self.state = State::WaitSync;
        }
    }

    fn _reset(&mut self) {
        self.msg_data.clear();
        self.checksum = 0;
        self.checksum_data = 0;
    }

    fn _to_bin(data: u8) -> u8 {
        match data as char {
            '0'..='9' => data - b'0',
            'a'..='f' => data - b'a' + 10,
            'A'..='F' => data - b'A' + 10,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_bin() {
        assert_eq!(ParserNmea::_to_bin('0' as u8), 0);
        assert_eq!(ParserNmea::_to_bin('9' as u8), 9);
        assert_eq!(ParserNmea::_to_bin('a' as u8), 10);
        assert_eq!(ParserNmea::_to_bin('A' as u8), 10);
        assert_eq!(ParserNmea::_to_bin('f' as u8), 15);
        assert_eq!(ParserNmea::_to_bin('F' as u8), 15);

        assert_eq!(ParserNmea::_to_bin('x' as u8), 0);
    }

    #[test]
    fn ok() {
        let data = "$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6A";
        let mut uut = ParserNmea::new();
        assert_eq!(uut.frames_received(), 0);
        uut.process(&data.as_bytes());
        assert_eq!(uut.frames_received(), 1);
    }

    #[test]
    fn wrong_checksum() {
        let data = "$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6B";
        let mut uut = ParserNmea::new();
        uut.process(&data.as_bytes());
        assert_eq!(uut.frames_received(), 0);
    }

    #[test]
    fn checksum_missing() {
        let data_fail = "$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W";
        let data_ok = "$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6A";
        let mut uut = ParserNmea::new();
        uut.process(&data_fail.as_bytes());
        assert_eq!(uut.frames_received(), 0);

        // now next line must be properly read
        uut.process(&data_ok.as_bytes());
        assert_eq!(uut.frames_received(), 1);
    }
}
