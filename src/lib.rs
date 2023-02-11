#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(test)]
mod tests;

use gvox_sys;

pub struct Context {
    ptr: *mut gvox_sys::GvoxContext,
}
pub struct AdapterContext {
    ptr: *mut gvox_sys::GvoxAdapterContext,
}

pub type InputAdapterInfo = gvox_sys::GvoxInputAdapterInfo;
pub type InputAdapter = *mut gvox_sys::GvoxInputAdapter;
pub type OutputAdapterInfo = gvox_sys::GvoxOutputAdapterInfo;
pub type OutputAdapter = *mut gvox_sys::GvoxOutputAdapter;
pub type ParseAdapterInfo = gvox_sys::GvoxParseAdapterInfo;
pub type ParseAdapter = *mut gvox_sys::GvoxParseAdapter;
pub type SerializeAdapterInfo = gvox_sys::GvoxSerializeAdapterInfo;
pub type SerializeAdapter = *mut gvox_sys::GvoxSerializeAdapter;

pub type Offset3D = gvox_sys::GvoxOffset3D;
pub type Extent3D = gvox_sys::GvoxExtent3D;
pub type RegionRange = gvox_sys::GvoxRegionRange;
pub type Region = gvox_sys::GvoxRegion;

impl Context {
    pub fn new() -> Self {
        let ctx = unsafe { gvox_sys::gvox_create_context() };
        Context { ptr: ctx }
    }

    fn get_error(&self) -> String {
        unsafe {
            let mut msg_size: usize = 0;
            gvox_sys::gvox_get_result_message(self.ptr, 0 as *mut i8, &mut msg_size);
            let mut buf: Vec<u8> = Vec::new();
            buf.resize(msg_size, 0);
            gvox_sys::gvox_get_result_message(self.ptr, buf.as_mut_ptr() as *mut i8, &mut msg_size);
            gvox_sys::gvox_pop_result(self.ptr);
            use std::str;
            match str::from_utf8(buf.as_slice()) {
                Ok(v) => v.to_string(),
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            }
        }
    }

    pub fn register_input_adapter(&self, adapter_info: &InputAdapterInfo) -> InputAdapter {
        unsafe { gvox_sys::gvox_register_input_adapter(self.ptr, adapter_info) }
    }
    pub fn get_input_adapter(&self, adapter_name: &str) -> InputAdapter {
        let cstring = std::ffi::CString::new(adapter_name)
            .expect("Failed to convert Rust string to C string");
        let str_cstr = cstring.as_ptr();
        unsafe { gvox_sys::gvox_get_input_adapter(self.ptr, str_cstr) }
    }
    pub fn register_output_adapter(&self, adapter_info: &OutputAdapterInfo) -> OutputAdapter {
        unsafe { gvox_sys::gvox_register_output_adapter(self.ptr, adapter_info) }
    }
    pub fn get_output_adapter(&self, adapter_name: &str) -> OutputAdapter {
        let cstring = std::ffi::CString::new(adapter_name)
            .expect("Failed to convert Rust string to C string");
        let str_cstr = cstring.as_ptr();
        unsafe { gvox_sys::gvox_get_output_adapter(self.ptr, str_cstr) }
    }
    pub fn register_parse_adapter(&self, adapter_info: &ParseAdapterInfo) -> ParseAdapter {
        unsafe { gvox_sys::gvox_register_parse_adapter(self.ptr, adapter_info) }
    }
    pub fn get_parse_adapter(&self, adapter_name: &str) -> ParseAdapter {
        let cstring = std::ffi::CString::new(adapter_name)
            .expect("Failed to convert Rust string to C string");
        let str_cstr = cstring.as_ptr();
        unsafe { gvox_sys::gvox_get_parse_adapter(self.ptr, str_cstr) }
    }
    pub fn register_serialize_adapter(
        &self,
        adapter_info: &SerializeAdapterInfo,
    ) -> SerializeAdapter {
        unsafe { gvox_sys::gvox_register_serialize_adapter(self.ptr, adapter_info) }
    }
    pub fn get_serialize_adapter(&self, adapter_name: &str) -> SerializeAdapter {
        let cstring = std::ffi::CString::new(adapter_name)
            .expect("Failed to convert Rust string to C string");
        let str_cstr = cstring.as_ptr();
        unsafe { gvox_sys::gvox_get_serialize_adapter(self.ptr, str_cstr) }
    }

