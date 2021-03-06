//! Driver for a MMIO UART Interface

use crate::*;
use crate::fs::devfs::tty::TeletypeDevice;
use crate::process::PID;
use crate::utils::ByteRingBuffer;

use super::generic::ByteInterface;
use super::mmio;
use super::generic;

use crate::fs::devfs::tty::TeletypeSettings;

use crate::fs::devfs::tty_consts::*;

/// Safety: if the base address is a vaild base address for a UART driver,
/// this will perform as expected.
unsafe fn init(base: usize)
{
    // Set word length 0b11 will set an 8 bit word length
    let lcr = 0b0000011;
    mmio::write_offset::<u8>(base, 3, lcr);

    // Enable the recieve buffer interrupts
    mmio::write_offset::<u8>(base, 1, 0b0000001);

    // Divisor calculation
    let divisor = 592u16;
    let divisor_low = divisor & 0xFF;
    let divisor_high = (divisor & 0xFF00) >> 8;

    // Open the divisor latch
    mmio::write_offset::<u8>(base, 3, lcr | 1 << 7);

    mmio::write_offset::<u8>(base, 0, divisor_low as u8);
    mmio::write_offset::<u8>(base, 1, divisor_high as u8);

    // Close the divisor latch
    mmio::write_offset::<u8>(base, 3, lcr);
}

/// Read a byte from the UART port
/// Safety: if the base address is a vaild base address for an initialized UART
/// driver, this will perform as expected.
unsafe fn read_byte(base: usize) -> Option<u8>
{
    // Check if there is pending data
    if mmio::read_offset::<u8>(base, 5) & 1 == 0
    {
        None
    }
    else
    {
        Some(mmio::read_offset::<u8>(base, 0))
    }
}

/// Write a byte to the UART port
/// Safety: if the base address is a vaild base address for an initialized UART
/// driver, this will perform as expected.
unsafe fn write_byte(base: usize, data: u8)
{
    mmio::write_offset::<u8>(base, 0, data);
}

/// MMIO UART Driver
pub struct UARTDriver
{
    base: usize,
    input_buffer: ByteRingBuffer,
    line_buffer: ByteRingBuffer,
    terminal_settings: crate::fs::devfs::tty::TeletypeSettings,
    fgpgid: PID,
    tty_paused: bool,
    tty_preserve_next: bool
}

impl UARTDriver
{
    /// Create a new UART Driver
    /// Safety: if the base address is a vaild base address for a UART driver,
    /// this will perform as expected.
    pub const unsafe fn new(base: usize) -> Self
    {
        Self
        {
            base,
            input_buffer: ByteRingBuffer::new(),
            line_buffer: ByteRingBuffer::new(),
            terminal_settings: crate::fs::devfs::tty::TeletypeSettings::new(),
            fgpgid: 0,
            tty_paused: false,
            tty_preserve_next: false
        }
    }

    /// Initialize the UART Driver
    pub fn init(&mut self)
    {
        // Safety: Assuming the safety from the `new` implementation is
        // satisfied, this is safe
        unsafe 
        {
            init(self.base);
        }
    }

    /// Notify of a byte being recieved by the device
    pub fn notify_recieve(&mut self)
    {
        // Safety: Assuming the safety from the `new` implementation is
        // satisfied, this is safe
        if let Some(byte) = unsafe { read_byte(self.base) }
        {
            self.tty_push_byte(byte);
        }
    }
}

impl generic::ByteInterface for UARTDriver
{
    /// Read a byte from the UART
    fn read_byte(&mut self) -> Option<u8>
    {
        if self.get_tty_settings().local_flags & ICANON > 0
        {
            self.line_buffer.dequeue_byte()
        }
        else
        {
            self.input_buffer.dequeue_byte()
        }

        // unsafe { read_byte(self.base) }
    }

    /// Write a byte to the UART
    fn write_byte(&mut self, data: u8)
    {
        // Safety: Assuming the safety from the `new` implementation is
        // satisfied, this is safe
        unsafe 
        {
            write_byte(self.base, data);
        }   
    }
}

// Implement the core::fmt::Write trait for the UART Driver
impl core::fmt::Write for UARTDriver
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result
    {
        for byte in s.as_bytes()
        {
            self.write_byte(*byte);
        }

        Ok(())
    }
}

impl crate::fs::devfs::tty::TeletypeDevice for UARTDriver
{
    fn tty_read_byte(&mut self) -> Option<u8>
    {
        self.read_byte()
    }

    fn tty_write_byte(&mut self, byte: u8)
    {
        if self.terminal_settings.output_flags & OPOST > 0 && byte == 0xA
        {
            self.write_byte(byte);
            self.write_byte(0x0D);
        }
        else
        {
            self.write_byte(byte);
        }
    }

    fn tty_close(&mut self)
    {
        // Nothing to do here, this tty can't be closed
    }

    fn tty_push_byte(&mut self, byte: u8)
    {
        let settings = self.get_tty_settings();

        if self.handle_input(byte)
        {
            return;
        }

        if byte == 0xD && settings.input_flags & ICRNL > 0
        {
            self.input_buffer.enqueue_byte(0xA);
        }
        else
        {
            self.input_buffer.enqueue_byte(byte);
        }

        if settings.local_flags & ICANON > 0
        {
            if byte == 0xD
            {
                while let Some(b) = self.input_buffer.dequeue_byte()
                {
                    self.line_buffer.enqueue_byte(b);
                }
            }
            else if byte == 0x4
            {
                while let Some(b) = self.input_buffer.dequeue_byte()
                {
                    self.line_buffer.enqueue_byte(b);
                }
            }
        }
    }

    fn tty_pop_byte(&mut self) -> Option<u8>
    {
        // Not needed for UART, as whenever a byte is written to the tty, it
        // immediately moves that byte on to the UART port
        unimplemented!()
    }

    fn get_tty_settings(&self) -> TeletypeSettings
    {
        self.terminal_settings
    }

    fn set_tty_settings(&mut self, settings: TeletypeSettings)
    {
        self.terminal_settings = settings;
    }

    fn bytes_to_backaspace(&self) -> bool
    {
        !self.input_buffer.is_empty()
    }

    fn backspace(&mut self) -> bool
    {
        self.input_buffer.pop_byte().is_some()
    }

    fn bytes_available(&self) -> bool
    {
        if self.get_tty_settings().local_flags & ICANON > 0
        {
            !self.line_buffer.is_empty()
        }
        else
        {
            !self.input_buffer.is_empty()
        }
    }

    fn flush_tty(&mut self)
    {
        while let Some(_) = self.input_buffer.pop_byte() {}
        while let Some(_) = self.line_buffer.pop_byte() {}
    }

    fn get_foreground_process_group(&self) -> PID
    {
        self.fgpgid
    }

    fn set_foreground_process_group(&mut self, pgid: PID)
    {
        self.fgpgid = pgid;
    }

    fn get_paused_state(&self) -> bool
    {
        self.tty_paused
    }

    fn set_paused_state(&mut self, state: bool)
    {
        self.tty_paused = state;
    }

    fn get_preserve_next_state(&self) -> bool
    {
        self.tty_preserve_next
    }

    fn set_preserve_next_state(&mut self, state: bool)
    {
        self.tty_preserve_next = state;
    }
}