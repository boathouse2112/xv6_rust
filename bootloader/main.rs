#![no_main]
#![no_std]

use core::{arch::global_asm, panic::PanicInfo};

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub static ASDF: u16 = 0xABCD;

// Define our _start entry function in assembly.
// This will be copied to 0x00 of the bootloader binary
global_asm!(
    r#"
.code16                                # Produce 16-bit machine code
.global _start                         # Make _start function globally-visible
_start:
    cli                                # Disable interrupts

    # Zero out segment registers
    movw     $0, %ax
    movw     %ax, %ds                  # movw with segment registers only seems to work
    movw     %ax, %es                  #     with a register src. $0, %ds doesn't work.
    movw     %ax, %ss

    # Enable address line A20
set_a20_controller_port:
    inb      $0x64, %al
    testb    $0b0010, %al              # bit 1 set => input buffer full
    jnz      set_a20_controller_port   # Wait for empty input buffer

    movb     $0xD1, %al
    outb     %al, $0x64

set_a20_data_port:
    inb      $0x64, %al
    testb    0b0010, %al
    jnz set_a20_data_port

    movb     $0b11011111, %al         # Bit 1 sets A20 to work. The other bits are the same as xv6. Idk.
    outb     %al, $0x60

    # Switch to protected mode
    lgdt gdt_descriptor
    movl    %cr0, %eax
    orl     $1, %eax
    movl    %eax, %cr0
    ljmp $0b1000, $start32

.code32
start32:
    movw $0b1000, %ax

gdt_descriptor:
    .word (gdt_end - gdt - 1)         # size -- size of GDT table - 1
    .long gdt
gdt:
    # Null segment -- 8 bytes
    .skip 8

    # Code segment -- base = 0x0, limit = 0xFFFF_FFFF, Access = 0x9A
    .word 0xFFFF                     # limit_0_15
    .word 0x0000 # base_0_15
    .byte 0x00 # base_16_23
    .byte 0x9A # access -- It's in the wiki. TODO: explanation
    .byte 0xCF
    .byte 0x00 # base_24_31

    # Data segment -- base = 0x0, limit = 0xFFFF_FFFF, Access = 0x92
    .word 0xFFFF
    .word 0x0000
    .byte 0x00
    .byte 0x92
    .byte 0xCF
    .byte 0x00
gdt_end:
"#,
    options(att_syntax)
);

//     # Switch from real to protected mode
//     lgdt     gdtdesc
//     movl     %cr0, %eax
//     orl      $0b00000001, %eax
//     movl     %eax, %cr0

//     # ???
//     ljmp     $(1<<3), $start32
// .code32
// start32:
//     # Set up the protected-mode data segment registers
//     movw    $(1<<3), %ax    # Our data segment selector
//     movw    %ax, %ds                # -> DS: Data Segment
//     movw    %ax, %es                # -> ES: Extra Segment
//     movw    %ax, %ss                # -> SS: Stack Segment
//     movw    $0, %ax                 # Zero segments not ready for use
//     movw    %ax, %fs                # -> FS
//     movw    %ax, %gs                # -> GS

//     # Set up the stack pointer and call into C.
//     movl    $_start, %esp
//     # call    bootmain

//     # If bootmain returns (it shouldn't), trigger a Bochs
//     # breakpoint if running under Bochs, then loop.
//     movw    $0x8a00, %ax            # 0x8a00 -> port 0x8a00
//     movw    %ax, %dx
//     outw    %ax, %dx
//     movw    $0x8ae0, %ax            # 0x8ae0 -> port 0x8a00
//     outw    %ax, %dx

// .p2align 2                            # force 4 byte alignment
// gdt:
//     # Null segment
//     .fill    8, 1, 0

//     # Code segment
//     .word (((0xFFFFFFFF) >> 12) & 0xffff), ((0) & 0xffff)
//     .byte (((0) >> 16) & 0xff), (0x90 | (0x08|0x02)), (0xC0 | (((0xFFFFFFFF) >> 28) & 0xf)), (((0) >> 24) & 0xff)

//     # Data segment
//     .word (((0xFFFFFFFF) >> 12) & 0xffff), ((0) & 0xffff)
//     .byte (((0) >> 16) & 0xff), (0x90 | (0x08|0x02)), (0xC0 | (((0xFFFFFFFF) >> 28) & 0xf)), (((0) >> 24) & 0xff)

// gdtdesc:
//     .word   (gdtdesc - gdt - 1)             # sizeof(gdt) - 1
//     .long   gdt                             # address gdt

// #[no_mangle]
// pub extern "C" fn _start() {}
#[no_mangle]
fn main() -> u16 {
    let a = ASDF;
    return a;
}
