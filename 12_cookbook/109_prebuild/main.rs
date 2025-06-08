use std::ffi::CString;
use std::os::raw::c_char;

unsafe extern "C" {
    fn hello(name: *const c_char);
    fn add(x: i32, y: i32) -> i32;
    fn multiply(x: i32, y: i32) -> i32;
    fn print_app_info();
}

fn main() {
    let sum = unsafe { add(42, 23) };
    println!("42 + 23 = {}", sum);
    let mul = unsafe { multiply(42, 23) };
    println!("42 * 23 = {}", mul);
    let name = CString::new("User").unwrap();
    unsafe { hello(name.as_ptr()) };
    unsafe {
        print_app_info();
    }
}
