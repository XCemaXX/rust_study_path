// cargo run -p methods --bin traits

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

fn main() {
    let parrot = Parrot{name: String::from("Pavel"), age: 2};
    let duck = Duck{color: String::from("Grey")};
    parrot.greet();
    duck.greet();
    // ######
    let pets: Vec<Box<dyn Pet>> = vec! [
        Box::new(Parrot{name: String::from("Pavel"), age: 2}),
        Box::new(Duck{color: String::from("Grey")}),
    ];
    for pet in pets {
        pet.greet();
    }

    println!("Struct size. Parrot: {}, Duck: {}", std::mem::size_of::<Parrot>(), std::mem::size_of::<Duck>());
    println!("Link size. Parrot: {}, Duck: {}", std::mem::size_of::<&Parrot>(), std::mem::size_of::<&Duck>());
    println!("Trait fat pointer size: {}", std::mem::size_of::<&dyn Pet>());
    println!("Box trait size: {}", std::mem::size_of::<Box<dyn Pet>>());
}