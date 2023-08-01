1. fix important todos (copy, free, spinlock thats locked forever or moving and an option )
3. testing (LD_LIBRARY_PATH? since cargo test does not use build.rs get cargo test through .cargo and rustc to work and other testing)
run the linter


go through the makefile targets

-> figure out what the exception handler error is and see if i can run qemu shell commands (read the qemu bootup logs)
i have the student version of egos so this will be tough to run things
on qemu, i cannot write to disk

easier to just test on x86? 
is it enough to have mkfs.c the bootup logs to demonstrate working on x86 and riscv? 
gdb ./tools/rust_test b 76 and inspect the metadata and fs array with a smaller FS_DISK_SIZE


i think i have a conclusion (diagram): the ffi boundary is a dangerous place where pointers are passed around in unsafe blocks, leading to memory issues (sharing memory), the exact thing we want to avoid, however, once wrapped properly, we should get the benefits of rust safe?, C is danger!
extra danger from sharing memory we are sharing memory at the ffi boundary, hence locks needed for example 

direction: disk drivers and a raid controller in Rust on top of the filesystem abstractions to see the benefits of static guarantees and state machines and encoding state in types with rust, sounds fancy, but hard to measure the benefits

# General improvements
Negative trait impl of Unpin, PhantomData to have static analysis on lifetimes for FFI with C pointers.


b r_359, b r_365, b c_74



kernel modules in rust, as well as rust in userspace in terms of applications
egos kernel written in rust 

more realistic filesystem like the treedisk or the fatdisk running in egos
entire operating system kernel in rust, look at egos kernel written entirely in rust
run rust programs on top of egos


RAID 0 and RAID 1 disk exists in egos 

init (read, write, getsize, setsize)
clockdisk
fatdisk
ramdisk/sddisk (sd driver readblock and writeblock interface (make the same interface))

make raid layers in rust

make demo may be hard for presentation, focus more on motivation and challenges