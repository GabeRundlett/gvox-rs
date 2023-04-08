#![allow(warnings)]

use std::ptr::{null, null_mut};

use crate::{self as gvox_rs};
mod procedural_parse;

macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0") as *const str as *const [std::os::raw::c_char]
            as *const std::os::raw::c_char
    };
}

const BYTES: &[u8] = include_bytes!("palette.gvox");

#[test]
fn test_version() {
    let gvox_version = gvox_rs::get_version();
    println!("{}.{}.{}", gvox_version.major, gvox_version.minor, gvox_version.patch);
}

#[test]
pub fn gvox_rs_test_procedural() {
    let mut o_buffer = Box::default();

    {
        let gvox_ctx = gvox_rs::Context::new();
        gvox_ctx.register_adapter::<gvox_rs::Parse, procedural_parse::Procedural>();

        let o_config = gvox_rs::adapters::ByteBufferOutputAdapterConfig::from(&mut o_buffer);

        let s_config = gvox_rs::adapters::ColoredTextSerializeAdapterConfig {
            downscale_factor: 1,
            downscale_mode: gvox_rs::adapters::ColoredTextSerializeAdapterDownscaleMode::Nearest,
            non_color_max_value: 5,
            vertical: false,
        };

        let mut i_ctx = gvox_ctx
            .get_adapter::<gvox_rs::Input, gvox_rs::adapters::ByteBuffer>()
            .expect("Failed to get byte buffer input adapter.")
            .create_adapter_context(BYTES)
            .expect("Failed to create adapter context.");

        let mut o_ctx = gvox_ctx
            .get_adapter::<gvox_rs::Output, gvox_rs::adapters::ByteBuffer>()
            .expect("Failed to get byte buffer input adapter.")
            .create_adapter_context(o_config)
            .expect("Failed to create adapter context.");

        let mut p_ctx = gvox_ctx
            .get_adapter::<gvox_rs::Parse, procedural_parse::Procedural>()
            .expect("Failed to get byte buffer input adapter.")
            .create_adapter_context(())
            .expect("Failed to create adapter context.");

        let mut s_ctx = gvox_ctx
            .get_adapter::<gvox_rs::Serialize, gvox_rs::adapters::ColoredText>()
            .expect("Failed to get byte buffer input adapter.")
            .create_adapter_context(s_config)
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
            Some(&region),
            gvox_rs::ChannelId::COLOR
                | gvox_rs::ChannelId::NORMAL
                | gvox_rs::ChannelId::MATERIAL_ID,
        )
        .expect("Error while translating.");
    }

    assert_eq!(
        22228,
        o_buffer.len(),
        "Buffer output length did not match expected."
    );
    println!(
        "{}",
        std::str::from_utf8(&o_buffer).expect("Bad string slice.")
    );
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
        vertical: false,
    };

    let mut i_ctx = gvox_ctx
        .get_adapter::<gvox_rs::Input, gvox_rs::adapters::ByteBuffer>()
        .expect("Failed to get byte buffer input adapter.")
        .create_adapter_context(BYTES)
        .expect("Failed to create adapter context.");

    let mut o_ctx = gvox_ctx
        .get_adapter::<gvox_rs::Output, gvox_rs::adapters::ByteBuffer>()
        .expect("Failed to get byte buffer input adapter.")
        .create_adapter_context(o_config)
        .expect("Failed to create adapter context.");

    let mut p_ctx = gvox_ctx
        .get_adapter::<gvox_rs::Parse, gvox_rs::adapters::GvoxPalette>()
        .expect("Failed to get byte buffer input adapter.")
        .create_adapter_context(())
        .expect("Failed to create adapter context.");

    let mut s_ctx = gvox_ctx
        .get_adapter::<gvox_rs::Serialize, gvox_rs::adapters::ColoredText>()
        .expect("Failed to get byte buffer input adapter.")
        .create_adapter_context(s_config)
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
        Some(&region),
        gvox_rs::ChannelId::TRANSPARENCY.into(),
    )
    .map_err(|e| e.error_type());

    assert!(matches!(
        res,
        Err(gvox_rs::ErrorType::ParseAdapterRequestedChannelNotPresent)
    ));
}

pub struct CustomAdapter;

impl gvox_rs::AdapterDescriptor<gvox_rs::Input> for CustomAdapter {
    type Configuration<'a> = ();
    type Handler = Self;
}

impl gvox_rs::NamedAdapter for CustomAdapter {
    fn name() -> &'static str {
        "palette_gvox_input_adapter"
    }
}

impl gvox_rs::BaseAdapterHandler<gvox_rs::Input, Self> for CustomAdapter {
    fn create(config: &()) -> Result<Self, gvox_rs::GvoxError> {
        Ok(Self)
    }

    fn destroy(self) -> Result<(), gvox_rs::GvoxError> {
        Ok(())
    }
}

impl gvox_rs::InputAdapterHandler<CustomAdapter> for CustomAdapter {
    fn read(
        &mut self,
        blit_ctx: &gvox_rs::InputBlitContext,
        position: usize,
        data: &mut [u8],
    ) -> Result<(), gvox_rs::GvoxError> {
        if position + data.len() <= BYTES.len() {
            data.clone_from_slice(&BYTES[position..position + data.len()]);
            Ok(())
        } else {
            Err(gvox_rs::GvoxError::new(
                gvox_rs::ErrorType::InputAdapter,
                "Tried reading past the end of the provided input buffer.",
            ))
        }
    }
}

#[test]
pub fn gvox_rs_test_rust_adapter() {
    let mut o_buffer = Box::default();

    {
        let gvox_ctx = gvox_rs::Context::new();
        gvox_ctx.register_adapter::<gvox_rs::Input, CustomAdapter>();

        let o_config = gvox_rs::adapters::ByteBufferOutputAdapterConfig::from(&mut o_buffer);

        let s_config = gvox_rs::adapters::ColoredTextSerializeAdapterConfig {
            downscale_factor: 1,
            downscale_mode: gvox_rs::adapters::ColoredTextSerializeAdapterDownscaleMode::Nearest,
            non_color_max_value: 5,
            vertical: false,
        };

        let mut i_ctx = gvox_ctx
            .get_adapter::<gvox_rs::Input, CustomAdapter>()
            .expect("Failed to get custom input adapter.")
            .create_adapter_context(())
            .expect("Failed to create adapter context.");

        let mut o_ctx = gvox_ctx
            .get_adapter::<gvox_rs::Output, gvox_rs::adapters::ByteBuffer>()
            .expect("Failed to get byte buffer input adapter.")
            .create_adapter_context(o_config)
            .expect("Failed to create adapter context.");

        let mut p_ctx = gvox_ctx
            .get_adapter::<gvox_rs::Parse, gvox_rs::adapters::GvoxPalette>()
            .expect("Failed to get byte buffer input adapter.")
            .create_adapter_context(())
            .expect("Failed to create adapter context.");

        let mut s_ctx = gvox_ctx
            .get_adapter::<gvox_rs::Serialize, gvox_rs::adapters::ColoredText>()
            .expect("Failed to get byte buffer input adapter.")
            .create_adapter_context(s_config)
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
            Some(&region),
            gvox_rs::ChannelId::COLOR
                | gvox_rs::ChannelId::NORMAL
                | gvox_rs::ChannelId::MATERIAL_ID,
        )
        .expect("Error while translating.");
    }

    assert_eq!(
        22228,
        o_buffer.len(),
        "Buffer output length did not match expected."
    );
}
