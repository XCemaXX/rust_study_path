use bitflags::{Flags, bitflags};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct Fields: u32 {
        const A = 0b1;
        const B = 0b10;
        const C = 0b100;
        const ABC = Self::A.bits() | Self::B.bits() | Self::C.bits();
    }
}

fn main() {
    let e1 = Fields::A | Fields::C;
    let e2 = Fields::B | Fields::C;

    assert_eq!((e1 | e2), Fields::ABC);
    assert_eq!((e1 & e2), Fields::C);
    assert_eq!((e1 - e2), Fields::A);
    assert_eq!(!e2, Fields::A);

    let mut flags = Fields::ABC;
    println!("{:?}", flags);
    flags.clear();
    println!("{:?}", flags);
    println!("{:?}", Fields::B);
    println!("{:?}", Fields::A | Fields::B);
}
