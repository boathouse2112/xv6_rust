// x86 instruction wrappers

use core::arch::asm;

pub unsafe fn port_out_byte(port: u16, data: u8) {
    asm!(
        "out dx, al",
        in("dx") port,
        in("al") data
    );
}

pub unsafe fn port_in_byte(port: u16) -> u8 {
    let data: u8;
    asm!("in al, dx", out("al") data, in("dx") port);
    data
}
