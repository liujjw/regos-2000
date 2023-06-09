use std::env;
use std::path::PathBuf;

fn main() {
  println!("cargo:rustc-link-search=.");
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
}