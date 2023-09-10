# info
See the [slides](https://docs.google.com/presentation/d/10hhuhs7llfoi1PVC1HcoEz6FisEelULdneU84EUE_8o/edit?usp=sharing) for a high level overview and more context. egos-2000 with drop-in rust modules. regos-2000 invokes rust -> earth -> grass, because rust code permeates the original egos-2000.

# getting the right environment
**NOTE** Fedora 36 seems to work just fine, and none of the below is necessary.
**NOTE** ARM Macbooks (the ones with M[n] chips) are not supported.

Install Vagrant by Hashicorp for your (`x86`) OS and a hypervisor (KVM, VirtualBox, etc on Linux x86), set those up. Use the provided `Vagrantfile`.

**Use the following steps whenever running or developing.**  

**NOTE** `fourteen` is for cargo builds, `ateen` is for `c2rust`, and `twenty` is for `make` (`twenty` may also make cargo builds easier since its `clang` version should be high enough). `make qemu` does not work in `ubuntu14`. 

For example, you need to use `vagrant ssh twenty` after `vagrant up twenty` to `ssh` into the `ubuntu20` distro.

`vagrant up` in root of this project on your host computer and then `vagrant ssh [vm]` with `vagrant` as password. You are now inside a VM if you didn't notice any errors. The root of this project in the VM is at `/vagrant` (it's a "shared" folder between the host OS and the VM, changes in here are preserved between host and VM). 

# setup scripts
Now `cd /vagrant/rusty_c/` into the Rust project and run the setup scripts for the VM platform used by the Vagrantfile. To build just Rust, we need `cargo` and the right target architecture cross-compiler (e.g. `riscv-32i`); to build C inside of Rust we need an up to date version of `clang` that only some later versions of ubuntu may provide, as well as the RISC-V toolchain C compiler.
**If error for missing binaries re-export (`source ./exports.sh`) or add build tools into PATH variable.**
**If `cargo` not found restart the terminal after setup.**

# build and run on rv32i/qemu
1. Switch to `/mydisk`. Make sure the `build.rs` file is setup properly. It is needed to generate the `C` bindings for the Rust code called `bindings.h`. The Rust bindings to C have a canonical path where they will be found (see `bindings/` module).
2. Set the target in `.cargo`, 
3. then `cargo build --release`. 
4. Make sure the generated `C` bindings are up to date and placed in the right folders for the `C` code. 
5. Then follow the `egos` build process: `make rust_apps` ==> `make rust_install` ==> `make qemu`. 

# Debugging + testing + running on x86
The makefile target `make rust_test` and `make rust_test_fatdisk` runs a basic test and produces binaries like `rust_test` in the `tools/` directory. This binary is built from C code linked with the Rust object code. This binary can be stepped through with `gdb`. T

# miscellaneous
# c2rust treedisk
Similar process as before, but can now use instructions in `c2rust/` to generate a raw transpilation, which you can modify for `cargo build`.

## Rust FFI to C
Auto-generate Rust FFI bindings for C libraries using `cargo build`, building and linking C files as well (`bindgen`, `cc`, etc. using the build script `build.rs`). Look for `bindings.rs` in `target/` to see what they look like. The generated FFI bindings are dumped with the `include!` macro into your Rust library. Run `cargo test` to verify layout, size, and alignment. 

## C FFI to Rust
A header file of the Rust function signatures suffices. Make sure demangling and C ABI is used in your Rust functions if they are to be called by C. Make sure your Rust library is compiled as a static library archive (`.a` file in `target/`) and copy and link it in the Makefile.

## Directories explained
`/mydisk` contains the implementation of a basic filesystem. `treedisk_c2rust` contains the implementation of a transpiled `treedisk.c` into Rust. It needs to be manually reviewed. The `super` prefix in this directory means all the declarations are in one file for simplicity.

## Addressing large Rust binaries for the memory layout
The `/apps` memory region only has `~12KB` of memory, with `~16MB` used in total from text section up to the heap for all three parts of the kernel (earth, grass, apps). Stack pointer starts at roughly the 2048th megabyte. With dead code elimination the size of the full runtime crates don't matter, we just make sure to use as little as possible. Crates are also verified to be under the category "No standard library".

See `Cargo.toml` for `--release` optimizations. We try to avoid enlarging the `egos` memory layout if possible. Speed and debugging symbols were sacrificed for a smaller binary to fit in `egos` memory.

## Running on QEMU
Writing to disk not possible when running egos on QEMU, as only the memory-mapped ROM is readable.
