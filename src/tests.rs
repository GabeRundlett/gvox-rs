use std::ptr::{null, null_mut};

use crate::{self as gvox_rs};
//mod procedural_parse;

macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0") as *const str as *const [std::os::raw::c_char]
            as *const std::os::raw::c_char
    };
}
        
const BYTES: &[u8] = include_bytes!("palette.gvox");

#[test]
fn gvox_rs_test_procedural() {
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
            .expect("Failed to get byte buffer input adapter.").create_adapter_context(&BYTES)
            .expect("Failed to create adapter context.");
    
        let mut o_ctx = gvox_ctx.get_adapter::<gvox_rs::Output, gvox_rs::adapters::ByteBuffer>()
            .expect("Failed to get byte buffer input adapter.").create_adapter_context(&o_config)
            .expect("Failed to create adapter context.");
        
        let mut p_ctx = gvox_ctx.get_adapter::<gvox_rs::Parse, gvox_rs::adapters::GvoxPalette>()
            .expect("Failed to get byte buffer input adapter.").create_adapter_context(&())
            .expect("Failed to create adapter context.");
    
        let mut s_ctx = gvox_ctx.get_adapter::<gvox_rs::Serialize, gvox_rs::adapters::ColoredText>()
            .expect("Failed to get byte buffer input adapter.").create_adapter_context(&s_config)
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

    println!("{}", std::str::from_utf8(&o_buffer).expect("Bad string slice."));
}

#[test]
fn test_blit_error() {
    let gvox_ctx = gvox_rs::Context::new();

    let mut o_buffer = Box::default();
    let o_config = gvox_rs::adapters::ByteBufferOutputAdapterConfig::from(&mut o_buffer);

    let s_config = gvox_rs::adapters::ColoredTextSerializeAdapterConfig {
        downscale_factor: 1,
        downscale_mode: gvox_rs::adapters::ColoredTextSerializeAdapterDownscaleMode::Nearest,
        non_color_max_value: 5,
    };

    let mut i_ctx = gvox_ctx.get_adapter::<gvox_rs::Input, gvox_rs::adapters::ByteBuffer>()
        .expect("Failed to get byte buffer input adapter.").create_adapter_context(&BYTES)
        .expect("Failed to create adapter context.");

    let mut o_ctx = gvox_ctx.get_adapter::<gvox_rs::Output, gvox_rs::adapters::ByteBuffer>()
        .expect("Failed to get byte buffer input adapter.").create_adapter_context(&o_config)
        .expect("Failed to create adapter context.");
    
    let mut p_ctx = gvox_ctx.get_adapter::<gvox_rs::Parse, gvox_rs::adapters::GvoxPalette>()
        .expect("Failed to get byte buffer input adapter.").create_adapter_context(&())
        .expect("Failed to create adapter context.");

    let mut s_ctx = gvox_ctx.get_adapter::<gvox_rs::Serialize, gvox_rs::adapters::ColoredText>()
        .expect("Failed to get byte buffer input adapter.").create_adapter_context(&s_config)
        .expect("Failed to create adapter context.");

    let region = gvox_rs::RegionRange {
        offset: gvox_rs::Offset3D {
            x: -4,
            y: -4,
            z: -4,
        },
        extent: gvox_rs::Extent3D { x: 8, y: 8, z: 8 },
    };

    let res = gvox_rs::blit_region(
        &mut i_ctx,
        &mut o_ctx,
        &mut p_ctx,
        &mut s_ctx,
        &region,
        gvox_rs::ChannelId::TRANSPARENCY.into(),
    ).map_err(|e| e.error_type());

    assert!(matches!(res, Err(gvox_rs::ErrorType::ParseAdapterRequestedChannelNotPresent)));
}