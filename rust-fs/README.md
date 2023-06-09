# info
egos-2000 filesystem in rust

## usage
### setup
Run setup scripts. Now `cd` into the Rust project.

### FFI
Auto-generates Rust FFI bindings for C using `cargo build`. Look for `bindings.rs` in `target/`. Uses Rust `bindgen` (`build.rs` and `wrapper.h`). 

### FFI API
The generated FFI bindings are dumped with the `include!` macro. Look for `bindings.rs` in `target/` for raw FFI bindings. Run `cargo build` and `cargo test` to verify layout, size, and alignment.

### linking