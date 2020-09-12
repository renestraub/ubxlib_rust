mod checksum;
mod cid;
mod frame;
mod parser;


// const MAX_POINTS: u32 = 100_000;

fn test(x: isize) {
    println!("test fn {}", x);
}

fn by_two(value: isize) -> isize {
    return 2*value;
}



fn main() {
    println!("Hello, world!");
    println!("Rene is here");

    let i = 0b1111_0000;
    println!("Rene is here {}", i);

    for x in 0..10 {
        test(x);
        
        let d = by_two(x);
        println!("twice {}", d);
    }

    let mut chksum = checksum::Checksum::new();
    chksum.value();

    chksum.reset();
    let (a, b) = chksum.value();
    println!("Parser::reset(), checksum is {:X} {:X}", a, b);

    chksum.add(0x01);
    chksum.add(0x01);
    chksum.value();

    let matches = chksum.matches(2, 3);
    println!("match1 {}", matches);
    let matches = chksum.matches(2, 1);
    println!("match2 {}", matches);

    let _p = parser::Parser::new();
    // p.test();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two() {
        let xyz = 123isize;
        assert_eq!(by_two(xyz), 2*123);
    }
}