    pub fn create_adapter_context(
        &self,
        input_adapter: Option<InputAdapter>,
        input_config: *mut std::os::raw::c_void,
        output_adapter: Option<OutputAdapter>,
        output_config: *mut std::os::raw::c_void,
        parse_adapter: Option<ParseAdapter>,
        parse_config: *mut std::os::raw::c_void,
        serialize_adapter: Option<SerializeAdapter>,
        serialize_config: *mut std::os::raw::c_void,
    ) -> AdapterContext {
        use std::ptr::null_mut;
        let result = unsafe {
            gvox_sys::gvox_create_adapter_context(
                self.ptr,
                input_adapter.unwrap_or(null_mut() as InputAdapter),
                input_config,
                // (input_config
                //     .as_mut()
                //     .unwrap_or(&mut *(null_mut() as *mut InputConfigT))
                //     as *mut InputConfigT) as *mut std::os::raw::c_void,
                output_adapter.unwrap_or(null_mut() as OutputAdapter),
                output_config,
                // (output_config
                //     .as_mut()
                //     .unwrap_or(&mut *(null_mut() as *mut OutputConfigT))
                //     as *mut OutputConfigT) as *mut std::os::raw::c_void,
                parse_adapter.unwrap_or(null_mut() as ParseAdapter),
                parse_config,
                // (parse_config
                //     .as_mut()
                //     .unwrap_or(&mut *(null_mut() as *mut ParseConfigT))
                //     as *mut ParseConfigT) as *mut std::os::raw::c_void,
                serialize_adapter.unwrap_or(null_mut() as SerializeAdapter),
                serialize_config,
                // (serialize_config
                //     .as_mut()
                //     .unwrap_or(&mut *(null_mut() as *mut SerializeConfigT))
                //     as *mut SerializeConfigT) as *mut std::os::raw::c_void,
            )
        };
        AdapterContext { ptr: result }
    }

    // GvoxRegion gvox_load_region(GvoxAdapterContext *ctx, GvoxOffset3D const *offset, uint32_t channel_id);
    // void gvox_unload_region(GvoxAdapterContext *ctx, GvoxRegion *region);
    // uint32_t gvox_sample_region(GvoxAdapterContext *ctx, GvoxRegion *region, GvoxOffset3D const *offset, uint32_t channel_id);
    // uint32_t gvox_query_region_flags(GvoxAdapterContext *ctx, GvoxRegionRange const *range, uint32_t channel_id);

    // void gvox_adapter_push_error(GvoxAdapterContext *ctx, GvoxResult result_code, char const *message);

    // void gvox_input_adapter_set_user_pointer(GvoxAdapterContext *ctx, void *ptr);
    // void gvox_output_adapter_set_user_pointer(GvoxAdapterContext *ctx, void *ptr);
    // void gvox_parse_adapter_set_user_pointer(GvoxAdapterContext *ctx, void *ptr);
    // void gvox_serialize_adapter_set_user_pointer(GvoxAdapterContext *ctx, void *ptr);

    // void *gvox_input_adapter_get_user_pointer(GvoxAdapterContext *ctx);
    // void *gvox_output_adapter_get_user_pointer(GvoxAdapterContext *ctx);
    // void *gvox_parse_adapter_get_user_pointer(GvoxAdapterContext *ctx);
    // void *gvox_serialize_adapter_get_user_pointer(GvoxAdapterContext *ctx);

    // void gvox_input_read(GvoxAdapterContext *ctx, size_t position, size_t size, void *data);

    // void gvox_output_write(GvoxAdapterContext *ctx, size_t position, size_t size, void const *data);
    // void gvox_output_reserve(GvoxAdapterContext *ctx, size_t size);
}

impl AdapterContext {
    pub fn gvox_translate_region(&self, range: &RegionRange, channel_flags: u32) {
        unsafe { gvox_sys::gvox_translate_region(self.ptr, range, channel_flags) }
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

#[no_mangle]
pub extern "C" fn add(left: i32, right: i32) -> i32 {
    Context::new();
    left + right
}
