

fn unpack_tuple(point: (i32, i32)) {
    match point {
        (0, 0) => println!("BULLSEYE"),
        (0, _) => println!("on Y"),
        (_, 0) => println!("on X"),
        (x, _) if x < 0 => println!("left from Y"),
        (_, y) if y < 0 => println!("down X"),
        _ => println!("other"),
    }
}

//fn unpack_array<const SIZE: usize, T>(arr: [T; SIZE]) {
//fn unpack_array<const SIZE: usize>(arr: [i32; SIZE]) {
fn unpack_array(arr: [i32; 3]) {
    match arr {
        [0, y, z] => println!("first is 0, y = {y} и z = {z}"),
        [1, ..] => println!("first is 1"),
        _ => println!("other"),
    }
}

struct Person {
    name: String,
    age: u32
}

fn unpack_struct(man: Person) {
    match man {
        Person{ ref name, age } if name == "Kate" => println!("Dear Kate. Age: {age}"), //work around String
        Person{ age: 15, name: alias } => println!("Name: {alias} with 15 age"),
        Person{ age, .. } => println!("Unknown. Age: {age}"),
    }
}

fn main() {
    unpack_tuple((0, -1));
    unpack_tuple((1, -1));
    unpack_tuple((1, 1));

    let arr = [0, -2, 3];
    println!("{arr:?}");
    unpack_array(arr);
    unpack_array([1, -2, 3]);
    unpack_array([2, -2, 3]);

    unpack_struct( Person{name: "Kate".to_string(), age: 15});
    unpack_struct( Person{name: "Tom".to_string(), age: 15});
    unpack_struct( Person{name: "John".to_string(), age: 25});
}