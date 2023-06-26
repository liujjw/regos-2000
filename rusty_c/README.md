# info
egos-2000 with drop-in rust modules. regos-2000 invokes rust -> earth -> grass, because rust code permeates the original egos-2000.

## usage
**NOTE** ARM Macbooks (the ones with M[n] chips) are not supported.

Install Vagrant by Hashicorp for your (`x86`) OS and a hypervisor (KVM, VirtualBox, etc on Linux x86), set those up. Use the provided `Vagrantfile`.

**Use the following steps whenever running or developing.**  

**NOTE** `fourteen` is for cargo builds, `ateen` is for `c2rust`, and `twenty` is for make. For example, you need to use `vagrant ssh twenty` after `vagrant up twenty` to `ssh` into the `ubuntu18` distro and use `ubuntu18_setup_0` to setup `QEMU` and then you can call `make qemu`. `make qemu` does not work in `ubuntu14`.

`vagrant up` in root of this project on your host computer and then `vagrant ssh [vm]` with `vagrant` as password. You are now inside a VM if you didn't notice any errors. The root of this project in the VM is at `/vagrant` (it's a "shared" folder between the host OS and the VM, changes in here are preserved between host and VM). 
### setup
Now `cd /vagrant/rusty_c/rust_fs` into the Rust project and run the setup scripts for the VM platform used by the Vagrantfile (for example, if `bento/ubuntu-14.04 --box-version 201808.24.0`, use the ubuntu setup). 

## Writing Rust modules and integrating them into the current C build system
### Rust FFI to C
**If error for missing binaries re-export (`source ./exports.sh`) or add build tools into PATH variable.**
**If `cargo` not found restart the terminal after setup.**

Auto-generate Rust FFI bindings for C libraries using `cargo build`, building and linking C files as well (`bindgen`, `cc`, etc. using the build script `build.rs`). Look for `bindings.rs` in `target/` to see what they look like. The generated FFI bindings are dumped with the `include!` macro into your Rust library. Run `cargo test` to verify layout, size, and alignment. 

### C FFI to Rust
A header file of the Rust function signatures suffices. Make sure demangling and C ABI is used in your Rust functions if they are to be called by C. Make sure your Rust library is compiled as a static library archive (`.a` file in `target/`) and copy and link it in the Makefile.