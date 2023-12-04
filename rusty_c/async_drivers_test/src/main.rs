#![no_std]
#![no_main]

extern crate panic_halt;
extern crate riscv_rt;

use riscv_rt::entry;
use heapless::Vec;


// REAL
// alloc::collections::vec_deque

#[entry]
fn main() -> ! {
    // approach 1
    // write a toy kernel image just for testing, we wont write everything in egos itself yet, use toy features like heapless
    // simulate interrupts and observe behavior in main()
    // when we do need to integrate into egos, insert at MARK using REAL code

    // approach 2
    // integrate into egso from the start (hard to test and reason about)
    // downside is we need some utilites that egos has like tty interface 
    // and handling interrupts as well and heap memory management
    loop {}
}

mod Interrupts {
    // use bindings to register interrupt handler
    // earth->intr_register(intr_entry);

    // write a interrupt handler
    // void intr_entry(int id); 

    // 1.5 not sure if all the registers are automatically saved when the interrupt handler is called in kernel mode (we can instruct the compiler to insert instructions to save certain registers for us, or we can use assembly to manually save certain registers)
    // 2. (`main.rs`) register (`process.c`) and modify the interrupt handler (`void intr_entry(int id)`), disabling interrupts while in the handler, (set the interrupts from SP1 controller from step 1) enable interrupts in the 1) `SPI1` controller interface chapter19 2) and in `PLIC` interface, and return from handler with setting an `mepc` program counter and an `mret` instruction
}

mod SD_Driver {
    // 1. (`sd-rw.c`) modify the sd card driver, sp1 chatper 19, instead of busy waiting in `sd_rw.c` for `recv_data_byte`, we enqueue reads and writes, then when we receive interrupts for when the device is ready, we continue with `sd_exec_cmd` which returns immediately, then instead of busy waiting for the response, we return until interrupted again, when we receive it we finish our upcalls (async)
}