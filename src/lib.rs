mod gvox {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    pub struct Context {
        ctx: *mut GVoxContext,
    }

    pub type Scene = GVoxScene;

    fn path_to_buf(path: &std::path::Path) -> Vec<u8> {
        let mut buf = Vec::new();

        buf.extend(path.to_string_lossy().as_bytes());
        buf.push(0);

        buf
    }

    impl Context {
        pub fn new() -> Self {
            let ctx = unsafe { gvox_create_context() };
            Context { ctx }
        }

        pub fn push_root_path(&self, path: &std::path::Path) {
            let path_buf = path_to_buf(path);
            let path_cstr = path_buf.as_ptr() as *const std::os::raw::c_char;
            unsafe { gvox_push_root_path(self.ctx, path_cstr) }
        }

        pub fn pop_root_path(&self) {
            unsafe { gvox_pop_root_path(self.ctx) }
        }

        fn get_error(&self) -> String {
            unsafe {
                let mut msg_size: usize = 0;
                gvox_get_result_message(self.ctx, 0 as *mut i8, &mut msg_size);
                let mut buf: Vec<u8> = Vec::new();
                buf.resize(msg_size, 0);
                gvox_get_result_message(self.ctx, buf.as_mut_ptr() as *mut i8, &mut msg_size);
                gvox_pop_result(self.ctx);
                use std::str;
                return match str::from_utf8(buf.as_slice()) {
                    Ok(v) => v.to_string(),
                    Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                };
            }
        }

        pub fn load(&self, path: &std::path::Path) -> Result<Scene, String> {
            let path_buf = path_to_buf(path);
            let path_cstr = path_buf.as_ptr() as *const std::os::raw::c_char;
            let result = unsafe { gvox_load(self.ctx, path_cstr) };

            if unsafe { gvox_get_result(self.ctx) != GVoxResult_GVOX_SUCCESS } {
                return Err(self.get_error());
            }

            Ok(result)
        }

        pub fn load_from_raw(
            &self,
            path: &std::path::Path,
            src_format: &String,
        ) -> Result<Scene, String> {
            let path_buf = path_to_buf(path);
            let path_cstr = path_buf.as_ptr() as *const std::os::raw::c_char;
            let str_cstr = src_format.as_ptr() as *const std::os::raw::c_char;
            let result = unsafe { gvox_load_from_raw(self.ctx, path_cstr, str_cstr) };

            if unsafe { gvox_get_result(self.ctx) != GVoxResult_GVOX_SUCCESS } {
                return Err(self.get_error());
            }

            Ok(result)
        }

        pub fn save(
            &self,
            scene: &Scene,
            path: &std::path::Path,
            dst_format: &String,
        ) -> Result<(), String> {
            let path_buf = path_to_buf(path);
            let path_cstr = path_buf.as_ptr() as *const std::os::raw::c_char;
            let str_cstr = dst_format.as_ptr() as *const std::os::raw::c_char;
            unsafe { gvox_save(self.ctx, *scene, path_cstr, str_cstr) }

            if unsafe { gvox_get_result(self.ctx) != GVoxResult_GVOX_SUCCESS } {
                return Err(self.get_error());
            }

            Ok({})
        }

        pub fn save_as_raw(
            &self,
            scene: &Scene,
            path: &std::path::Path,
            dst_format: &String,
        ) -> Result<(), String> {
            let path_buf = path_to_buf(path);
            let path_cstr = path_buf.as_ptr() as *const std::os::raw::c_char;
            let str_cstr = dst_format.as_ptr() as *const std::os::raw::c_char;
            unsafe { gvox_save_as_raw(self.ctx, *scene, path_cstr, str_cstr) }

            if unsafe { gvox_get_result(self.ctx) != GVoxResult_GVOX_SUCCESS } {
                return Err(self.get_error());
            }

            Ok({})
        }

        pub fn destroy_scene(&self, scene: Scene) {
            unsafe {
                gvox_destroy_scene(scene);
            }
        }
    }

    impl Drop for Context {
        fn drop(&mut self) {
            unsafe { gvox_destroy_context(self.ctx) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gvox_rs_test() {
        let gvox_ctx = gvox::Context::new();
        gvox_ctx.push_root_path(std::path::Path::new("prebuilt"));
        let scene = gvox_ctx.load(std::path::Path::new("scene.gvox"));
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
                    let voxel_n = node.size_x * node.size_y * node.size_z;

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
