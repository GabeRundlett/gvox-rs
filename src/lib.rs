#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod ffi {
    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct _GVoxContext {
        _unused: [u8; 0],
    }
    pub type GVoxContext = _GVoxContext;

    pub const GVoxResult_GVOX_SUCCESS: GVoxResult = 0;
    pub const GVoxResult_GVOX_ERROR_FAILED_TO_LOAD_FILE: GVoxResult = -1;
    pub const GVoxResult_GVOX_ERROR_FAILED_TO_LOAD_FORMAT: GVoxResult = -2;
    pub const GVoxResult_GVOX_ERROR_INVALID_FORMAT: GVoxResult = -3;
    pub type GVoxResult = ::std::os::raw::c_int;

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct GVoxVoxel {
        pub color: GVoxVoxel_Color,
        pub id: u32,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct GVoxVoxel_Color {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct GVoxSceneNode {
        pub size_x: usize,
        pub size_y: usize,
        pub size_z: usize,
        pub voxels: *mut GVoxVoxel,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct GVoxScene {
        pub node_n: usize,
        pub nodes: *mut GVoxSceneNode,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct GVoxPayload {
        pub size: usize,
        pub data: *mut u8,
    }

    extern "C" {
        pub fn gvox_create_context() -> *mut GVoxContext;
        pub fn gvox_destroy_context(ctx: *mut GVoxContext);
        pub fn gvox_push_root_path(ctx: *mut GVoxContext, path: *const ::std::os::raw::c_char);
        pub fn gvox_pop_root_path(ctx: *mut GVoxContext);
        pub fn gvox_get_result(ctx: *mut GVoxContext) -> GVoxResult;
        pub fn gvox_get_result_message(
            ctx: *mut GVoxContext,
            str_buffer: *mut ::std::os::raw::c_char,
            str_size: *mut usize,
        );
        pub fn gvox_pop_result(ctx: *mut GVoxContext);
        pub fn gvox_load(
            ctx: *mut GVoxContext,
            filepath: *const ::std::os::raw::c_char,
        ) -> GVoxScene;
        pub fn gvox_load_from_raw(
            ctx: *mut GVoxContext,
            filepath: *const ::std::os::raw::c_char,
            src_format: *const ::std::os::raw::c_char,
        ) -> GVoxScene;
        pub fn gvox_save(
            ctx: *mut GVoxContext,
            scene: GVoxScene,
            filepath: *const ::std::os::raw::c_char,
            dst_format: *const ::std::os::raw::c_char,
        );
        pub fn gvox_save_as_raw(
            ctx: *mut GVoxContext,
            scene: GVoxScene,
            filepath: *const ::std::os::raw::c_char,
            dst_format: *const ::std::os::raw::c_char,
        );
        pub fn gvox_parse(
            ctx: *mut GVoxContext,
            payload: GVoxPayload,
            src_format: *const ::std::os::raw::c_char,
        ) -> GVoxScene;
        pub fn gvox_serialize(
            ctx: *mut GVoxContext,
            scene: GVoxScene,
            dst_format: *const ::std::os::raw::c_char,
        ) -> GVoxPayload;
        pub fn gvox_destroy_scene(scene: GVoxScene);
        pub fn gvox_destroy_payload(
            ctx: *mut GVoxContext,
            payload: GVoxPayload,
            format: *const ::std::os::raw::c_char,
        );
    }
}

pub struct Context {
    ctx: *mut ffi::GVoxContext,
}

pub type Scene = ffi::GVoxScene;

pub type Payload = ffi::GVoxPayload;

fn path_to_buf(path: &std::path::Path) -> Vec<u8> {
    let mut buf = Vec::new();

    buf.extend(path.to_string_lossy().as_bytes());
    buf.push(0);

    buf
}

impl Context {
    pub fn new() -> Self {
        let ctx = unsafe { ffi::gvox_create_context() };
        Context { ctx }
    }

    pub fn push_root_path(&self, path: &std::path::Path) {
        let path_buf = path_to_buf(path);
        let path_cstr = path_buf.as_ptr() as *const std::os::raw::c_char;
        unsafe { ffi::gvox_push_root_path(self.ctx, path_cstr) }
    }

    pub fn pop_root_path(&self) {
        unsafe { ffi::gvox_pop_root_path(self.ctx) }
    }

    fn get_error(&self) -> String {
        unsafe {
            let mut msg_size: usize = 0;
            ffi::gvox_get_result_message(self.ctx, 0 as *mut i8, &mut msg_size);
            let mut buf: Vec<u8> = Vec::new();
            buf.resize(msg_size, 0);
            ffi::gvox_get_result_message(self.ctx, buf.as_mut_ptr() as *mut i8, &mut msg_size);
            ffi::gvox_pop_result(self.ctx);
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
        let result = unsafe { ffi::gvox_load(self.ctx, path_cstr) };

        if unsafe { ffi::gvox_get_result(self.ctx) != ffi::GVoxResult_GVOX_SUCCESS } {
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
        let result = unsafe { ffi::gvox_load_from_raw(self.ctx, path_cstr, str_cstr) };

        if unsafe { ffi::gvox_get_result(self.ctx) != ffi::GVoxResult_GVOX_SUCCESS } {
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
        unsafe { ffi::gvox_save(self.ctx, *scene, path_cstr, str_cstr) }

        if unsafe { ffi::gvox_get_result(self.ctx) != ffi::GVoxResult_GVOX_SUCCESS } {
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
        unsafe { ffi::gvox_save_as_raw(self.ctx, *scene, path_cstr, str_cstr) }

        if unsafe { ffi::gvox_get_result(self.ctx) != ffi::GVoxResult_GVOX_SUCCESS } {
            Err(self.get_error())
        } else {
            Ok({})
        }
    }

    pub fn parse(&self, payload: &Payload, src_format: &str) -> Result<Scene, String> {
        let cstring =
            std::ffi::CString::new(src_format).expect("Failed to convert Rust string to C string");
        let str_cstr = cstring.as_ptr();
        let result = unsafe { ffi::gvox_parse(self.ctx, *payload, str_cstr) };

        if unsafe { ffi::gvox_get_result(self.ctx) != ffi::GVoxResult_GVOX_SUCCESS } {
            Err(self.get_error())
        } else {
            Ok(result)
        }
    }
    pub fn serialize(&self, scene: &Scene, dst_format: &str) -> Result<Payload, String> {
        let cstring =
            std::ffi::CString::new(dst_format).expect("Failed to convert Rust string to C string");
        let str_cstr = cstring.as_ptr();
        let result = unsafe { ffi::gvox_serialize(self.ctx, *scene, str_cstr) };

        if unsafe { ffi::gvox_get_result(self.ctx) != ffi::GVoxResult_GVOX_SUCCESS } {
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
            ffi::gvox_destroy_payload(self.ctx, payload, str_cstr);
        }
    }

    pub fn destroy_scene(&self, scene: Scene) {
        unsafe {
            ffi::gvox_destroy_scene(scene);
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { ffi::gvox_destroy_context(self.ctx) }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn gvox_rs_test() {
        let gvox_ctx = super::Context::new();
        gvox_ctx.push_root_path(std::path::Path::new("prebuilt"));
        let scene = gvox_ctx.load_from_raw(
            std::path::Path::new("scene.gvox"),
            "ace_of_spades",
        );
        if scene.is_err() {
            println!("ERROR LOADING FILE: {}", scene.unwrap_err());
        } else {
            let scene = scene.unwrap();
            println!("node count: {}", scene.node_n);
            unsafe {
                for node_i in 0..scene.node_n {
                    let node = *(scene.nodes.add(node_i));

                    println!(
                        "node {} size: {}, {}, {}",
                        node_i, node.size_x, node.size_y, node.size_z
                    );

                    for zi in 0..node.size_z {
                        for yi in 0..node.size_y {
                            for xi in 0..node.size_x {
                                let voxel_i = xi
                                    + yi * node.size_x
                                    + (node.size_z - 1 - zi) * node.size_x * node.size_y;
                                let voxel = *(node.voxels.add(voxel_i));
                                print!(
                                    "\x1b[38;2;{0:03};{1:03};{2:03}m\x1b[48;2;{0:03};{1:03};{2:03}m__",
                                    (voxel.color.x * 255.0) as u32,
                                    (voxel.color.y * 255.0) as u32,
                                    (voxel.color.z * 255.0) as u32
                                );
                            }
                            print!("\x1b[0m ");
                        }
                        print!("\x1b[0m\n");
                    }
                }
                gvox_ctx.destroy_scene(scene);
            }
        }
    }
}
