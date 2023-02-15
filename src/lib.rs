#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(test)]
mod tests;

use gvox_sys;

pub const RESULT_SUCCESS: gvox_sys::GvoxResult = gvox_sys::GvoxResult_GVOX_RESULT_SUCCESS;
pub const RESULT_ERROR_UNKNOWN: gvox_sys::GvoxResult =
    gvox_sys::GvoxResult_GVOX_RESULT_ERROR_UNKNOWN;
pub const RESULT_ERROR_INVALID_PARAMETER: gvox_sys::GvoxResult =
    gvox_sys::GvoxResult_GVOX_RESULT_ERROR_INVALID_PARAMETER;
pub const RESULT_ERROR_INPUT_ADAPTER: gvox_sys::GvoxResult =
    gvox_sys::GvoxResult_GVOX_RESULT_ERROR_INPUT_ADAPTER;
pub const RESULT_ERROR_OUTPUT_ADAPTER: gvox_sys::GvoxResult =
    gvox_sys::GvoxResult_GVOX_RESULT_ERROR_OUTPUT_ADAPTER;
pub const RESULT_ERROR_PARSE_ADAPTER: gvox_sys::GvoxResult =
    gvox_sys::GvoxResult_GVOX_RESULT_ERROR_PARSE_ADAPTER;
pub const RESULT_ERROR_SERIALIZE_ADAPTER: gvox_sys::GvoxResult =
    gvox_sys::GvoxResult_GVOX_RESULT_ERROR_SERIALIZE_ADAPTER;
pub const RESULT_ERROR_PARSE_ADAPTER_INVALID_INPUT: gvox_sys::GvoxResult =
    gvox_sys::GvoxResult_GVOX_RESULT_ERROR_PARSE_ADAPTER_INVALID_INPUT;
pub const RESULT_ERROR_PARSE_ADAPTER_REQUESTED_CHANNEL_NOT_PRESENT: gvox_sys::GvoxResult =
    gvox_sys::GvoxResult_GVOX_RESULT_ERROR_PARSE_ADAPTER_REQUESTED_CHANNEL_NOT_PRESENT;
pub const RESULT_ERROR_SERIALIZE_ADAPTER_UNREPRESENTABLE_DATA: gvox_sys::GvoxResult =
    gvox_sys::GvoxResult_GVOX_RESULT_ERROR_SERIALIZE_ADAPTER_UNREPRESENTABLE_DATA;

pub const CHANNEL_ID_COLOR: u32 = gvox_sys::GVOX_CHANNEL_ID_COLOR;
pub const CHANNEL_ID_NORMAL: u32 = gvox_sys::GVOX_CHANNEL_ID_NORMAL;
pub const CHANNEL_ID_MATERIAL_ID: u32 = gvox_sys::GVOX_CHANNEL_ID_MATERIAL_ID;
pub const CHANNEL_ID_ROUGHNESS: u32 = gvox_sys::GVOX_CHANNEL_ID_ROUGHNESS;
pub const CHANNEL_ID_METALNESS: u32 = gvox_sys::GVOX_CHANNEL_ID_METALNESS;
pub const CHANNEL_ID_TRANSPARENCY: u32 = gvox_sys::GVOX_CHANNEL_ID_TRANSPARENCY;
pub const CHANNEL_ID_IOR: u32 = gvox_sys::GVOX_CHANNEL_ID_IOR;
pub const CHANNEL_ID_EMISSIVE_COLOR: u32 = gvox_sys::GVOX_CHANNEL_ID_EMISSIVITY;
pub const CHANNEL_ID_HARDNESS: u32 = gvox_sys::GVOX_CHANNEL_ID_HARDNESS;
pub const CHANNEL_ID_LAST_STANDARD: u32 = gvox_sys::GVOX_CHANNEL_ID_LAST_STANDARD;
pub const CHANNEL_ID_LAST: u32 = gvox_sys::GVOX_CHANNEL_ID_LAST;
pub const CHANNEL_BIT_COLOR: u32 = gvox_sys::GVOX_CHANNEL_BIT_COLOR;
pub const CHANNEL_BIT_NORMAL: u32 = gvox_sys::GVOX_CHANNEL_BIT_NORMAL;
pub const CHANNEL_BIT_MATERIAL_ID: u32 = gvox_sys::GVOX_CHANNEL_BIT_MATERIAL_ID;
pub const CHANNEL_BIT_ROUGHNESS: u32 = gvox_sys::GVOX_CHANNEL_BIT_ROUGHNESS;
pub const CHANNEL_BIT_METALNESS: u32 = gvox_sys::GVOX_CHANNEL_BIT_METALNESS;
pub const CHANNEL_BIT_TRANSPARENCY: u32 = gvox_sys::GVOX_CHANNEL_BIT_TRANSPARENCY;
pub const CHANNEL_BIT_IOR: u32 = gvox_sys::GVOX_CHANNEL_BIT_IOR;
pub const CHANNEL_BIT_EMISSIVE_COLOR: u32 = gvox_sys::GVOX_CHANNEL_BIT_EMISSIVITY;
pub const CHANNEL_BIT_HARDNESS: u32 = gvox_sys::GVOX_CHANNEL_BIT_HARDNESS;
pub const CHANNEL_BIT_LAST_STANDARD: u32 = gvox_sys::GVOX_CHANNEL_BIT_LAST_STANDARD;
pub const CHANNEL_BIT_LAST: u32 = gvox_sys::GVOX_CHANNEL_BIT_LAST;

