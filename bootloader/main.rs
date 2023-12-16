#![no_main]
#![no_std]
#![feature(const_for)]

use core::{arch::global_asm, panic::PanicInfo, mem::size_of};

// TODO: Rewrite to use static data structures, and try and get the
// linker to build the bootloader in-place.

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

// pub unsafe fn inb(addr: u16) -> u8 {
//     let result: u8;
//     asm!(
//         "inb {addr:x}, {result}",
//         addr = in(reg) addr,
//         result = out(reg_byte) result,
//     );
//     result
// }

// /// ATA PIO mode
// pub mod disk {
//     use crate::inb;

//     const ADDR_BASE: u16 = 0x1F0;
//     const STATUS_OFFSET: u16 = 7;

//     mod status {
//         pub const READY_BIT: u8 = 0b0100_0000;
//         pub const BUSY_BIT: u8 = 0b1000_0000;
//     }

//     /// Wait until disk is ready
//     unsafe fn wait_disk() {
//         let mut ready = false;
//         while !ready {
//             let status = inb(ADDR_BASE + STATUS_OFFSET);
//             let status_ready = status & status::READY_BIT != 0;
//             let status_busy = status & status::BUSY_BIT != 0;
//             ready = status_ready && !status_busy;
//         }
//     }

//     /// Read 512B disk sector at the given address
//     /// addr: 28-bit sector address
//     pub unsafe fn read_sector(addr: usize) {
//         wait_disk(); // Don't read until disk is ready

//         // Assert addr is 28-bits or less
//         assert!(addr & 0xF000_0000 == 0);

//         // Split up the address
//         let addr_0_7 = addr as u8;
//         let addr_8_15 = (addr >> 8) as u8;
//         let addr_16_23 = (addr >> 16) as u8;
//         let addr_24_27 = (addr >> 24) as u8;

//         // Addr bits 24-27 are put in bits 0-4 of
//     }
// }

/// 0 -- Byte granularity -- limit is denominated in 1 Byte blocks
/// 1 -- Page granularity -- limit is denominated in 4KiB blocks
#[derive(Copy, Clone)]
pub enum SegmentDescriptorGranularity {
    Byte = 0,
    Page = 1,
}

/// 0 -- 16-bit protected mode segment
/// 1 -- 32-bit protected mode segment
#[derive(Copy, Clone)]
pub enum SegmentDescriptorSize {
    Protected16 = 0,
    Protected32 = 1,
}

/// Flags setting this segment's granularity, size, and long mode
#[derive(Copy, Clone)]
pub struct SegmentDescriptorFlags {
    /// 0 -- Byte granularity -- limit is denominated in 1 Byte blocks
    /// 1 -- Page granularity -- limit is denominated in 4KiB blocks
    pub granularity: SegmentDescriptorGranularity,
    /// 0 -- 16-bit protected mode segment
    /// 1 -- 32-bit protected mode segment
    pub size: SegmentDescriptorSize,
    /// 0 -- default
    /// 1 -- 64-bit code segment TODO: How does this work in x86?
    pub long_mode: bool,
}

impl SegmentDescriptorFlags {
    pub const fn new() -> Self {
        SegmentDescriptorFlags {
            granularity: SegmentDescriptorGranularity::Page,
            size: SegmentDescriptorSize::Protected32,
            long_mode: false,
        }
    }

    /// Convert to the byte representation that x86 expects.
    /// Top 4 bits are unused, and set to 0
    pub const fn to_bytes(&self) -> u8 {
        // |-------------+------+-----------+----------|
        // |           3 |    2 |         1 |        0 |
        // |-------------+------+-----------+----------|
        // | Granularity | Size | Long mode | Reserved |
        // |-------------+------+-----------+----------|

        let granularity: u8 = self.granularity as u8;
        let size: u8 = self.size as u8;
        let long_mode: u8 = if self.long_mode { 1 } else { 0 };

        (granularity << 3) | (size << 2) | (long_mode << 1)
    }
}

/// The x86 CPU privilege level.
/// On modern systems only Ring 0 (kernel privileges) and Ring 3 (user privileges) are used.
#[derive(Copy, Clone)]
pub enum PrivilegeLevel {
    Kernel = 0,
    User = 3,
}

#[derive(Copy, Clone)]
pub enum DataSegmentDirection {
    GrowsDown = 0,
    GrowsUp = 1,
}

#[derive(Copy, Clone)]
pub enum CodeSegmentRequiredPrivilege {
    Equal = 0,
    LessOrEqual = 1,
}

