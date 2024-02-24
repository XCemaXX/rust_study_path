#[derive(Debug, Default)]
struct Foo {
    x: u32,
    y: String,
    z: Implemented,
}

#[derive(Debug)]
struct Implemented(String);

impl Default for Implemented {
    fn default() -> Self {
        Self("Cool string".into())
    }
}

fn main() {
    let foo = Foo::default();
    println!("{foo:#?}"); // :#? - pretty printing

    let foo2 = Foo{y: "Y String".into(), ..Foo::default()};
    println!("{foo2:?}");

    let none: Option<Foo> = None;
    println!("{:?}", none.unwrap_or_default());
}