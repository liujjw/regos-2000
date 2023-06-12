use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
  let libdir_path = PathBuf::from("file")
    .canonicalize()
    .expect("Couldn't find file/ directory");
  let headers_path = libdir_path.join("wrapper.h");
  let headers_path_str = headers_path.to_str().expect("Path is not a valid string");
  // let obj_path = libdir_path.join("egos_file.o");
  // let lib_path = libdir_path.join("libegos_file.a");
  let bindings_out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

  // cargo build scripts can emit environment variables/print to stdout
  // to tell cargo how to link
  println!("cargo:rustc-link-search={}", libdir_path.to_str().unwrap());
  println!("cargo:rustc-link-lib=static=egos_file");
  println!("cargo:rerun-if-changed={}", headers_path_str);

  // use cc to build C static library archive of the egos file/ directory, scraping 
  // target and then compiler using .cargo/config, ar is standard archiver
  cc::Build::new()
    .file(libdir_path.join("disk.h"))
    .file(libdir_path.join("egos.h"))
    .file(libdir_path.join("file.h"))
    .file(libdir_path.join("inode.h"))
    .file(libdir_path.join("disk.c"))
    .out_dir(&libdir_path)
    .compile("egos_file");

  // bindgen to generate Rust bindings for C library
  let bindings = bindgen::Builder::default()
    .header(headers_path_str)
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    .ctypes_prefix("cty")
    .use_core()
    .generate()
    .expect("Unable to generate bindings");

  bindings
    .write_to_file(bindings_out_path.join("bindings.rs"))
    .expect("Couldn't write bindings!");

}