/// Read is always allowed for data segments
type DataSegmentWritable = bool;
/// Write is never allowed for code segments
type CodeSegmentReadable = bool;

/// Whether a segment is a data or code segment
#[derive(Copy, Clone)]
pub enum SegmentType {
    /// System segment TODO: What is a system segment?
    System,
    /// Non-executable data segment
    Data(DataSegmentDirection, DataSegmentWritable),
    /// Executable code segment
    Code(CodeSegmentRequiredPrivilege, CodeSegmentReadable),
}

#[derive(Copy, Clone)]
pub struct SegmentDescriptorAccessByte {
    /// Privilege level (TODO: required? granted by?) this segment
    pub privilege_level: PrivilegeLevel,
    /// data or code segment?
    pub segment_type: SegmentType,
}

impl SegmentDescriptorAccessByte {
    /// Access byte has a PRESENT bit that must be set to 1 for a valid segment.
    const PRESENT: bool = true;
    /// CPU Sets ACCESSED bit unless you do it first.
    /// If GDT descriptor is stored on a read-only page, that'll page-fault.
    /// So it should always be set to 1.
    const ACCESSED: bool = true;

    pub const fn kernel_code_segment() -> Self {
        Self {
            privilege_level: PrivilegeLevel::Kernel,
            segment_type: SegmentType::Code(CodeSegmentRequiredPrivilege::Equal, true),
        }
    }

    pub const fn user_data_segment() -> Self {
        Self {
            privilege_level: PrivilegeLevel::User,
            segment_type: SegmentType::Data(DataSegmentDirection::GrowsUp, true),
        }
    }

    /// Convert to the byte representation that x86 expects.
    pub const fn to_bytes(&self) -> u8 {
        use SegmentType::*;
        // |-------------+-----------------+-----------------+------------+------------------------+---------------------+----------|
        // |           7 | 6 - 5           |               4 |          3 |                      2 |                   1 |        0 |
        // |-------------+-----------------+-----------------+------------+------------------------+---------------------+----------|
        // | Present bit | Privilege level | Descriptor type | Executable | Direction / Conforming | Readable / Writable | Accessed |
        // |-------------+-----------------+-----------------+------------+------------------------+---------------------+----------|

        let present: u8 = if Self::PRESENT { 1 } else { 0 };
        let accessed: u8 = if Self::ACCESSED { 1 } else { 0 };
        let privilege_level: u8 = self.privilege_level as u8;

        // Descriptor type bit
        // 0 - system segment (Task State Segment, ...)
        // 1 - code/data segment
        let descriptor_type: u8 = match self.segment_type {
            System => 0,
            Data(_, _) | Code(_, _) => 1,
        };

        // Executable bit
        // 0 - non-executable data segment
        // 1 - executable code segment
        let executable: u8 = match self.segment_type {
            System => 0, // TODO: Idk.
            Data(_, _) => 0,
            Code(_, _) => 1,
        };

        // DC - Direction bit / Conforming bit
        // For data segments -- direction bit
        //  0 - segment grows up
        //  1 - segment grows down (base > limit)
        // For code segments -- conforming bit
        //  0 - Code can only be executed from the =privilege level= ring set in =DPL=
        //  1 - Code can be executed from an equal or lower =privilege level=
        let direction_conforming = match self.segment_type {
            System => 0, // TODO: Idk.
            Data(DataSegmentDirection::GrowsUp, _) => 0,
            Data(DataSegmentDirection::GrowsDown, _) => 1,
            Code(CodeSegmentRequiredPrivilege::Equal, _) => 0,
            Code(CodeSegmentRequiredPrivilege::LessOrEqual, _) => 1,
        };

        // RW - Readable bit / Writable bit
        // For data segments -- writable bit
        //  0 - No write access
        //  1 - Write access
        // Read access is always allowed for data segments
        // For code segments -- readable bit
        //  0 - No read access
        //  1 - Read access
        // Write is never allowed for code segments
        let readable_writeable = match self.segment_type {
            System => 0, // TODO: Idk.
            Data(_, false) => 0,
            Data(_, true) => 1,
            Code(_, false) => 0,
            Code(_, true) => 1,
        };

        (present << 7)
            | (privilege_level << 5)
            | (descriptor_type << 4)
            | (executable << 3)
            | (direction_conforming << 2)
            | (readable_writeable << 1)
            | accessed
    }
}

/// Byte representation of a SegmentDescriptor expected by x86
pub type SegmentDescriptorBytes = u64;

