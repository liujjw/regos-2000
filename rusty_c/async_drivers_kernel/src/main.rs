#![no_std]
#![no_main]

extern crate panic_halt;
extern crate riscv_rt;

use riscv_rt::entry;

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
    let mut sd_driver = SD_Driver::new();
    sd_driver.single_read([0; 512]);
    sd_driver.single_write([0; 512]);
    unsafe {
        Interrupts::enable_interrupts_set_handler();
    }
    loop {}
}

mod Interrupts {
    use riscv;
    use riscv::interrupt::enable;
    use riscv::register::mtvec;

    // 1.5 not sure if all the registers are automatically saved when the interrupt handler is called in kernel mode (we can instruct the compiler to insert instructions to save certain registers for us, or we can use assembly to manually save certain registers)
    // 2. in egos: enable general interrupts and set interrupt handler(`cpu_intr.c`) (done already), register (`process.c`) (already done) and modify the interrupt handler (`void intr_entry(int id)`) (todo since empty)
    pub fn enable_interrupts_set_handler() {
        unsafe {
            riscv::interrupt::enable();
            mtvec::write(interrupt_handler as usize, mtvec::TrapMode::Direct); 
        }
    } 

    // 3. enable interrupts in the 1) `SPI1` controller interface chapter19  disabling interrupts while in the handler 2) and in `PLIC` interface, and return from handler with setting an `mepc` program counter and an `mret` instruction
    pub fn interrupt_handler() {
        // assume only interrupts as mcause 
        // files to reference for manual impl:
        // `cpu_intr.c, disk.h, egos.h, sd.h, sd_utils.c, sd_init.c, sd_rw.c`
    }
}

// replace the sd card driver in `dev_disk.c`
mod SD_Driver {
    use heapless::Deque;

    // 1. rewrite `sd-rw.c`, sp1 chatper 19
    // instead of busy waiting in `sd_rw.c` for `recv_data_byte`, we enqueue reads and writes
    // then when we receive interrupts for when the device is ready, we continue with `sd_exec_cmd` which returns immediately, then instead of busy waiting for the response, we return until interrupted again, when we receive it we finish our upcalls (async)
    enum IO_CallType {
        Read,
        Write,
    }

    pub struct IO_Call {
        call_type: IO_CallType,
        bytes: [u8; 512]
    }

    pub struct SD_Driver {
        queue: Deque<IO_Call, 32>,
    }

    impl SD_Driver {
        pub fn single_read(&mut self, bytes: [u8; 512]) -> Result<(), IO_Call> {
            self.queue.push_back(IO_Call {
                call_type: IO_CallType::Read,
                bytes: bytes,
            })
        }

        pub fn single_write(&mut self, bytes: [u8; 512]) -> Result<(), IO_Call> {
            self.queue.push_back(IO_Call {
                call_type: IO_CallType::Write,
                bytes: bytes,
            })
        }
    }

    pub fn new() -> SD_Driver {
        SD_Driver {
            queue: Deque::new(),
        }
    }
}