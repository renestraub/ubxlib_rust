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

    pub fn matches(&self, cka: u8, ckb: u8) -> bool {
        return self.cka == cka && self.ckb == ckb;
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
        let frame = vec![
            0x13, 0x40, 0x18, 0x00, 0x10, 0x00, 0x00, 0x12, 0xE4, 0x07, 0x09, 0x05, 0x06, 0x28,
            0x30, 0x00, 0x40, 0x28, 0xEF, 0x0C, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let mut uut = Checksum::new();
        for byte in frame.iter() {
            uut.add(*byte);
        }
        let ok = uut.matches(0x51, 0xAC);
        assert_eq!(ok, true);
    }

    /*
        #[test]
        fn calculation_2() {
            /* UBX-NAV-VELECEF 20 */
            /*      Cls Id| Len |  iTow     |  ecefVX   |  ecefVY   | ecefVZ    |  sAcc     |        */
            /* B5 62 01 11 14 00 a8 a4 57 01 ff ff ff ff ff ff ff ff fe ff ff ff 19 00 00 00 B5 62   */
            /*       01 11 14 00 f8 98 35 02 01 00 00 00 00 00 00 00 00 00 00 00 15 00 00 00 4d 75d */

            /* hdr  | <--                         checksum                              --> | chksum */
            let frame = vec![0x01, 0x11, 0x00, 0x14, 0xa8, 0xa4, 0x57, 0x01, 0xff, 0xff, 0xff, 0xff, 0xff,
                             0xff, 0xff, 0xff, 0xfe, 0xff, 0xff, 0xff, 0x19, 0x00, 0x00, 0x00];
            let mut uut = Checksum::new();
            for byte in frame.iter() {
                uut.add(*byte);
            }
            let (a,b) = uut.value();
            println!("{:02x} {:02x}", a, b);    // --> d6 (214), 9c (156)
            let ok = uut.matches(0x51, 0xAC);
            assert_eq!(ok, true);
        }


        #[test]
        fn calculation_3() {
            /* UBX-NAV-DOP (0x01 0x04) ? */
            /*      Cls Id| Len |           |           |           |           |     |      */
            /* B5 62 01 00 12 00 a8 a4 57 01 f3 00 d3 00 79 00 c1 00 54 00 3d 00 3a 00 86 55 */
            /* hdr  | <--                         checksum                              --> | chksum */
            let frame = vec![0x01, 0x00, 0x12, 0x00, 0xa8, 0xa4, 0x57, 0x01, 0xf3, 0x00, 0xd3,
                             0x00, 0x79, 0x00, 0xc1, 0x00, 0x54, 0x00, 0x3d, 0x00, 0x3a, 0x00, 0x86, 0x55];
            let mut uut = Checksum::new();
            for byte in frame.iter() {
                uut.add(*byte);
            }
            let (a,b) = uut.value();
            println!("{:02x} {:02x}", a, b);    // --> d6 (214), 9c (156)
            let ok = uut.matches(0x51, 0xAC);
            assert_eq!(ok, true);
        }

        #[test]
        fn calculation_4() {
            /* UBX-NAV-POSECEF */
            /* checksum computed (90, 79) */
            /*      Cls Id| Len |           |           |           |           |           |        */
            /* B5 62 01 01 14 00 f8 59 78 03 12 48 00 19 f7 f8 94 03 af 34 d0 1b b0 01 00 00 236 75  */
            /* hdr  | <--                         checksum                              --> | chksum */
            let frame = vec![0x01, 0x01, 0x14, 0x00, 0xf8, 0x59, 0x78, 0x03, 0x12, 0x48, 0x00, 0x19, 0xf7, 0xf8, 0x94, 0x03, 0xaf, 0x34, 0xd0, 0x1b, 0xb0, 0x01, 0x00, 0x00];
            let mut uut = Checksum::new();
            for byte in frame.iter() {
                uut.add(*byte);
            }
            let (a,b) = uut.value();
            println!("{:02x} {:02x}", a, b);    // --> 0x5a (90), 0x4f (79) -> Ok, but payload checksum is wrong: 0xEC (236), 0x4B (75)
            let ok = uut.matches(0x51, 0xAC);
            assert_eq!(ok, true);

            // 0x4B   0100 1011    0x5A  0101 1010
            // 0x4F   0100 1111    0xEC  1110 1100
            //              ^
        }
    */
}

