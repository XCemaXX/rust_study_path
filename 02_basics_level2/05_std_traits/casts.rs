

fn from_example() {
    let s = String::from("Hello");
    let addr = std::net::Ipv4Addr::from([127, 0, 0, 1]);
    let one = i16::from(true);
    let bigger = i32::from(123_i16);
    println!("{s}, {addr}, {one}, {bigger}");
}

fn into_example() {
    let s: String = "Hello".into();
    let addr: std::net::Ipv4Addr = [127, 0, 0, 1].into();
    let one: i16 = true.into();
    let bigger: i32 = 123_i16.into();
    println!("{s}, {addr}, {one}, {bigger}");
}

fn cast_example() {
    let value: i64 = 1000;
    println!("as u16: {}", value as u16);
    println!("as u32: {}", value as u32);
    println!("as i32: {}", value as i32);
}

fn main() {
    from_example();
    into_example();
    cast_example();
    // there is TryFrom, TryInto
}