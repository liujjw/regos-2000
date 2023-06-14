1. Use FFI into C for `malloc`, otherwise include the platform specific allocator crate.
2. Collections only available if a global default allocator is configured.
3. no_std -> no libstd, no libc, no rust runtime or POSIX 
4. Use appropriate C attributes like no_mangle and repr(C).
5. See `main()` and `README.md` of 3210 for ARMv8 or embedded rust book for ARM Cortex-M, [RISC-V](https://github.com/rust-embedded/riscv).
6. `make` > `make install` > `make qemu` before. Now, `cargo build`, modified `make` > `make install` > `make qemu`.
7. No defined `MKFS`, no `#include <stdio.h>`, and no `MKFS` panic because `fdopen` requires `POSIX` interface (`stdio` used for dev purposes? we use `egos.h` instead, but look at `egos.h` comments about the matter).
