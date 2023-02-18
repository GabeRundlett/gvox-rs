# gvox-rs
## Safe, high-level Rust API for the [GVOX voxel data library](https://github.com/GabeRundlett/gvox)

[![Crates.io](https://img.shields.io/crates/v/gvox_rs.svg)](https://crates.io/crates/gvox_rs)
[![Docs.rs](https://docs.rs/gvox_rs/badge.svg)](https://docs.rs/gvox_rs)

This library supplies an idiomatic Rust abstraction over the GVOX C API. It provides type safety, memory safety, and thread safety
without any significant deviation from the C library's design. For more information on the API's design, see the [GVOX Wiki](https://github.com/GabeRundlett/gvox/wiki).

Below is a simple example which demonstrates how to create adapter contexts and utilize them to convert a `.gvox` file to colored text console output.
For additional examples, see the tests in `src/tests.rs`.



## Building
For now, you must have the following things installed to build the repository
 * A C++ compiler
 * CMake (3.21 or higher)
 * Ninja build
 * vcpkg (plus the VCPKG_ROOT environment variable)
 * The latest WASI_SDK (if you are building for WASM)