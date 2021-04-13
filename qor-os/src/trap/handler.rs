use crate::*;

use super::frame::TrapFrame;

#[no_mangle]
extern "C"
fn m_trap(epc: usize, tval: usize, cause: usize, hart: usize, _status: usize, _frame: &mut TrapFrame) -> usize
{
    // The trap is async if bit 63 of the cause registers is set
    let is_async = cause >> 63 & 1 == 1;

    // Extract the cause
    let cause_num = cause & 0xfff;

    let mut return_pc = epc;

    kdebug!(Interrupts, "CPU {}, Inst: 0x{:08x}:     ", hart, epc);

    match (cause_num, is_async)
    {
        (2, false) =>
        {
            // Illegal Instruction
            panic!("Illegal Instruction 0x:{:08x}", tval);
        },
        (3, true) =>
        {
            // Machine Software Interrupt (this should never happen)
            panic!("Machine Software Interrupt");
        },
        (7, true) =>
        {
            // Hardware Timer Interrupt
            kdebugln!(Interrupts, "Timer Interrupt");
            drivers::TIMER_DRIVER.set_remaining_time(1_000_000);
        },
        (8, false) =>
        {
            // ECALL from Supervisor Mode
            kdebugln!(Interrupts, "Supervisor Mode ECALL");
            return_pc += 4;
        },
        (11, false) =>
        {
            // ECALL from Machine Mode
            kdebugln!(Interrupts, "Machine Mode ECALL");
            return_pc += 4;
        },
        (11, true) =>
        {
            // Interrupt from the PIC
            let interrupt = drivers::PLIC_DRIVER.next().unwrap();
            kdebugln!(Interrupts, "Machine External Interrupt {}", interrupt);

            super::external::external_interrupt_handler(interrupt);

            drivers::PLIC_DRIVER.complete(interrupt);
        },
        (12, false) =>
        {
            // Instruction Page Fault
            panic!("Instruction Page Fault 0x:{:08x}", tval);
        },
        (13, false) =>
        {
            // Load Page Fault
            panic!("Load Page Fault 0x:{:08x}", tval);
        },
        (15, false) =>
        {
            // Store Page Fault
            panic!("Store Page Fault 0x:{:08x}", tval);
        },
        _ => 
        {
            // Unhandled exception
            panic!("Unhandled {} trap 0x{:x}", if is_async {"async"} else {"sync"}, cause);
        }
    }

    return_pc
}