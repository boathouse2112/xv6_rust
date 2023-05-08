// qemu-system-x86_64 with the -nographic option reads from the COM1 serial port.
// xv6 src says it's a 8250 UART,
// but qemu docs say it's 16550 UART. I don't know the difference.

// See:
// https://github.com/rust-osdev/x86_64/blob/22066fa1a1d6efb43ca8bb64db30c208322c32c5/testing/src/serial.rs#L4

use core::arch::asm;
use core::fmt;
use core::fmt::Write;
use core::ops::BitAnd;
use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;
use crate::x86;

const SERIAL_PORT_ADDRESS: u16 = 0x3f8;
const DATA: u16 = SERIAL_PORT_ADDRESS;
const INTERRUPT_ENABLE: u16 = SERIAL_PORT_ADDRESS + 1;
const DIVISOR_MSB: u16 = SERIAL_PORT_ADDRESS + 1;
const FIFO_CONTROL: u16 = SERIAL_PORT_ADDRESS + 2;
const INTERRUPT_IDENTIFICATION: u16 = SERIAL_PORT_ADDRESS + 2;
const LINE_CONTROL: u16 = SERIAL_PORT_ADDRESS + 3;
const MODEM_CONTROL: u16 = SERIAL_PORT_ADDRESS + 4;
const LINE_STATUS: u16 = SERIAL_PORT_ADDRESS + 5;

// Global serial port
lazy_static! {
    pub static ref SERIAL_PORT: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(SERIAL_PORT_ADDRESS) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

// Print function, print! and println! macros using the global serial port

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    SERIAL_PORT.lock().write_fmt(args).expect("Printing to serial failed");
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::uart::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