/*
UBX-NAV-POSECEF

2020-09-13 17:21:02,297 WARNING  checksum computed (67, 164)
2020-09-13 17:21:02,302 WARNING  01 01 0012 b'985ab9039700850046006f0049003a002d00' 70 227  <-- Length is wrong should be 0x14 !


2020-09-13 16:10:02,349 WARNING  checksum computed (90, 79)
2020-09-13 16:10:02,354 WARNING  01 01 0014 b'f859780312480019f7f89403af34d01bb0010000' 236 75

2020-09-13 13:22:46,679 WARNING  checksum computed (194, 142)
2020-09-13 13:22:46,683 WARNING  01 01 0014 b'c03adf024049921943f894033434d01b77010000' 200 166


UBX-NAV-DOP

2020-09-13 17:03:46,634 WARNING  checksum computed (181, 230)
2020-09-13 17:03:46,636 WARNING  01 04 0012 b'268fa90387007a0039006800400031002a00' 47 122


UBX-NAV-VELECEF

2020-09-13 11:47:07,722 WARNING  checksum computed (212, 131)
2020-09-13 11:47:07,725 WARNING  01 11 0014 b'c8a8230201000000010000000100000016000000' 56 139

2020-09-13 18:20:20,651 WARNING  checksum computed (203, 112)
2020-09-13 18:20:20,655 WARNING  00 11 0014 b'f0a8ef0302f00000030000000200000025000000' 220 120




UBX-NAV-SVINFO

2020-09-13 12:38:11,933 WARNING  checksum computed (209, 28)
2020-09-13 12:38:11,938 WARNING  01 30 0128 b'8869b602 18 04000007010f0725388200a7ffffff08030f0726505c011301000001040c010008bd00000000000f060c01000a33010000000012090d041a0dd1002dfcffff0e0b0f041a179c005afeffff05110c0100032101000000000a130d071f1e3501ecffffff0b160f073441430081ffffff161f0f072d164600730000000c200e07270b2900e7000000117b1401001f950000000000027f0d0722137c007dfcffffff80040000016500000000000d881401001fb8000000000003460d07282a2a008cffffff09470d071d4d1701befbffff04480c01001ced0000000000ff4e0c000080540100000000144f0d07230b1a00f8ffffff10500c0415064e0076fdffff06550d041920a2006cfeffff00560c041248e5000000000017570d071e154701d0020000' 164 133
Len bytearray = 592/2 = 296 = 0x128 == Ok (8+12*24)


UBX-NAV-SAT

2020-09-13 17:47:22,255 WARNING  checksum computed (155, 12)
2020-09-13 17:47:22,261 WARNING  01 35 011c b'7876d103 01 17 000000021b17ed00d4ff1e19000000051c3b24014c001f19000000060004c6003a00111900000007333e4100efff1f1901000009291e580001001f190100000d141c13010c001c19000000101a0216001cff17190100001c200b9e008dff1f190000001e224dad00e7ff1f190100017b001f9500000001120000017f23137c0014001f19000001800001650000001012000001880024b80000000112000006012d3367001c001f19000006022435430100001f19000006030004310100001012000006082a0b7300edff1f190000060a29111e00f3ff1f190000060b334150002d001f190000060c1e2bb100eaff1f1900000611000a0a0100001119000006121f123a01bd0014190000061317076601290014120000' 249 16
Len bytearray = 568/2 = 284, 11Ch == Ok

2020-09-13 17:59:02,265 WARNING  checksum computed (32, 216)
2020-09-13 17:59:02,270 WARNING  01 35 0140 b'd824dc03 01 1a 000000020012e90000001119000000051f3f1b0105001f19000000060000c400000011190000000733393d00f7ff1f19010000092a1a5b0003001f190100000d1920160100ff1c190000000f00001a01000010120000001200025301000011120000001b13002800000014120000001c22109c002a001f190100001e1e51980037001f190100017b001f9500000001120000017f23137c00d7ff1f19000001800001650013001012000001880024b80000000112000006012f2f6d0003001f1900000602263a47011a001f19000006031309340123001412000006081d067700d2ff17190000060a280b1d00e4ff1f190000060b343f410003001f190000060c1e32b10026ff1f190000060d0003c70000001112000006110005050100001119000006121210330100001419000006132809610134ff17190000' 158 188
Len bytearray = 640/2 = 320, 140h == Ok


UBX-NAV-PVT

2020-09-13 18:37:31,623 WARNING  checksum computed (31, 114)
2020-09-13 18:37:31,628 WARNING  01 07 005c b'4864ff03e407090d12251f3f0900000044f1fcff0323ea0923b4c004834f001c91740800d6bb07005408000053090000f3ffffffdfffffff2000000023000000f4d95a012f01000049072300b50000007e922d23f4d95a01f6003200' 83 10
Len bytearray = 184/2 = 92, 5ch == Ok

*/
