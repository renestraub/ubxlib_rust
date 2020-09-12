use std::fmt;

// TODO: automatic debug formatter or ..
// #[derive(Debug)]
pub struct UbxCID {
    cls: u8,
    id: u8,
}

impl UbxCID {
    pub fn new(cls: u8, id: u8) -> Self {
        Self { cls: cls, id: id }
    }

    pub fn cls(&self) -> u8 {
        self.cls
    }

    pub fn id(&self) -> u8 {
        self.id
    }
}

impl PartialEq for UbxCID {
    fn eq(&self, other: &Self) -> bool {
        self.cls == other.cls && self.id == other.id
    }
}


impl fmt::Debug for UbxCID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CID")
         .field("cls", &self.cls)   // TODO: &format_args!("0x{:02X}"), 
         .field("id", &self.id)
         .finish()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction() {
        let dut = UbxCID::new(1, 2);
        assert_eq!(format!("{:?}", dut), "CID { cls: 1, id: 2 }");

        let dut = UbxCID::new(0x13, 0x00);
        assert_eq!(format!("{:?}", dut), "CID { cls: 19, id: 0 }");
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
}
