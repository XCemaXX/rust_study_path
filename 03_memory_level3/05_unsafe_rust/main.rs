
#[repr(C)]
union MyUnion {
    i: u8,
    b: bool,
}

static mut COUNTER: u32 = 0;

fn add_to_counter(inc: u32) {
    unsafe { //cannot change safely because of multithreading
        COUNTER += inc;
    }
}

extern "C" { // foreign function interface, FFI
    fn abs(input: i32) -> i32;
}

fn count_chars(s: &str) -> usize {
    s.chars().count()
}

unsafe fn swap(a: *mut u8, b: *mut u8) {
    let temp = *a;
    *a = *b;
    *b = temp;
}

fn main() {
    let mut s = "Hello".to_string();
    let r1 = &mut s as *mut String;
    let r2 = r1 as *const String;
    unsafe {
        println!("r1 is: {}", *r1); //cannot deref without unsafe
        *r1 = String::from("unsafe");
        println!("r2 is: {}", *r2);
    }

    add_to_counter(23);

    unsafe { // in one thread actually safe
        println!("COUNTER: {COUNTER}");
    }

    let u = MyUnion{ i: 42 };
    println!("int: {:?}", unsafe { u.i });
    println!("int: {:?}", unsafe { u.b }); // undefined behavior

    let emojis = "👍∈🌏";
    unsafe {
        println!("emoji: {}", emojis.get_unchecked(0..4));
        println!("emoji: {}", emojis.get_unchecked(4..7));
        println!("emoji: {}", emojis.get_unchecked(7..11));
    }
    // unchecked functions usually unsafe
    println!("Symbols count {}", count_chars(unsafe { emojis.get_unchecked(0..7) }));

    unsafe {
        println!("Abs of -3 according to C is: {}", abs(-3));
    }

    let mut a = 5;
    let mut b = 7;
    unsafe {
        swap(&mut a, &mut b);
    }
    println!("a = {}, b = {}", a, b);
}