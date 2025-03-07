use std::fmt::Display;

struct Meter(f32);
struct Mile(f32);

impl Display for Meter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} meters", self.0)
    }
}

impl Display for Mile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} miles", self.0)
    }
}

impl From<Mile> for Meter {
    fn from(value: Mile) -> Self {
        Meter(value.0 * 1609.34)
    }
}

impl From<Meter> for Mile {
    fn from(value: Meter) -> Self {
        Mile(value.0 / 1609.34)
    }
}

fn main() {
    let mile = Mile(2.0);
    print!("{} - ", mile);
    let meter: Meter = mile.into();
    println!("{}", meter);

    let meter = Meter(5000.0);
    print!("{} - ", meter);
    let mile = Mile::from(meter);
    println!("{}", mile);
}
