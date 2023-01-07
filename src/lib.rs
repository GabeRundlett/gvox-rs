#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(test)]
mod tests;

pub use gvox_sys;

pub struct Context {
    ctx: *mut gvox_sys::GVoxContext,
}

pub type Scene = gvox_sys::GVoxScene;

pub type Payload = gvox_sys::GVoxPayload;

fn path_to_buf(path: &std::path::Path) -> Vec<u8> {
    let mut buf = Vec::new();

    buf.extend(path.to_string_lossy().as_bytes());
    buf.push(0);

    buf
}

impl Context {
    pub fn new() -> Self {
        let ctx = unsafe { gvox_sys::gvox_create_context() };
        Context { ctx }
    }

    pub fn push_root_path(&self, path: &std::path::Path) {
        let path_buf = path_to_buf(path);
        let path_cstr = path_buf.as_ptr() as *const std::os::raw::c_char;
        unsafe { gvox_sys::gvox_push_root_path(self.ctx, path_cstr) }
    }

    pub fn pop_root_path(&self) {
        unsafe { gvox_sys::gvox_pop_root_path(self.ctx) }
    }

    fn get_error(&self) -> String {
        unsafe {
            let mut msg_size: usize = 0;
            gvox_sys::gvox_get_result_message(self.ctx, 0 as *mut i8, &mut msg_size);
            let mut buf: Vec<u8> = Vec::new();
            buf.resize(msg_size, 0);
            gvox_sys::gvox_get_result_message(self.ctx, buf.as_mut_ptr() as *mut i8, &mut msg_size);
            gvox_sys::gvox_pop_result(self.ctx);
            use std::str;
            match str::from_utf8(buf.as_slice()) {
                Ok(v) => v.to_string(),
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            }
        }
    }

    pub fn load(&self, path: &std::path::Path) -> Result<Scene, String> {
        let path_buf = path_to_buf(path);
        let path_cstr = path_buf.as_ptr() as *const std::os::raw::c_char;
        let result = unsafe { gvox_sys::gvox_load(self.ctx, path_cstr) };

        if unsafe { gvox_sys::gvox_get_result(self.ctx) != gvox_sys::GVoxResult_GVOX_SUCCESS } {
            Err(self.get_error())
        } else {
            Ok(result)
        }
    }

    pub fn load_from_raw(&self, path: &std::path::Path, src_format: &str) -> Result<Scene, String> {
        let path_buf = path_to_buf(path);
        let path_cstr = path_buf.as_ptr() as *const std::os::raw::c_char;
        let cstring =
            std::ffi::CString::new(src_format).expect("Failed to convert Rust string to C string");
        let str_cstr = cstring.as_ptr();
        let result = unsafe { gvox_sys::gvox_load_from_raw(self.ctx, path_cstr, str_cstr) };

        if unsafe { gvox_sys::gvox_get_result(self.ctx) != gvox_sys::GVoxResult_GVOX_SUCCESS } {
            Err(self.get_error())
        } else {
            Ok(result)
        }
    }

    pub fn save(
        &self,
        scene: &Scene,
        path: &std::path::Path,
        dst_format: &str,
    ) -> Result<(), String> {
        let path_buf = path_to_buf(path);
        let path_cstr = path_buf.as_ptr() as *const std::os::raw::c_char;
        let cstring =
            std::ffi::CString::new(dst_format).expect("Failed to convert Rust string to C string");
        let str_cstr = cstring.as_ptr();
        unsafe { gvox_sys::gvox_save(self.ctx, *scene, path_cstr, str_cstr) }

        if unsafe { gvox_sys::gvox_get_result(self.ctx) != gvox_sys::GVoxResult_GVOX_SUCCESS } {
            Err(self.get_error())
        } else {
            Ok({})
        }
    }

    pub fn save_as_raw(
        &self,
        scene: &Scene,
        path: &std::path::Path,
        dst_format: &str,
    ) -> Result<(), String> {
        let path_buf = path_to_buf(path);
        let path_cstr = path_buf.as_ptr() as *const std::os::raw::c_char;
        let cstring =
            std::ffi::CString::new(dst_format).expect("Failed to convert Rust string to C string");
        let str_cstr = cstring.as_ptr();
        unsafe { gvox_sys::gvox_save_as_raw(self.ctx, *scene, path_cstr, str_cstr) }

        if unsafe { gvox_sys::gvox_get_result(self.ctx) != gvox_sys::GVoxResult_GVOX_SUCCESS } {
            Err(self.get_error())
        } else {
            Ok({})
        }
    }

    pub fn parse(&self, payload: &Payload, src_format: &str) -> Result<Scene, String> {
        let cstring =
            std::ffi::CString::new(src_format).expect("Failed to convert Rust string to C string");
        let str_cstr = cstring.as_ptr();
        let result = unsafe { gvox_sys::gvox_parse(self.ctx, *payload, str_cstr) };

        if unsafe { gvox_sys::gvox_get_result(self.ctx) != gvox_sys::GVoxResult_GVOX_SUCCESS } {
            Err(self.get_error())
        } else {
            Ok(result)
        }
    }
    pub fn serialize(&self, scene: &Scene, dst_format: &str) -> Result<Payload, String> {
        let cstring =
            std::ffi::CString::new(dst_format).expect("Failed to convert Rust string to C string");
        let str_cstr = cstring.as_ptr();
        let result = unsafe { gvox_sys::gvox_serialize(self.ctx, *scene, str_cstr) };

        if unsafe { gvox_sys::gvox_get_result(self.ctx) != gvox_sys::GVoxResult_GVOX_SUCCESS } {
            Err(self.get_error())
        } else {
            Ok(result)
        }
    }

    pub fn destroy_payload(&self, payload: Payload, format: &str) {
        let cstring =
            std::ffi::CString::new(format).expect("Failed to convert Rust string to C string");
        let str_cstr = cstring.as_ptr();
        unsafe {
            gvox_sys::gvox_destroy_payload(self.ctx, payload, str_cstr);
        }
    }

    pub fn destroy_scene(&self, scene: Scene) {
        unsafe {
            gvox_sys::gvox_destroy_scene(scene);
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { gvox_sys::gvox_destroy_context(self.ctx) }
    }
}

#[no_mangle]
pub extern "C" fn add(left: i32, right: i32) -> i32 {
    Context::new();
    left + right
}
