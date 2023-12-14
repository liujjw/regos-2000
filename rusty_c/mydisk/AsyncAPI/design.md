# info 
build concurency into the kernel directly to speed up io
use a single thread, set of upcalls, these upcalls can be nested, values passed, and look synchronous if desired
implement a rw-lock on inodes
the upcalls can also be unrelated, be synchronized like the the fork-join pattern from c
basically, a scheduler runtime will determine which task to run, and yield the task when waiting on disk for example

# c api?
syntax and syntactic sugar
in c to simulate, `run("read", 1, 1, buf).then().then().then()...`

```
c_async_b("read", 1, 1, buf, upcall1), 
 upcall1 = int x = 1; c_await_b("read", 2, 3, buf, upcall2); int y = 2; 
   upcall2 = int z = 2; c_await_b("read", 4, 5, buf, upcall3);
     ...
```
the semantics of the above will be eager blocking synchrnonous execution of the first read,
however we cannot pass values to nested upcalls

then blocking asynchronous execution within the nested upcalls as needed we could also do
```
void join(an array of function pointers)
-or- 
c_async_b_join([
 c_await_nb("read", 1, 1, buf, null),
 c_await_nb("read", 1, 2, buf, null),
 c_await_nb("read", 1, 3, buf, null) 
])
```
`void async_run() { async_nb(), async_nb(), async_nb() }`
`void async_run() { async_b(), async_b(), async_b() }` // eager


# just rust
initially i am designing a c api, but stuck because C lacks a lot of language f
eatures which makes expressing  what i want to do difficult, for example 
its not clear how to pass values to nested upcalls without a promise library, 
lack of method syntax, we need too much to add to c.
but we need to call this code from C, or do we?

what if the only way to access the async code is through rust, 
so we can only use rust to get async io. we offer a synchronous filesystem api offered to
the c code, and an asynchrnous api when you use pure rust, and of course we have that
the c and rust code both  

# async rust api
rust: we use the built in async/await syntax 
`async fn run(type: &str) -> () {if type == "read" read.await?; }` // c wrapper (deprecated)
we can simply create another trait, `Async_Stackable`, which is imcompatible with `Stackable`
the following could be the signatures
`async fn read(ino: u32, offset: u32, buf: &mut Block) -> Result<i32, Error> {}`

in std, we have the following distinctions:
std::futures/futures-rs only contain types, not the runtime
lazy(Rust Tokio) vs eager(JS) engine 

in nostd, runtime/executor/engine: pasts vs Tokio

in tokio:
`async fn run() { tokio::join! {task1, task2, task3} }` // async and unrelated
`async fn run() { tokio::spawn(async {task1.await?; task2.await?; task3.await?;}) }` // eager, illusion of synchronous
let future = run(); // lazy future
let rt = runtime::new();
rt.block_on(future) // blocking
the executor must go over the specific tasks and run them, the executor does not go over all your code, hence top level call is blocking

# progress
toy code running on x86
REWRITE NECESSAERY

# riscv
works on x86, not riscv yet

# rw-lock
the instructions for riscv 

# meeting nov 9
* reentrancy of upcalls, timer?
* queue of func arg (timer) pointer pairs (thunks), an eager event loop that checks the queue of thunks algol60
* the timer is for relative order in simulations
* read(ino, offset, thunk) // the thunks can create further downcalls or further thunks, put new thunks at the end of the queue, 
* no nesting (if calling up that would be reentrant, so just call down and put a thunk at the end of the queue) 
* continuations with async/awaits? saving context and restoring context so bad, prev system not necessary
* use a makefile to encode 

# code progess for nov 16
## clock cache
clock cache will contain an event loop, if the elem is not in the cache, then we will submit those disk ops to the event loop



## batch processing
we can  have a fixed vector to loop on, so everytime we add a set of operations to our queue, we will loop some of them to run, and then keep looping until there are no more left (not dynamic)

need to figure out how long one of them takes though, ideally by running on the arduino board, or i can just set configuration parameters and benchmark the best batch size

## spawn
or i can spawn to put task on a queue, once the number of tasks on the queue is greater than the batch size, then we can run the batch, but no upcalls
or have a background daemon that runs the batch every so often, or if the batch is full
or just run the batch on reads, or if the batch is full (need logs for recovery on power loss)

## additional layer
have an additional layer which contains the event loop handler?

## clock cache no good
having an async wrapper over slow sync code wont make it faster, we need to rewrite the entire code down to be async
1. start with the lowest level disk read and write at the hardware level, make it async (rewrite all the code)
2. then have a layer on top of all async layers which contains the event loop handler (basically use pasts as the high level api, its the queue, and all the upcalls)
3. now the hard part is actually making the the low level code async, with interrupts or threads

## upcalls
we can register upcalls as described in the api

