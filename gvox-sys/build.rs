extern crate bindgen;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_ARCH") == Ok("wasm32".to_string()) {
        let dst = cmake::Config::new("gvox")
            .build_target("gvox")
            .generator("Ninja Multi-Config")
            .configure_arg("-DGVOX_BUILD_FOR_RUST=1")
            .configure_arg(format!("-DCMAKE_TOOLCHAIN_FILE={}/scripts/buildsystems/vcpkg.cmake", std::env::var("VCPKG_ROOT").unwrap()))
            .configure_arg(format!("-DVCPKG_OVERLAY_TRIPLETS={}/gvox/cmake/vcpkg_triplets", std::env::current_dir().unwrap().display()))
            .configure_arg("-DVCPKG_TARGET_TRIPLET=wasm32-wasisdk")
            .configure_arg(format!("-DVCPKG_CHAINLOAD_TOOLCHAIN_FILE={}/gvox/cmake/toolchains/wasi-llvm-unknown-unknown.cmake", std::env::current_dir().unwrap().display()))
            .profile(get_profile())
            .build();
        println!(
            "cargo:rustc-link-search={}/share/wasi-sysroot/lib/wasm32-wasi",
            env::var("WASI_SDK_PATH").unwrap()
        );
        println!("cargo:rustc-link-lib=dylib=c++");
        println!("cargo:rustc-link-lib=dylib=c++abi");
        println!(
            "cargo:rustc-link-search=native={}/build/{}",
            dst.display(),
            get_profile()
        );
    } else {
        let static_crt = std::env::var("CARGO_ENCODED_RUSTFLAGS")
            .unwrap_or_default()
            .contains("target-feature=+crt-static");
        let dst = cmake::Config::new("gvox")
            .build_target("gvox")
            .profile(get_profile())
            .configure_arg("-DGVOX_BUILD_FOR_RUST=1")
            .configure_arg(format!(
                "-DGVOX_USE_STATIC_CRT={}",
                if static_crt { 1 } else { 0 }
            ))
            .build();
        println!(
            "cargo:rustc-link-search=native={}/build/{}",
            dst.display(),
            get_profile()
        );
    }
    println!("cargo:rustc-link-lib=static=gvox");
    use std::env;
    use std::path::PathBuf;
    println!("cargo:rerun-if-changed=src/gvox.h");
    println!("cargo:rerun-if-changed=gvox");
    let bindings = bindgen::Builder::default()
        .clang_arg("--target=x86_64-pc-windows-msvc")
        .clang_arg("--language=c")
        .clang_arg("-DGVOX_ENABLE_FILE_IO=0")
        .header("src/gvox.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn get_profile() -> &'static str {
    // For now, just use release. Thanks Douglas!
    // https://github.com/rust-lang/rust/issues/39016
    "Release"

    // match std::env::var("DEBUG").unwrap_or_default().as_str() {
    //     "true" => "Debug",
    //     "false" => "Release",
    //     _ => panic!("Couldn't detect target profile for CMake.")
    // }
}
