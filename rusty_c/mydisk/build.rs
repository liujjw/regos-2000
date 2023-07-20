use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // test target config
    // println!("{}", env::var("CARGO_CFG_TARGET_FEATURE").unwrap());

    let super_directory = PathBuf::from("../riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14/bin")
        .canonicalize()
        .expect("couldnt find rv-gcc toolchain");
    let riscv_gcc_path = super_directory.join("riscv64-unknown-elf-gcc");
    let directory = PathBuf::from("file")
        .canonicalize()
        .expect("Couldn't find file/ directory");
    let headers_path = directory.join("wrapper.h");
    let headers_path_str = headers_path.to_str().expect("Path is not a valid string");
    // let obj_path = directory.join("egos_file.o");
    // let lib_path = directory.join("libegos_file.a");
    let bindings_out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    // let stdlibdir_path = std::process::Command::new("echo $(find / -name "stdlib.so" -print 2>/dev/null)/..");
    // let stringlibdir_path = std::process::Command::new("echo $(find / -name "string.so" -print 2>/dev/null)/..");
    // let output = Command::new("sh")
    //   .arg("-c")
    //   .arg("echo $(find / -name \"libc.so\" -print 2>/dev/null)/..")
    //   .output()
    //   .expect("Failed to run command");
    // let libclibdir_path = String::from_utf8_lossy(&output.stdout);

    // tell cargo how to link
    println!("cargo:rustc-link-search={}", directory.to_str().unwrap());
    // println!("cargo:rustc-link-search={}", stdlibdir_path);
    // println!("cargo:rustc-link-search={}", stringlibdir_path);
    // println!("cargo:rustc-link-search={}", libclibdir_path);

    println!("cargo:rustc-link-lib=static=egos_file");
    // println!("cargo:rustc-link-lib=static=stdlid");
    // println!("cargo:rustc-link-lib=static=string");
    // println!("cargo:rustc-link-lib=static=libc");

    println!("cargo:rerun-if-changed={}", headers_path_str);

    // use cc to build C static library archive of the egos file/ directory, scraping
    // target and then compiler using .cargo/config, ar is standard archiver
    cc::Build::new()
        .compiler(riscv_gcc_path)
        .file(directory.join("disk.h"))
        .file(directory.join("egos.h"))
        .file(directory.join("file.h"))
        .file(directory.join("inode.h"))
        .file(directory.join("disk.c"))
        .no_default_flags(true)
        .flag("-mcmodel=medlow")
        .flag("-march=rv32i")
        .flag("-mabi=ilp32")
        .flag("-ffunction-sections")
        .flag("-fdata-sections")
        .out_dir(&directory)
        .compile("egos_file");

    // bindgen to generate Rust bindings for C library
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