pub const REGION_FLAG_UNIFORM: u32 = gvox_sys::GVOX_REGION_FLAG_UNIFORM;

pub struct Context {
    ptr: *mut gvox_sys::GvoxContext,
}
pub type Adapter = *mut gvox_sys::GvoxAdapter;

pub struct AdapterContext {
    ptr: *mut gvox_sys::GvoxAdapterContext,
}

pub type Offset3D = gvox_sys::GvoxOffset3D;
pub type Extent3D = gvox_sys::GvoxExtent3D;
pub type RegionRange = gvox_sys::GvoxRegionRange;
pub type Region = gvox_sys::GvoxRegion;

pub type AdapterBaseInfo = gvox_sys::GvoxAdapterBaseInfo;
pub type InputAdapterInfo = gvox_sys::GvoxInputAdapterInfo;
pub type OutputAdapterInfo = gvox_sys::GvoxOutputAdapterInfo;
pub type ParseAdapterInfo = gvox_sys::GvoxParseAdapterInfo;
pub type SerializeAdapterInfo = gvox_sys::GvoxSerializeAdapterInfo;

impl Context {
    pub fn new() -> Self {
        let ctx = unsafe { gvox_sys::gvox_create_context() };
        Context { ptr: ctx }
    }
    pub fn get_error(&self) -> ::std::result::Result<(), String> {
        unsafe {
            if gvox_sys::gvox_get_result(self.ptr) != gvox_sys::GvoxResult_GVOX_RESULT_SUCCESS {
                let mut msg_size: usize = 0;
                gvox_sys::gvox_get_result_message(self.ptr, 0 as *mut i8, &mut msg_size);
                let mut buf: Vec<u8> = Vec::new();
                buf.resize(msg_size, 0);
                gvox_sys::gvox_get_result_message(
                    self.ptr,
                    buf.as_mut_ptr() as *mut i8,
                    &mut msg_size,
                );
                gvox_sys::gvox_pop_result(self.ptr);
                use std::str;
                match str::from_utf8(buf.as_slice()) {
                    Ok(v) => Err(v.to_string()),
                    Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                }
            } else {
                Ok(())
            }
        }
    }
    pub fn register_input_adapter(&self, adapter_info: &InputAdapterInfo) -> Adapter {
        unsafe { gvox_sys::gvox_register_input_adapter(self.ptr, adapter_info) }
    }
    pub fn get_input_adapter(&self, adapter_name: &str) -> Adapter {
        let cstring = std::ffi::CString::new(adapter_name)
            .expect("Failed to convert Rust string to C string");
        let str_cstr = cstring.as_ptr();
        unsafe { gvox_sys::gvox_get_input_adapter(self.ptr, str_cstr) }
    }
    pub fn register_output_adapter(&self, adapter_info: &OutputAdapterInfo) -> Adapter {
        unsafe { gvox_sys::gvox_register_output_adapter(self.ptr, adapter_info) }
    }
    pub fn get_output_adapter(&self, adapter_name: &str) -> Adapter {
        let cstring = std::ffi::CString::new(adapter_name)
            .expect("Failed to convert Rust string to C string");
        let str_cstr = cstring.as_ptr();
        unsafe { gvox_sys::gvox_get_output_adapter(self.ptr, str_cstr) }
    }
    pub fn register_parse_adapter(&self, adapter_info: &ParseAdapterInfo) -> Adapter {
        unsafe { gvox_sys::gvox_register_parse_adapter(self.ptr, adapter_info) }
    }
    pub fn get_parse_adapter(&self, adapter_name: &str) -> Adapter {
        let cstring = std::ffi::CString::new(adapter_name)
            .expect("Failed to convert Rust string to C string");
        let str_cstr = cstring.as_ptr();
        unsafe { gvox_sys::gvox_get_parse_adapter(self.ptr, str_cstr) }
    }
    pub fn register_serialize_adapter(&self, adapter_info: &SerializeAdapterInfo) -> Adapter {
        unsafe { gvox_sys::gvox_register_serialize_adapter(self.ptr, adapter_info) }
    }
    pub fn get_serialize_adapter(&self, adapter_name: &str) -> Adapter {
        let cstring = std::ffi::CString::new(adapter_name)
            .expect("Failed to convert Rust string to C string");
        let str_cstr = cstring.as_ptr();
        unsafe { gvox_sys::gvox_get_serialize_adapter(self.ptr, str_cstr) }
    }
    pub fn create_adapter_context<ConfigT>(
        &self,
        adapter: &Option<Adapter>,
        config: Option<ConfigT>,
    ) -> std::result::Result<AdapterContext, String> {
        use std::ptr::null_mut;
        let result = unsafe {
            gvox_sys::gvox_create_adapter_context(
                self.ptr,
                adapter.unwrap_or(null_mut() as Adapter),
                match config {
                    Some(mut c) => (&mut c) as *mut ConfigT as *mut std::os::raw::c_void,
                    None => null_mut(),
                },
            )
        };
        self.get_error()?;
        Ok(AdapterContext { ptr: result })
    }
}

