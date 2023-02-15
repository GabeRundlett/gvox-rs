use std::ptr::null_mut;

use crate::{self as gvox_rs};

fn stable_rand(x: f32) -> f32 {
    ((x * 91.3458).sin() * 47453.5453) % 1.0
}
fn stable_rand_2(x: f32, y: f32) -> f32 {
    ((x * 12.9898 + y * 78.233).sin() * 47453.5453) % 1.0
}
fn stable_rand_3(x: f32, y: f32, z: f32) -> f32 {
    stable_rand_2(x + stable_rand(z), y + stable_rand(z))
}
fn stable_rand_3i(xi: i32, yi: i32, zi: i32) -> f32 {
    let x = (xi as f32 + 0.5) * (1.0 / 8.0);
    let y = (yi as f32 + 0.5) * (1.0 / 8.0);
    let z = (zi as f32 + 0.5) * (1.0 / 8.0);
    stable_rand_3(x, y, z)
}

fn sample_terrain(x: f32, y: f32, z: f32) -> f32 {
    return -(x * x + y * y + z * z) + 0.25;
}

fn sample_terrain_i(xi: i32, yi: i32, zi: i32) -> f32 {
    let x = (xi as f32 + 0.5) * (1.0 / 8.0);
    let y = (yi as f32 + 0.5) * (1.0 / 8.0);
    let z = (zi as f32 + 0.5) * (1.0 / 8.0);
    sample_terrain(x, y, z)
}

pub unsafe extern "C" fn create(
    _ctx: *mut gvox_sys::GvoxAdapterContext,
    _config: *const std::os::raw::c_void,
) {
}
pub unsafe extern "C" fn destroy(_ctx: *mut gvox_sys::GvoxAdapterContext) {}
pub unsafe extern "C" fn blit_begin(
    _blit_ctx: *mut gvox_sys::GvoxBlitContext,
    _ctx: *mut gvox_sys::GvoxAdapterContext,
) {
}
pub unsafe extern "C" fn blit_end(
    _blit_ctx: *mut gvox_sys::GvoxBlitContext,
    _ctx: *mut gvox_sys::GvoxAdapterContext,
) {
}

pub unsafe extern "C" fn query_region_flags(
    _blit_ctx: *mut gvox_sys::GvoxBlitContext,
    _ctx: *mut gvox_sys::GvoxAdapterContext,
    _range: *const gvox_sys::GvoxRegionRange,
    _channel_flags: u32,
) -> u32 {
    0
}

fn create_color(rf: f32, gf: f32, bf: f32, a: u32) -> u32 {
    let r = (rf.min(1.0).max(0.0) * 255.0) as u32;
    let g = (gf.min(1.0).max(0.0) * 255.0) as u32;
    let b = (bf.min(1.0).max(0.0) * 255.0) as u32;
    (r << 0x00) | (g << 0x08) | (b << 0x10) | (a << 0x18)
}

fn create_normal(xf: f32, yf: f32, zf: f32) -> u32 {
    let x = ((xf * 0.5 + 0.5).min(1.0).max(0.0) * 255.0) as u32;
    let y = ((yf * 0.5 + 0.5).min(1.0).max(0.0) * 255.0) as u32;
    let z = ((zf * 0.5 + 0.5).min(1.0).max(0.0) * 255.0) as u32;
    let w = 0;
    (x << 0x00) | (y << 0x08) | (z << 0x10) | (w << 0x18)
}

macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0") as *const str as *const [std::os::raw::c_char]
            as *const std::os::raw::c_char
    };
}

pub unsafe extern "C" fn load_region(
    _blit_ctx: *mut gvox_sys::GvoxBlitContext,
    ctx: *mut gvox_sys::GvoxAdapterContext,
    range: *const gvox_sys::GvoxRegionRange,
    channel_flags: u32,
) -> gvox_sys::GvoxRegion {
    let available_channels =
        gvox_rs::CHANNEL_BIT_COLOR | gvox_rs::CHANNEL_BIT_NORMAL | gvox_rs::CHANNEL_BIT_MATERIAL_ID;
    if (channel_flags & !available_channels) != 0 {
        // gvox_sys::gvox_adapter_push_error(
        //     ctx,
        //     gvox_rs::RESULT_ERROR_PARSE_ADAPTER_REQUESTED_CHANNEL_NOT_PRESENT,
        //     cstr!("procedural 'parser' does not generate anything other than color & normal"),
        // );
    }
    let region = gvox_rs::Region {
        range: *range,
        channels: channel_flags & available_channels,
        flags: 0,
        data: null_mut(),
    };
    return region;
}

pub unsafe extern "C" fn unload_region(
    _blit_ctx: *mut gvox_sys::GvoxBlitContext,
    _ctx: *mut gvox_sys::GvoxAdapterContext,
    _region: *mut gvox_sys::GvoxRegion,
) {
}

pub unsafe extern "C" fn sample_region(
    _blit_ctx: *mut gvox_sys::GvoxBlitContext,
    _ctx: *mut gvox_sys::GvoxAdapterContext,
    _region: *const gvox_sys::GvoxRegion,
    offset: *const gvox_sys::GvoxOffset3D,
    channel_id: u32,
) -> u32 {
    let val = sample_terrain_i((*offset).x, (*offset).y, (*offset).z);
    let mut color = create_color(0.6, 0.7, 0.9, 0);
    let mut normal = create_normal(0.0, 0.0, 0.0);
    let mut id = 0;
    if val > 0.0 {
        {
            let nx_val = sample_terrain_i((*offset).x - 1, (*offset).y, (*offset).z);
            let ny_val = sample_terrain_i((*offset).x, (*offset).y - 1, (*offset).z);
            let nz_val = sample_terrain_i((*offset).x, (*offset).y, (*offset).z - 1);
            let px_val = sample_terrain_i((*offset).x + 1, (*offset).y, (*offset).z);
            let py_val = sample_terrain_i((*offset).x, (*offset).y + 1, (*offset).z);
            let pz_val = sample_terrain_i((*offset).x, (*offset).y, (*offset).z + 1);
            if nx_val < 0.0
                || ny_val < 0.0
                || nz_val < 0.0
                || px_val < 0.0
                || py_val < 0.0
                || pz_val < 0.0
            {
                let nx = px_val - val;
                let ny = py_val - val;
                let nz = pz_val - val;
                let inv_mag = 1.0 / (nx * nx + ny * ny + nz * nz).sqrt();
                normal = create_normal(nx * inv_mag, ny * inv_mag, nz * inv_mag);
            }
        }
        let mut si = 0;
        for _ in 0..16 {
            let sval = sample_terrain_i((*offset).x, (*offset).y, (*offset).z + si);
            if sval < 0.0 {
                break;
            }
            si += 1;
        }
        if si < 2 {
            color = create_color(0.2, 0.5, 0.1, 1);
            id = 1;
        } else if si < 4 {
            color = create_color(0.4, 0.3, 0.2, 1);
            id = 2;
        } else {
            let r = stable_rand_3i((*offset).x, (*offset).y, (*offset).z);
            if r < 0.5 {
                color = create_color(0.36, 0.34, 0.34, 1);
            } else {
                color = create_color(0.25, 0.24, 0.23, 1);
            }
            id = 3;
        }
    }

    if channel_id == gvox_sys::GVOX_CHANNEL_ID_COLOR {
        return color;
    } else if channel_id == gvox_sys::GVOX_CHANNEL_ID_NORMAL {
        return normal;
    } else if channel_id == gvox_sys::GVOX_CHANNEL_ID_MATERIAL_ID {
        return id;
    } else {
        return 0;
    }
}
