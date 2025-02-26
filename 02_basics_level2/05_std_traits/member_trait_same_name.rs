struct CallBack {
    callback: Box<dyn Fn()>,
}

impl CallBack {
    fn callback(&self) {
        println!("trait");
    }
}

 
fn main() {
    let a = CallBack{ callback: Box::new(|| println!("member")) };
    (a.callback)(); // call member
    a.callback(); // call from trait
}