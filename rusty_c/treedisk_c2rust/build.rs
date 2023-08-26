
use std::env;
use std::path::PathBuf;
use std::process::Command;

extern crate cbindgen;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let super_directory =
        PathBuf::from("../riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14/bin")
            .canonicalize()
            .expect("couldnt find rv-gcc toolchain");
    let riscv_gcc_path = super_directory.join("riscv64-unknown-elf-gcc");
    let directory = PathBuf::from("file")
        .canonicalize()
        .expect("Couldn't find file/ directory");
    let headers_path = directory.join("wrapper.h");
    let headers_path_str = headers_path.to_str().expect("Path is not a valid string");
    let bindings_out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    println!("cargo:rustc-link-search={}", directory.to_str().unwrap());

    println!("cargo:rerun-if-changed={}", headers_path_str);

    let bindings = bindgen::Builder::default()
        .header(headers_path_str)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .ctypes_prefix("cty")
        .use_core()
        .clang_arg("--target=riscv32-unknown-none-elf")
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(bindings_out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("bindings.h");
}
