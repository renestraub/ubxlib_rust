use std::fmt;

#[derive(Clone, Copy, Default, Hash, PartialEq, Eq)]
pub struct UbxCID {
    cls: u8,
    id: u8,
}

impl UbxCID {
    pub fn new(cls: u8, id: u8) -> Self {
        Self { cls, id }
    }

    pub fn cls(&self) -> u8 {
        self.cls
    }

    pub fn id(&self) -> u8 {
        self.id
    }
}

impl fmt::Debug for UbxCID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CID: 0x{:02x} 0x{:02x}", self.cls, self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatting() {
        let dut = UbxCID::new(1, 2);
        assert_eq!(format!("{:?}", dut), "CID: 0x01 0x02");

        let dut = UbxCID::new(0x13, 0x00);
        assert_eq!(format!("{:?}", dut), "CID: 0x13 0x00");
    }

    #[test]
    fn get_cls() {
        let dut = UbxCID::new(0, 2);
        assert_eq!(dut.cls(), 0);
        let dut = UbxCID::new(255, 0);
        assert_eq!(dut.cls(), 255);
    }

    #[test]
    fn get_id() {
        let dut = UbxCID::new(1, 0);
        assert_eq!(dut.id(), 0);
        let dut = UbxCID::new(0, 255);
        assert_eq!(dut.id(), 255);
    }

    #[test]
    fn comparison_1() {
        let dut1 = UbxCID::new(1, 2);
        let dut2 = UbxCID::new(1, 2);
        assert_eq!(dut1, dut2);
    }

    #[test]
    fn comparison_2() {
        let dut1 = UbxCID::new(1, 2);
        let dut2 = UbxCID::new(1, 3);
        assert_ne!(dut1, dut2);
    }

    #[test]
    fn comparison_3() {
        let dut1 = UbxCID::new(1, 2);
        let dut2 = UbxCID::new(2, 2);
        assert_ne!(dut1, dut2);
    }

    #[test]
    fn assign() {
        let a = UbxCID::new(1, 2);
        let b = a;
        assert_eq!(a, b);
    }
}