pub fn blit_region(
    input_ctx: &AdapterContext,
    output_ctx: &AdapterContext,
    parse_ctx: &AdapterContext,
    serialize_ctx: &AdapterContext,
    range: &RegionRange,
    channel_flags: u32,
) {
    unsafe {
        gvox_sys::gvox_blit_region(
            input_ctx.ptr,
            output_ctx.ptr,
            parse_ctx.ptr,
            serialize_ctx.ptr,
            range,
            channel_flags,
        );
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { gvox_sys::gvox_destroy_context(self.ptr) }
    }
}

impl Drop for AdapterContext {
    fn drop(&mut self) {
        unsafe { gvox_sys::gvox_destroy_adapter_context(self.ptr) }
    }
}

pub mod InputAdapterConfigs {
    pub type ByteBuffer = gvox_sys::GvoxByteBufferInputAdapterConfig;
}
pub mod OutputAdapterConfigs {
    pub type ByteBuffer = gvox_sys::GvoxByteBufferOutputAdapterConfig;
}
pub mod ParseAdapters {
    pub type Voxlap = gvox_sys::GvoxVoxlapParseAdapterConfig;
}
pub mod SerializeAdapterConfigs {
    pub type ColoredText = gvox_sys::GvoxColoredTextSerializeAdapterConfig;
    pub const COLORED_TEXT_DOWNSCALE_MODE_NEAREST: gvox_sys::GvoxColoredTextSerializeAdapterDownscaleMode =
        gvox_sys::GvoxColoredTextSerializeAdapterDownscaleMode_GVOX_COLORED_TEXT_SERIALIZE_ADAPTER_DOWNSCALE_MODE_NEAREST;
    pub const COLORED_TEXT_DOWNSCALE_MODE_LINEAR: gvox_sys::GvoxColoredTextSerializeAdapterDownscaleMode =
        gvox_sys::GvoxColoredTextSerializeAdapterDownscaleMode_GVOX_COLORED_TEXT_SERIALIZE_ADAPTER_DOWNSCALE_MODE_LINEAR;
}
