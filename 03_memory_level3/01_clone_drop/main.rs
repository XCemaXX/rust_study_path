#[derive(Clone, Debug)]
struct Clonnable(i32, i32, String);

#[derive(Debug)]
struct Droppable {
    name: &'static str
}

impl Drop for Droppable {
    fn drop(&mut self) {
        println!("Killed {}", self.name);
    }
}

fn main() {
    let p1 = Clonnable(3, 4, String::from("asdf"));
    let p2 = p1.clone();
    let p3 = Droppable{name: "a"};
    {
        let p3 = Droppable{name: "b"};
    }
    drop(p3);
    println!("p1: {p1:?}");
    println!("p2: {p2:?}");
}