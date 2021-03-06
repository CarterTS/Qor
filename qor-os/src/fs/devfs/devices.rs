use crate::*;

use fs::ioctl::IOControlCommand;

use process::descriptor::*;

use fs::structures::FilesystemIndex;

use super::tty::TeletypeDevice;

/// Device Directory Enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceDirectories
{
    Root,
    PseudoTerminalSecondaries
}

pub const PSUEDO_TERMINAL_FLAG: usize = 1 << (16 + 1);

impl core::fmt::Display for DeviceDirectories
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            DeviceDirectories::Root => write!(f, "root"),
            DeviceDirectories::PseudoTerminalSecondaries => write!(f, "pts"),
        }
    }
}

/// DeviceFile object
pub struct DeviceFile
{
    pub name: &'static str,
    pub directory: DeviceDirectories,
    desc_const: Box<dyn Fn(FilesystemIndex) -> Box<dyn FileDescriptor>>,
    io_ctl: Box<dyn Fn(IOControlCommand) -> usize>
}

impl DeviceFile
{
    /// Create a new device file
    pub fn new(name: &'static str, desc_const: Box<dyn Fn(FilesystemIndex) -> Box<dyn FileDescriptor>>,
               io_ctl: Box<dyn Fn(IOControlCommand) -> usize>) -> Self
    {
        Self
        {
            name, desc_const, io_ctl, directory: DeviceDirectories::Root
        }
    }

    /// Create a new device file in a sub directory
    pub fn new_in_dir(name: &'static str, desc_const: Box<dyn Fn(FilesystemIndex) -> Box<dyn FileDescriptor>>,
               io_ctl: Box<dyn Fn(IOControlCommand) -> usize>, directory: DeviceDirectories) -> Self
    {
        Self
        {
            name, desc_const, io_ctl, directory
        }
    }

    /// Make the descriptor
    pub fn make_descriptor(&self, index: FilesystemIndex) -> Box<dyn FileDescriptor>
    {
        (self.desc_const)(index)
    }

    /// Execute an ioctl command on the driver
    pub fn exec_ioctl(&self, cmd: IOControlCommand) -> usize
    {
        (self.io_ctl)(cmd)
    }
}

/// Return all available device directories for the system
pub fn get_device_directories() -> Vec<DeviceDirectories>
{
    let mut result: Vec<DeviceDirectories> = Vec::new();

    result.push(DeviceDirectories::PseudoTerminalSecondaries);

    result
}

/// Return all available device files for the system
pub fn get_device_files() -> Vec<DeviceFile>
{
    let mut result: Vec<DeviceFile> = Vec::new();
    
    // Only add graphics devices if the graphics driver is loaded
    if drivers::gpu::is_graphics_driver_loaded()
    {
        // /dev/disp : Text mode for the frame buffer
        result.push(
            DeviceFile::new(
                "disp",
                Box::new(
                    |inode| Box::new(
                        ByteInterfaceDescriptor::new(drivers::gpu::get_global_graphics_driver(), inode)
                    )),
                    Box::new( |_| usize::MAX)
                ));

        // /dev/fb0 : Raw frame buffer access
        result.push(
            DeviceFile::new(
                "fb0",
                Box::new(
                    |inode| Box::new(
                        BufferDescriptor::new(drivers::gpu::get_global_graphics_driver(), inode)
                    )),
                    Box::new( |cmd| drivers::gpu::get_global_graphics_driver().exec_ioctl(cmd))
                ));
    }

    // /dev/uart0 : UART Port
    result.push(
        DeviceFile::new(
            "uart0",
            Box::new(
                |inode| Box::new(
                    ByteInterfaceDescriptor::new(drivers::get_uart_driver(), inode)
                )),
                Box::new( |_| { usize::MAX })
            ));

    // /dev/tty0 : Teletype connected to the UART port
    result.push(
        DeviceFile::new(
            "tty0",
            Box::new(
                |inode| Box::new(
                    super::tty::TeletypeSecondaryDescriptor::new(drivers::get_uart_driver(), inode)
                )),
                Box::new( |cmd| { drivers::get_uart_driver().exec_ioctl(cmd) } )
            ));

    // /dev/null : Null Descriptor
    result.push(
        DeviceFile::new(
            "null",
            Box::new(
                |inode| Box::new(
                    NullDescriptor{ inode })
                ),
            Box::new( |_| usize::MAX)
        ));

    // TODO: This needs to respect the interrupt requirements of the RTC, however,
    // for right now we will just implement a null descriptor for it
    // /dev/rtc0 : Real Time Clock
    result.push(
        DeviceFile::new(
            "rtc0",
            Box::new(
                |inode| Box::new(
                    NullDescriptor{ inode })
                ),
            Box::new( |cmd| drivers::rtc::RealTimeClockDriver::get_driver().exec_ioctl(cmd))
        ));

    result
}