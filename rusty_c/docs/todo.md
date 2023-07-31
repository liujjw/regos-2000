1. fix important todos (copy, free, spinlock thats locked forever or moving and an option )
3. testing (LD_LIBRARY_PATH? since cargo test does not use build.rs get cargo test through .cargo and rustc to work and other testing)
run the linter

-> figure out what the exception handler error is and see if i can run qemu shell commands (read the qemu bootup logs)
i have the student version of egos so this will be tough to run things
on qemu, i cannot write to disk

easier to just test on x86? 
is it enough to have mkfs.c the bootup logs to demonstrate working on x86 and riscv? 

gdb ./tools/rust_test b 76 and inspect the metadata with a smaller FS_DISK_SIZE

# General improvements
Negative trait impl of Unpin, PhantomData to have static analysis on lifetimes for FFI with C pointers.