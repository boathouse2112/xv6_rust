#![no_main]
#![no_std]

// mod proc;
// mod uart;

// use crate::uart::SERIAL_PORT;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // lazy_static::initialize(&uart::SERIAL_PORT);
    // println!("Hello, UART!");

    loop {}
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    // println!("{}", info);
    loop {}
}
