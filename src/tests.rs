use std::ptr::{null, null_mut};

use crate::{self as gvox_rs};
mod procedural_parse;

macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0") as *const str as *const [std::os::raw::c_char]
            as *const std::os::raw::c_char
    };
}

#[test]
fn gvox_rs_test_procedural() {
    let gvox_ctx = gvox_rs::Context::new();
    let procedural_adapter_info = gvox_rs::ParseAdapterInfo {
        base_info: gvox_rs::AdapterBaseInfo {
            name_str: cstr!("procedural"),
            create: Some(procedural_parse::create),
            destroy: Some(procedural_parse::destroy),
            blit_begin: Some(procedural_parse::blit_begin),
            blit_end: Some(procedural_parse::blit_end),
        },
        query_region_flags: Some(procedural_parse::query_region_flags),
        load_region: Some(procedural_parse::load_region),
        unload_region: Some(procedural_parse::unload_region),
        sample_region: Some(procedural_parse::sample_region),
    };
    gvox_ctx.register_parse_adapter(&procedural_adapter_info);
    const BYTES: &[u8] = include_bytes!("palette.gvox");
    let i_config = gvox_rs::InputAdapterConfigs::ByteBuffer {
        size: BYTES.len(),
        data: BYTES.as_ptr(),
    };
    let mut o_buffer_size: usize = 0;
    let mut o_buffer_ptr: *mut u8 = null_mut();
    let o_config = gvox_rs::OutputAdapterConfigs::ByteBuffer {
        out_size: (&mut o_buffer_size) as *mut usize,
        out_byte_buffer_ptr: (&mut o_buffer_ptr) as *mut *mut u8,
        allocate: None,
    };
    let s_config = gvox_rs::SerializeAdapterConfigs::ColoredText {
        downscale_factor: 1,
        downscale_mode: gvox_rs::SerializeAdapterConfigs::COLORED_TEXT_DOWNSCALE_MODE_NEAREST,
        non_color_max_value: 5,
    };
    let i_ctx = gvox_ctx
        .create_adapter_context(
            &Some(gvox_ctx.get_input_adapter("byte_buffer")),
            Some(i_config),
        )
        .expect("Failed to create adapter context");
    let o_ctx = gvox_ctx
        .create_adapter_context(
            &Some(gvox_ctx.get_output_adapter("byte_buffer")),
            Some(o_config),
        )
        .expect("Failed to create adapter context");
    let p_ctx = gvox_ctx
        .create_adapter_context::<()>(&Some(gvox_ctx.get_parse_adapter("gvox_palette")), None)
        .expect("Failed to create adapter context");
    let s_ctx = gvox_ctx
        .create_adapter_context(
            &Some(gvox_ctx.get_serialize_adapter("colored_text")),
            Some(s_config),
        )
        .expect("Failed to create adapter context");
    let region = gvox_rs::RegionRange {
        offset: gvox_rs::Offset3D {
            x: -4,
            y: -4,
            z: -4,
        },
        extent: gvox_rs::Extent3D { x: 8, y: 8, z: 8 },
    };
    gvox_rs::blit_region(
        &i_ctx,
        &o_ctx,
        &p_ctx,
        &s_ctx,
        &region,
        gvox_rs::CHANNEL_BIT_COLOR | gvox_rs::CHANNEL_BIT_NORMAL | gvox_rs::CHANNEL_BIT_MATERIAL_ID,
    );
    match gvox_ctx.get_error() {
        Ok(_) => {}
        Err(e) => println!("Error while translating: {}", e),
    }
    let s = unsafe {
        let slice = std::slice::from_raw_parts(o_buffer_ptr, o_buffer_size);
        std::str::from_utf8(slice).expect("bad string slice")
    };
    println!("{}", s);
}