#[derive(Copy, Clone)]
pub struct SegmentDescriptor {
    pub base: u32,
    pub limit: u32,
    pub flags: SegmentDescriptorFlags,
    pub access_byte: SegmentDescriptorAccessByte,
}

impl SegmentDescriptor {
    pub const fn new(access_byte: SegmentDescriptorAccessByte) -> Self {
        SegmentDescriptor {
            base: 0,
            limit: 0xFFFF_F,
            flags: SegmentDescriptorFlags::new(),
            access_byte,
        }
    }

    /// Convert the SegmentDescriptor to the byte representation expected by x86
    pub const fn to_bytes(&self) -> SegmentDescriptorBytes {
        // |--------------+---------+---------------+-------------+--------------+-------------+--------------|
        // | 63 - 56      | 55 - 52 | 51 - 48       | 47 - 40     | 39 - 32      | 31 - 16     | 15 - 0       |
        // |--------------+---------+---------------+-------------+--------------+-------------+--------------|
        // | Base 31 - 24 | Flags   | Limit 19 - 16 | Access Byte | Base 23 - 16 | Base 15 - 0 | Limit 15 - 0 |
        // |--------------+---------+---------------+-------------+--------------+-------------+--------------|

        // Get each cell's value pushed to the right
        // Then push them all left to their final position and OR them together
        let base_31_24: u64 = (self.base >> 24) as u64;
        let base_23_16: u64 = ((self.base >> 16) & 0xFF) as u64;
        let base_15_0: u64 = (self.base & 0xFFFF) as u64;

        let limit_19_16: u64 = ((self.limit >> 16) & 0xF) as u64;
        let limit_15_0: u64 = (self.limit & 0xFF) as u64;

        let flags: u64 = self.flags.to_bytes() as u64;
        let access_byte: u64 = self.access_byte.to_bytes() as u64;

        (base_31_24 << 56) | (flags << 52) | (limit_19_16 << 48) | (access_byte << 40) | (base_23_16 << 32) | (base_15_0 << 16) | limit_15_0
    }
}

type GdtBytes = [SegmentDescriptorBytes; 3];

pub struct Gdt {
    segments: [SegmentDescriptor; 2], // TODO: This should be variable...
}

impl Gdt {
    pub const fn new() -> Self {
        Self {
            segments: [
                SegmentDescriptor::new(SegmentDescriptorAccessByte::kernel_code_segment()),
                SegmentDescriptor::new(SegmentDescriptorAccessByte::user_data_segment()),
            ]
        }
    }

    pub const fn to_bytes(&self) -> GdtBytes {
        [
            0, // The first segment needs to be NULL
            self.segments[0].to_bytes(),
            self.segments[1].to_bytes(),
        ]
    }
}

#[repr(C)]
struct GdtDescriptor {
    size: u16,
    pointer: &'static GdtBytes,
}

impl GdtDescriptor {
    pub const fn new(size: u16, pointer: &'static GdtBytes) -> Self {
        Self {
            size,
            pointer,
        }
    }
}

// ==== Static objects for the Bootloader ====

#[no_mangle]
static FIND_ME: u32 = 1234;

#[no_mangle]
static GDT_STRUCT_AAAA: Gdt = Gdt::new();

#[no_mangle]
static GDT_AAAA: GdtBytes = GDT_STRUCT_AAAA.to_bytes();

#[no_mangle]
static GDT_DESCRIPTOR_AAAA: GdtDescriptor = GdtDescriptor::new(
    (size_of::<GdtBytes>() - 1) as u16,
    &GDT_AAAA,
);

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
    ljmp $(1 << 3), $start32 # Code segment in GDT entry 1, calc offset

.code32
start32:
    movw $(2 << 3), %ax # Data segment in entry 2, calculate offset
    movw    %ax, %ds                # -> DS: Data Segment
    movw    %ax, %es                # -> ES: Extra Segment
    movw    %ax, %ss                # -> SS: Stack Segment
    movw    $0, %ax                 # Zero segments not ready for use
    movw    %ax, %fs                # -> FS
    movw    %ax, %gs                # -> GS

    # Set up the stack pointer and call into Rust
    movl $_start, %esp
    call boot_main

    # Trigger Bochs breakpoint if we erroneously return from boot_main
    movw    $0x8a00, %ax            # 0x8a00 -> port 0x8a00
    movw    %ax, %dx
    outw    %ax, %dx
    movw    $0x8ae0, %ax            # 0x8ae0 -> port 0x8a00
    outw    %ax, %dx

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

#[no_mangle]
pub extern "C" fn boot_main() -> u32 {
    return FIND_ME;
}
