use crate as gvox_rs;

#[test]
fn gvox_rs_test() {
    let gvox_ctx = gvox_rs::Context::new();
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
