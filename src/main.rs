#![no_main]
#![no_std]

use core::panic::PanicInfo;

static HELLO: &[u8] = b"Hello World";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Memory location of VGA mode buffer
    let vga_buffer = 0xb8000 as *mut u8;

    // Write to the VGA mode buffer
    // Lower byte is character ASCII code-point
    // Higher (second?) byte is color data
    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }

    loop {}
}

#[panic_handler]
fn panic(_panic: &PanicInfo) -> ! {
    loop {}
}
