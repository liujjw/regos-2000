use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
  // let libdir_path = PathBuf::from("file")
  //   .canonicalize()
  //   .expect("Couldn't find file/ directory");
  // let headers_path = libdir_path.join("disk.h");
  // let headers_path_str = headers_path.to_str().expect("Path is not a valid string");

  // generate ffi bindings
  println!("cargo:rustc-link-search=file");
  println!("cargo:rustc-link-lib=file");
  println!("cargo:rustc-link-lib=egos");
  println!("cargo:rerun-if-changed=wrapper.h");

  let bindings = bindgen::Builder::default()
    .header("wrapper.h")
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    .ctypes_prefix("cty")
    .use_core()
    .generate()
    .expect("Unable to generate bindings");

  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("Couldn't write bindings!");

  // use cc to build C static library archive of the egos file/ directory
  // look for something like libegos_file.a in target/ directory
  cc::Build::new()
    .file("disk.h")
    .file("egos.h")
    .file("file.h")
    .file("inode.h")
    .compile("egos_file");
  // copy over C static library to target/ directory and link
  // 
}