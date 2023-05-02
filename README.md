# gvox-rs
## Safe, high-level Rust API for the [GVOX voxel data library](https://github.com/GabeRundlett/gvox)

[![Crates.io](https://img.shields.io/crates/v/gvox-rs.svg)](https://crates.io/crates/gvox-rs)
[![Docs.rs](https://docs.rs/gvox-rs/badge.svg)](https://docs.rs/gvox-rs)

This library supplies an idiomatic Rust abstraction over the GVOX C API. It provides type safety, memory safety, and thread safety
without any significant deviation from the C library's design. For more information on the API's design, see the [GVOX Wiki](https://github.com/GabeRundlett/gvox/wiki).

Below is a simple example which demonstrates how to create adapter contexts and utilize them to convert a `.gvox` file to colored text console output.
For additional examples, see the tests in `src/tests.rs`.

```rust
const BYTES: &[u8] = include_bytes!("palette.gvox");
let mut o_buffer = Box::default();

{
    let gvox_ctx = gvox_rs::Context::new();

    let o_config = gvox_rs::adapters::ByteBufferOutputAdapterConfig::from(&mut o_buffer);

    let s_config = gvox_rs::adapters::ColoredTextSerializeAdapterConfig {
        downscale_factor: 1,
        downscale_mode: gvox_rs::adapters::ColoredTextSerializeAdapterDownscaleMode::Nearest,
        non_color_max_value: 5,
    };

    let mut i_ctx = gvox_ctx.get_adapter::<gvox_rs::Input, gvox_rs::adapters::ByteBuffer>()
        .expect("Failed to get byte buffer input adapter.").create_adapter_context(BYTES)
        .expect("Failed to create adapter context.");

    let mut o_ctx = gvox_ctx.get_adapter::<gvox_rs::Output, gvox_rs::adapters::ByteBuffer>()
        .expect("Failed to get byte buffer input adapter.").create_adapter_context(o_config)
        .expect("Failed to create adapter context.");
    
    let mut p_ctx = gvox_ctx.get_adapter::<gvox_rs::Parse, gvox_rs::adapters::GvoxPalette>()
        .expect("Failed to get byte buffer input adapter.").create_adapter_context(())
        .expect("Failed to create adapter context.");

    let mut s_ctx = gvox_ctx.get_adapter::<gvox_rs::Serialize, gvox_rs::adapters::ColoredText>()
        .expect("Failed to get byte buffer input adapter.").create_adapter_context(s_config)
        .expect("Failed to create adapter context.");

    let region = gvox_rs::RegionRange {
        offset: gvox_rs::Offset3D {
            x: -4,
            y: -4,
            z: -4,
        },
        extent: gvox_rs::Extent3D { x: 8, y: 8, z: 8 },
    };

    gvox_rs::blit_region(
        &mut i_ctx,
        &mut o_ctx,
        &mut p_ctx,
        &mut s_ctx,
        &region,
        gvox_rs::ChannelId::COLOR | gvox_rs::ChannelId::NORMAL | gvox_rs::ChannelId::MATERIAL_ID,
    ).expect("Error while translating.");
}

assert_eq!(22228, o_buffer.len(), "Buffer output length did not match expected.");
println!("{}", std::str::from_utf8(&o_buffer).expect("Bad string slice."));
```

## Building
For now, you must have the following things installed to build the repository.
 * A C++ compiler
 * CMake (3.21 or higher)
 * Ninja build
 * vcpkg (plus the VCPKG_ROOT environment variable)
 * The latest WASI_SDK (if you are building for WASM)
These are necessary because the gvox-rs library is built on the gvox-sys library (subfolder), which is generated via bindgen in order to create language bindings to the C API of Gvox. See the [Gvox README](https://github.com/GabeRundlett/gvox/blob/master/README.md) for more info!
