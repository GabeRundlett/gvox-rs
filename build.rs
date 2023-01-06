// extern crate bindgen;

fn main() {
    let dst = cmake::Config::new("gvox")
        .build_target("gvox")
        .out_dir(".out")
        .profile("Release")
        .build();
    use std::env;
    println!(
        "cargo:rustc-link-search={}/.out/build/Release",
        env::var_os("CARGO_MANIFEST_DIR")
            .unwrap()
            .to_str()
            .expect("Failed converting to str")
    );
    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=gvox");

    // use std::path::PathBuf;
    // println!("cargo:rerun-if-changed=gvox/include/gvox/gvox.h");
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
