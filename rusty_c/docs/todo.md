# General improvements
Negative trait impl of Unpin, PhantomData to have static analysis on lifetimes for FFI with C pointers.

1. fix important todos (copy, free, spinlock thats locked forever or moving and an option )
3. testing (LD_LIBRARY_PATH? since cargo test does not use build.rs get cargo test through .cargo and rustc to work and other testing)
run the linter

-> figure out what the exception handler error is and see if i can run qemu shell commands (read the qemu bootup logs)
i have the student version of egos so this will be tough to run things
on qemu, i cannot write to disk

since spinlock wont work on riscv32i, try an rcu lock:
let guard = rcu::read_lock();
let pair = value.try_access_with_guard(&guard)?;
The more rusty way is to contain the guard and let the guard be dropped by RAII.
Use some Empty types?

# Debugging
gdb check 
valgrind check
## 3 memory bugs at the FFI boundary (encapsulation/decapsulation of pointers)
1. creating a new pointer instead of returning the original one
2. read persistence to a buffer 
2.5. For 1. and 2. added proper abstraction to a C pointer: a locked &mut ref
3. when returning malloc'ed pointers to C, C must take care to free them

# Fall 2023 CS4999
## Concurrency + full filesystem 
### Step 1
More realisitic filesystem (treedisk). Can use c2rust and then create safe wrappers. Incoporate into build system and existing test file. Get rid if mallocs

### Step 2
Adding IO concurrency to the filesystem (async io, callbacks). E.g. Make a read to the filesystem, but return the results asynchronously, and give the read call a callback to run when the result is returned.

Need to update the inode interface to allow async IO.
#### Rust for Egos/Linux
async, abstractions, build, coould write something directly in linux. i have to write all abstractions from scratch, but can take inspriation from the rust for linux project. make some OS component useful for an embedded system, but niche enough. along the flavor of a 9p networked filesystem.
See Rust for Linux async bindings, RFC locks.

### Step 3
Add a cache access layer (another filesystem layer) that is properly tuned so that it is appropriately updated on reads and writes. This cache layer is below the filesystem layer but above the disk layer.

### Step 4
Block access in parallel, locks may come in so you need to lock a piece of memory or a whole inode or some other granularity, sync comes in as well. UI for user may still be sync (RW lock the inodes) (freelist needs its own lock).

### Step 5: Peripherals
RAID 0 and RAID 1 disk exists in egos 
init (read, write, getsize, setsize)
clockdisk
fatdisk
ramdisk/sddisk (sd driver readblock and writeblock interface (make the same interface))
make raid layers in rust
direction: disk drivers and a raid controller in Rust on top of the filesystem abstractions to see the benefits of static guarantees and state machines and encoding state in types with rust, sounds fancy, but hard to measure the benefits

or new drivers could use these platform crates for cortex-m or x86_64, dont need egos
pursue a research direction (papers) and using the riscv crates and extending egos
egos an easy to understand and extend os to ompl new ideas, like RAID or new drivers in Rust 
using HAL crates, cs4999, and using static guarantees from the embedded rust book and 
looking into more of the theory of rust to create something https://faultlore.com/blah/linear-rust/

RAID controllers disks built with HALS and state machines, get static guarantees on peripherals for proper configuration and access control
strict types for proper configuration of peripherals, access control
obrm + refernces for memory safety


# Aug 29
(real rust filesystem for egos)
1. refactoring and modularizing existing code to reuse
2. automation of porting c code to rust? not there yet...
3. rewriting, fatdisk with existing code simpler, asap
4. TDD