#![no_std]
#![no_main]

pub mod clock_sync;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}