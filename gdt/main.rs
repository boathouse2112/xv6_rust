#![no_main]
#![no_std]

use core::{arch::global_asm, panic::PanicInfo};

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Gdt {
    name: u16,
    start: u16,
    len: u16,
}

#[no_mangle]
pub static MY_GDT: Gdt = Gdt {
    name: 0xAB,
    start: 0xAA,
    len: 0x08,
};

#[no_mangle]
fn main() -> Gdt {
    let a = MY_GDT;
    return a;
}
