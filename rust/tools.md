1. modified `setup.sh` from 3210 (with fedora for now) (`binutils`, `rustup target add [risc-v-target-triple]` or `xbuild`, `rustc` > 1.31.1)
2. gdb and qemu (`sudo dnf install gdb qemu-system-[arm]` replaced with risc-v)
2.5 openocd for hardware `sudo dnf install openocd` and [udev rules for openocd](https://docs.rust-embedded.org/book/intro/install/linux.html#udev-rules)
3. cargo-generate (`cargo install cargo-generate` +? `libssl-dev` +? `pkg-config` for ubuntu, etc.)