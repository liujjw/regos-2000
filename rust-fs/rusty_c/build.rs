use std::env;
use std::path::PathBuf;

fn main() {
  // generate ffi bindings
  println!("cargo:rustc-link-search=.");
  println!("cargo:rustc-link-lib=file");
  println!("cargo:rustc-link-lib=egos");
  println!("cargo:rerun-if-changed=wrapper.h");

  // use cty instead of std
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

  // call make to build apps layer
  let status = std::process::Command::new("make")
    .current_dir("egos")
    .status()
    .expect("failed to execute make, ensure riscv toolchain and qemu installed, see setup scripts");
  // copy over static library to target build directory 

}