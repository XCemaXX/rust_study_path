use linked_list_allocator::LockedHeap;

use crate::memmap::MmapOptions;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    crate::println!("{}", info);
    core::intrinsics::abort();
}

#[lang = "eh_personality"]
fn eh_personality() {}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
unsafe extern "C" fn _Unwind_Resume() {}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

const HEAP_SIZE_MB: u64 = 128;

/// Initialize a global allocator that only uses `mmap`, with a fixed heap size.
///
/// # Safety
/// Calling this too late (or not at all) and doing a heap allocation will
/// fail. The `mmap` syscall can also fail, which would be disastrous.
pub unsafe fn init_allocator() {
    let heap_size = HEAP_SIZE_MB * 1024 * 1024;
    let heap_bottom = MmapOptions::new(heap_size).map().unwrap();
    unsafe {
        ALLOCATOR.lock().init(heap_bottom as _, heap_size as _);
    }
}
