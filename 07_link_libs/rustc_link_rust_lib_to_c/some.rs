#![no_std]

#[no_mangle]
pub extern "C" 
fn add(x: i32, y: i32) -> i32 {
    x + y
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}