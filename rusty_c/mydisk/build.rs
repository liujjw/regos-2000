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

    // also remove this line
    // println!("cargo:rustc-link-lib=static=egos_file");
    // end remove

    println!("cargo:rerun-if-changed={}", headers_path_str);

    // TODO remove build and linkage from egos compilation or remove from here
    // let mut cc_builder = cc::Build::new();

    // TODO remove cc_builder code
    // #[cfg(not(unix))]
    // {
    //     cc_builder.compiler(riscv_gcc_path);
    // }
    // cc_builder
    //     .file(directory.join("disk.h"))
    //     .file(directory.join("egos.h"))
    //     .file(directory.join("file.h"))
    //     .file(directory.join("inode.h"))
    //     .file(directory.join("disk.c"));

    // #[cfg(not(unix))]
    // {
    //     cc_builder
    //         .no_default_flags(true)
    //         .flag("-mcmodel=medlow")
    //         .flag("-march=rv32i")
    //         .flag("-mabi=ilp32")
    //         .flag("-ffunction-sections")
    //         .flag("-fdata-sections");
    // }
    // cc_builder.out_dir(&directory).compile("egos_file");
    // end remove

    #[cfg(unix)] {
        let bindings = bindgen::Builder::default()
        .header(headers_path_str)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .ctypes_prefix("cty")
        .use_core()
        .clang_arg("--target=x86_64-unknown-linux-gnu")
        .generate()
        .expect("Unable to generate bindings");

        bindings
            .write_to_file(bindings_out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }

    #[cfg(not(unix))] {
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
    }

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("bindings.h");
}
