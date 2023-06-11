# info
egos-2000 filesystem in rust

## usage
### setup
Install Vagrant by Hashicorp for your host platform. Run `vagrant up`. Now `cd` into the Rust project and run the setup scripts for the VM platform used by Vagrant (if `bento/ubuntu-14.04 --box-version 201808.24.0`, use the ubuntu setup).

## Rust FFI to C
Auto-generate Rust FFI bindings for C using `cargo build`, building and linking C files as well. Look for `bindings.rs` and `*.a` archive in `target/`. Uses Rust `bindgen` and `cc` build crates (`build.rs` and `wrapper.h`).
### FFI API
The generated FFI bindings are dumped with the `include!` macro. Look for `bindings.rs` in `target/` for raw FFI bindings. Run `cargo test` to verify layout, size, and alignment.
### C FFI to Rust
Same general idea as Rust to C, but instead of `bindings.rs` a header file suffices.