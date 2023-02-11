use std::ptr::null_mut;

use crate as gvox_rs;
mod procedural_parse;

#[test]
fn gvox_rs_test_procedural() {
    let gvox_ctx = gvox_rs::Context::new();
    let cstring =
        std::ffi::CString::new("procedural").expect("Failed to convert Rust string to C string");
    let str_cstr = cstring.as_ptr();
    let procedural_adapter_info = gvox_rs::ParseAdapterInfo {
        name_str: str_cstr,
        begin: Some(procedural_parse::begin),
        end: Some(procedural_parse::end),
        query_region_flags: Some(procedural_parse::query_region_flags),
        load_region: Some(procedural_parse::load_region),
        unload_region: Some(procedural_parse::unload_region),
        sample_region: Some(procedural_parse::sample_region),
    };
    unsafe { gvox_sys::gvox_register_parse_adapter(gvox_ctx.ptr, &procedural_adapter_info) };
    // let i_adapter = gvox_ctx.get_input_adapter("byte_buffer");
    let o_adapter = gvox_ctx.get_output_adapter("byte_buffer");
    let p_adapter = gvox_ctx.get_parse_adapter("procedural");
    let s_adapter = gvox_ctx.get_serialize_adapter("colored_text");
    // let bytes = include_bytes!("test.gvox");
    // let mut i_config = gvox_sys::GvoxByteBufferInputAdapterConfig {
    //     size: bytes.len(),
    //     data: bytes.as_ptr(),
    // };
    let mut o_buffer_size: usize = 0;
    let mut o_buffer_ptr: *mut u8 = null_mut();
    let mut o_config = gvox_sys::GvoxByteBufferOutputAdapterConfig {
        out_size: (&mut o_buffer_size) as *mut usize,
        out_byte_buffer_ptr: (&mut o_buffer_ptr) as *mut *mut u8,
        allocate: None,
    };
    let mut s_config = gvox_sys::GvoxColoredTextSerializeAdapterConfig{
        downscale_factor: 1,
        downscale_mode: gvox_sys::GvoxColoredTextSerializeAdapterDownscaleMode_GVOX_COLORED_TEXT_SERIALIZE_ADAPTER_DOWNSCALE_MODE_NEAREST,
        channel_id: gvox_sys::GVOX_CHANNEL_ID_COLOR,
        non_color_max_value: 0,
    };
    let adapter_ctx = gvox_ctx
        .create_adapter_context(
            None,
            null_mut(),
            // Some(i_adapter),
            // &mut i_config as *mut gvox_sys::GvoxByteBufferInputAdapterConfig
            //     as *mut std::os::raw::c_void,
            Some(o_adapter),
            &mut o_config as *mut gvox_sys::GvoxByteBufferOutputAdapterConfig
                as *mut std::os::raw::c_void,
            Some(p_adapter),
            null_mut(),
            Some(s_adapter),
            &mut s_config as *mut gvox_sys::GvoxColoredTextSerializeAdapterConfig
                as *mut std::os::raw::c_void,
        )
        .expect("Failed to create adapter context");
    adapter_ctx.gvox_translate_region(
        &gvox_rs::RegionRange {
            offset: gvox_rs::Offset3D {
                x: -4,
                y: -4,
                z: -4,
            },
            extent: gvox_rs::Extent3D { x: 8, y: 8, z: 8 },
        },
        gvox_sys::GVOX_CHANNEL_BIT_COLOR,
    );
    match gvox_ctx.get_error() {
        Ok(_) => {}
        Err(e) => println!("Error while translating: {}", e),
    }
    let s = unsafe {
        let slice = std::slice::from_raw_parts(o_buffer_ptr, o_buffer_size);
        std::str::from_utf8(slice).expect("bad string slice")
    };
    println!("ptr: {}, size: {}", o_buffer_ptr as usize, o_buffer_size);
    println!("{}", s);
}
