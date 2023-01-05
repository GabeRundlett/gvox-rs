// extern crate bindgen;

fn main() {
    println!("cargo:rustc-link-search=prebuilt");
    println!("cargo:rustc-link-lib=gvox");

    // use std::env;
    // use std::path::PathBuf;
    // println!("cargo:rerun-if-changed=prebuilt/gvox.h");
    // let bindings = bindgen::Builder::default()
    //     .clang_arg("--target=x86_64-pc-windows-msvc")
    //     .clang_arg("--language=c")
    //     .header("prebuilt/gvox.h")
    //     .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    //     .generate()
    //     .expect("Unable to generate bindings");
    // let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    // bindings
    //     .write_to_file(out_path.join("bindings.rs"))
    //     .expect("Couldn't write bindings!");
}
