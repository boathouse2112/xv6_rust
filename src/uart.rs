// qemu-system-x86_64 with the -nographic option reads from the COM1 serial port.
// Intel 8250 serial port (UART).

use core::arch::asm;
use core::ops::BitAnd;
use crate::x86;

const COM_1: u16 = 0x3f8;
const DATA: u16 = COM_1;
const INTERRUPT_ENABLE: u16 = COM_1 + 1;
const DIVISOR_MSB: u16 = COM_1 + 1;
const FIFO_CONTROL: u16 = COM_1 + 2;
const INTERRUPT_IDENTIFICATION: u16 = COM_1 + 2;
const LINE_CONTROL: u16 = COM_1 + 3;
const MODEM_CONTROL: u16 = COM_1 + 4;
const LINE_STATUS: u16 = COM_1 + 5;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
struct LineStatus(u8);

impl LineStatus {
    fn new(data: u8) -> Self {
        LineStatus(data)
    }

    fn data_ready(self) -> bool {
        (self.0 & 0b0000_0001) != 0
    }

    fn overrun_error(self) -> bool {
        (self.0 & 0b0000_0010) != 0
    }

    fn parity_error(self) -> bool {
        (self.0 & 0b0000_0100) != 0
    }

    fn framing_error(self) -> bool {
        (self.0 & 0b0000_1000) != 0
    }

    fn break_indicator(self) -> bool {
        (self.0 & 0b0001_0000) != 0
    }

    fn transmitter_holding_register_empty(self) -> bool {
        (self.0 & 0b0010_0000) != 0
    }

    fn transmitter_empty(self) -> bool {
        (self.0 & 0b0100_0000) != 0
    }

    fn impending_error(self) -> bool {
        (self.0 & 0b1000_0000) != 0
    }
}

pub struct SerialPort;

impl SerialPort {
    pub fn init() -> Option<Self> {
        // Initialize the UART
        unsafe {
            // Turn off FIFO
            x86::port_out_byte(FIFO_CONTROL, 0);

            // 9600 baud, 8 data bits, 1 stop bit, parity off.
            x86::port_out_byte(LINE_CONTROL, 0x80);    // Unlock divisor
            x86::port_out_byte(DATA, (115200 as u32 / 9600 as u32) as u8);
            x86::port_out_byte(DIVISOR_MSB, 0);
            x86::port_out_byte(LINE_CONTROL, 0x03);    // Lock divisor, 8 data bits.
            x86::port_out_byte(MODEM_CONTROL, 0);
            x86::port_out_byte(INTERRUPT_ENABLE, 0x01);    // Enable receive interrupts.
        }

        let sp = SerialPort {};
        let status = sp.line_status();
        if status.0 == 0xff {
            return None;
        }

        // Acknowledge pre-existing interrupt conditions;
        // enable interrupts.
        unsafe {
            x86::port_in_byte(FIFO_CONTROL);
            x86::port_in_byte(DATA);
            // TODO: enable interrupts
            // ioapicenable(IRQ_COM1, 0);
        }

        // Announce that we're here.
        for byte in "xv6...\n".bytes() {
            sp.write_u8(byte);
        }

        Some(sp)
    }

    pub fn read_u8(&self) -> Result<u8, &str> {
        if self.line_status().data_ready() {
            Ok(unsafe {x86::port_in_byte(DATA)})
        } else {
            Err("No")
        }
    }

    pub fn write_u8(&self, data: u8) {
        // Spin until ready to write
        while !self.line_status().transmitter_holding_register_empty() {}

        unsafe { x86::port_out_byte(DATA, data) }
    }

    fn ready_to_read(&self) -> bool {
        self.line_status().transmitter_holding_register_empty()
    }

    fn line_status(&self) -> LineStatus {
        LineStatus::new(unsafe { x86::port_in_byte(LINE_STATUS) })
    }
}