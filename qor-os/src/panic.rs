//! Panic Implementation

use crate::*;

/// Panic handler for the kernel
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> !
{
    kerror!("\nAborting: ");

    if let Some(p) = info.location()
    {
        kerrorln!("line {}, file {}: {}", p.line(), p.file(), info.message().unwrap());
    }
    else
    {
        kerrorln!("no info available");
    }

    unsafe { crate::drivers::POWER_DRIVER.shutdown() };

    abort();
}

/// Terminate execution by waiting in a loop
#[no_mangle]
extern "C"
fn abort() -> !
{
    loop
    {
        unsafe { riscv::asm::wfi() };
    }
}