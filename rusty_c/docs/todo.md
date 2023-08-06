# General improvements
Negative trait impl of Unpin, PhantomData to have static analysis on lifetimes for FFI with C pointers.

1. fix important todos (copy, free, spinlock thats locked forever or moving and an option )
3. testing (LD_LIBRARY_PATH? since cargo test does not use build.rs get cargo test through .cargo and rustc to work and other testing)
run the linter

-> figure out what the exception handler error is and see if i can run qemu shell commands (read the qemu bootup logs)
i have the student version of egos so this will be tough to run things
on qemu, i cannot write to disk


# Debugging
gdb b r_359, b r_365, b c_74
valgrind
## 3 memory bugs at the FFI boundary (encapsulation/decapsulation of pointers)
1. creating a new pointer instead of returning the original one
2. read persistence to a buffer
3. when returning pointers to C, C must take care to free them

# More of the same future
more realistic filesystem like the treedisk or the fatdisk running in egos
entire operating system kernel in rust, look at egos kernel written entirely in rust
run rust programs on top of egos
kernel modules in rust, as well as rust in userspace in terms of applications
egos kernel written in rust 


# Peripheral Future 
RAID 0 and RAID 1 disk exists in egos 
init (read, write, getsize, setsize)
clockdisk
fatdisk
ramdisk/sddisk (sd driver readblock and writeblock interface (make the same interface))
make raid layers in rust
direction: disk drivers and a raid controller in Rust on top of the filesystem abstractions to see the benefits of static guarantees and state machines and encoding state in types with rust, sounds fancy, but hard to measure the benefits

