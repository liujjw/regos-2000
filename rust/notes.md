1. Use FFI into C for `malloc`, otherwise include the platform specific allocator crate.
2. Collections only available if a global default allocator is configured.
3. no_std -> no libstd, no libc, no rust runtime or POSIX 