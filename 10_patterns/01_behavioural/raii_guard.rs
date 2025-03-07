use std::ops::Deref;

#[derive(Debug)]
struct Resource;

trait Hello {
    fn hello(&self);
}

impl Hello for Resource {
    fn hello(&self) {
        println!("Hello {:?}", &self);
    }
}

struct Guard<'a, T: 'a> {
    data: &'a T,
}

impl<'a, T> Guard<'a, T> {
    pub fn new(data: &'a T) -> Self {
        println!("Entering guard");
        Self { data }
    }
}

impl<'a, T> Drop for Guard<'a, T> {
    fn drop(&mut self) {
        println!("Kill data");
    }
}

impl<'a, T> Deref for Guard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

fn main() {
    println!("start");
    {
        let guarded: Guard<'_, Resource> = Guard::new(&Resource);
        guarded.hello();
    }
    println!("end");
}
