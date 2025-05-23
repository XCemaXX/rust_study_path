// cargo run -p methods --bin traits
#![allow(dead_code)]
struct Duck {
    color: String,
}

struct Parrot {
    name: String,
    age: u32,
}

trait Pet { // like interface
    fn talk (&self) -> String;
    fn greet (&self) {
        println!("Hello! What is your name?\n {}", self.talk());
    }
}

impl Pet for Parrot {
    fn talk (&self) -> String {
        format!("My name is {}", self.name)
    }
}

impl Pet for Duck {
    fn talk (&self) -> String {
        String::from("Quack")
    }
}

trait SuperPet {
    fn talk (&self) -> String;
}

impl SuperPet for Parrot {
    fn talk (&self) -> String {
        format!("Hello! Hello! Hello!")
    }
}

fn main() {
    let parrot = Parrot{name: String::from("Pavel"), age: 2};
    let duck = Duck{color: String::from("Grey")};
    parrot.greet();
    duck.greet();
    let parrot: &dyn Pet = &parrot;
    let duck: &dyn Pet = &duck;
    let pets = vec![parrot, duck]; // only links, doesn't own
    for pet in pets {
        pet.greet();
    }
    // ######
    let pets: Vec<Box<dyn Pet>> = vec! [
        Box::new(Parrot{name: String::from("Pavel"), age: 2}),
        Box::new(Duck{color: String::from("Grey")}),
    ]; // owns
    for pet in pets {
        pet.greet();
    }

    println!("Struct size. Parrot: {}, Duck: {}", std::mem::size_of::<Parrot>(), std::mem::size_of::<Duck>());
    println!("Link size. Parrot: {}, Duck: {}", std::mem::size_of::<&Parrot>(), std::mem::size_of::<&Duck>());
    println!("Trait fat pointer size: {}", std::mem::size_of::<&dyn Pet>());
    println!("Box trait size: {}", std::mem::size_of::<Box<dyn Pet>>());

    // different traits with same name. Need to specify
    let parrot = Parrot{name: String::from("Igor"), age: 2};
    println!("Supper parrot: {}", SuperPet::talk(&parrot)); 
    println!("Parrot: {}", Pet::talk(&parrot));    
}