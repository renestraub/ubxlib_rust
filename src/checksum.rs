/*
 * u-blox checksum computation
 */

 #[derive(Debug)]
pub struct Checksum {
    cka: u8,
    ckb: u8,
}

impl Checksum {
    pub fn new() -> Self {
        Self { cka: 0, ckb: 0 }
    }

    pub fn value(&self) -> (u8, u8) {
        (self.cka, self.ckb)
    }

    #[allow(dead_code)]
    pub fn matches(&self, cka: u8, ckb: u8) -> bool {
        return self.cka == cka && self.ckb == ckb
    }

    pub fn reset(&mut self) {
        self.cka = 0;
        self.ckb = 0;
    }

    pub fn add(&mut self, byte: u8) {
        self.cka = (self.cka as usize + byte as usize) as u8;
        self.ckb = (self.ckb as usize + self.cka as usize) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction() {
        let dut = Checksum::new();
        let ok = dut.matches(0, 0);
        assert_eq!(ok, true);
    }

    #[test]
    fn reset() {
        let mut dut = Checksum::new();
        dut.add(0xF0);
        dut.add(0xE0);

        dut.reset();
        let ok = dut.matches(0, 0);
        assert_eq!(ok, true);
    }


    #[test]
    fn calculation() {
        /* B5 62 13 40 18 00 10 00 00 12 E4 07 09 05 06 28 30 00 40 28 EF 0C 0A 00 00 00 00 00 00 00 51 AC   */
        /* hdr  | <--                                 checksum                                  --> | chksum */
        let frame = vec![0x13, 0x40, 0x18, 0x00, 0x10, 0x00, 0x00, 0x12, 0xE4, 0x07, 0x09, 0x05, 
                         0x06, 0x28, 0x30, 0x00, 0x40, 0x28, 0xEF, 0x0C, 0x0A, 0x00, 0x00, 0x00, 
                         0x00, 0x00, 0x00, 0x00];
        let mut uut = Checksum::new();
        for byte in frame.iter() {
            uut.add(*byte);
        }
        let ok = uut.matches(0x51, 0xAC);
        assert_eq!(ok, true);
    }
}
