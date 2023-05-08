#![no_main]
#![no_std]

mod uart;
mod x86;
mod proc;

use core::panic::PanicInfo;
use crate::uart::{SERIAL_PORT};

#[no_mangle]
pub extern "C" fn _start() -> ! {

    lazy_static::initialize(&uart::SERIAL_PORT);
    println!("Hello, UART!");

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