## api
figuring out the types and api, quite complicated

## config
we target x86 no_std for now

compiling using nightly
nightly-x86_64-unknown-linux-gnu (default)
rustc 1.74.0-nightly (9f5fc1bd4 2023-09-02)

## pre nov 28
instead off interrupt vector theres a single interrupt handler that determines the source of the interrupt

address of the interrupt handler (buffer block of memory) []
disk block regsiter # []
command control register []      the ram the disk ()

instead of polling the command control register for when teh control bit is set for when the read/write is done, we can use interrupts

interupts disabled in kernel but enabled in userspace, interruptsa reanbled on returned

it is safe for the interrupt handler to call the upcall since the ineterrupts are disabled in the kernel

read (modify the disk controller code to immediately return) -> complete immediately and move on -> interrupt will happen, there will be an interrupt handler (so modify the interrupt handler), and which will call the upcall we have specified, the upcall will be called in the interrupt handler, and the interrupt handler will return, and the kernel will return to the user space code, and the user space code will continue executing 

run queue (timesharing)
keyboard queue, network queue, disk queue, timer queue, in our case its the disk queue, remove from disk queue and stick it on the run queue
2 queues

* be careful about processes that want to read the same block so dont do the same operation make sure to read the same block only once

* moving from c to rust, its interesting in this case as a reasearch case
* egos 1 thing at a time, no async now, unusable

no locks needed since no threads

# nov 28
read sifive (they make the arty board) and riscv manuals, maybe theres a rust library for riscv cores that already implements a few things i need (critical section, disabling interrupts)

nothing compiles yet so i dont have demo

my general work flow is:
1. (`sd-rw.c`) modify the sd card driver, sp1 chatper 19, instead of busy waiting in `sd_rw.c` for `recv_data_byte`, we enqueue reads and writes, then when we receive interrupts for when the device is ready, we continue with `sd_exec_cmd` which returns immediately, then instead of busy waiting for the response, we return until interrupted again, when we receive it we finish our upcalls (async)
1.5 not sure if all the registers are automatically saved when the interrupt handler is called in kernel mode (we can instruct the compiler to insert instructions to save certain registers for us, or we can use assembly to manually save certain registers)
2. (`main.rs`) register (`process.c`) and modify the interrupt handler (`void intr_entry(int id)`), disabling interrupts while in the handler, (set the interrupts from SP1 controller from step 1) enable interrupts in the 1) `SPI1` controller interface chapter19 2) and in `PLIC` interface, and return from handler with setting an `mepc` program counter and an `mret` instruction


1. contacting yunhao with a list of questions 
2. hard to test on qemu since only supports ROM

# nov 28
keyboard another process can run
t2i server deals with keyboard io, so do the same thing as the keyboard driver
interruot mask auto set
programmable logic interrupt controller
multiplex controller, we can read the right register to figure out the device
one handler instead of interruot vector
c compiler automatically saves and resotres registers from the c atttribute
return from the handler auto generated??
if doing syscall then we need instruction dealing with special register in interrupt handler
fix stack pointer in interrupt handler
switch to kernel stack instead of saving stuff to user stack

# dec 5
make a toy kernel to run on the hardware board to run tests on, its not gonna have anything except the sd card driver, the interrupt handler, and the uart driver from egos to print to the screen
although in egos its easy to swap out the interrupt handler, but there's a lot of missing code in my copy and egos is too heavyweight too much code to consider for me, and i know how to integrate the driver later into egos

the part where im struggling is that 
the only keyboard driver i can find is uart based, the interface for the sd card is i2c or spi, at least in my copy of egos i dont think theres any complete interrupt handling code that i can reference

ive found the memory mapped io address for the interrupt register in the spi interface, but im still trying to figure out how to program it
ive also been looking at things like hardware abstraction layers to see if they help

trying to wrap my head around the spi interface in the sifve manual and the sd card spefication for the sd card, sending an interrupt, but trying to read various manuals theres a lot of electrical engineering and hardware terminology

finally found something called a hardware abstraction layer devloped by the makers of the board, starting using it
its in C, but we can use Rust with C so its fine

in the interrupts section, theres an HAL api i can use to register a PLIC interrupt
and then perform reads and writes using the HAL api for the spi interface
"metal_interrupt_register_handler"

notes
theres a riscv instruction for waiting when no process wants to run
wfi (wait for interrupt) not in egos-2000
scheduler may panic if no process in queeu (fixed)
fix for tty.c first
check clock for when a character is delivered
scheduler.c

# dec 12
look at 
sys_call.c
bus_uart.c 
dev_tty.c > queue.h, queue.c > cpu_intr.c > scheduler.c > process.h > context.S > ult.c

finish todos


sys_recv
sys_send
use WFI if no process