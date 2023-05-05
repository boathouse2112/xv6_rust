#![no_main]
#![no_std]

mod vga_buffer;
mod uart;
mod x86;

use core::panic::PanicInfo;
use crate::uart::SerialPort;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // println!("no way never{}", "!");
    // panic!("It broke");

    let a = SerialPort::init();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
