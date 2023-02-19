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

pub struct Procedural;

impl gvox_rs::AdapterDescriptor<gvox_rs::Parse> for Procedural {
    type Configuration<'a> = ();
    type Handler = Self;
}

impl gvox_rs::NamedAdapter for Procedural {
    fn name() -> &'static str {
        "procedural"
    }
}

impl gvox_rs::BaseAdapterHandler<gvox_rs::Parse, Self> for Procedural {
    fn create(config: &()) -> Result<Self, gvox_rs::GvoxError> {
        Ok(Self)
    }

    fn destroy(self) -> Result<(), gvox_rs::GvoxError> {
        Ok(())
    }
}

impl gvox_rs::ParseAdapterHandler<Self> for Procedural {
    type RegionData = ();

    fn query_region_flags(
        &mut self,
        blit_ctx: &gvox_rs::ParseBlitContext,
        range: &gvox_rs::RegionRange,
        channel_flags: gvox_rs::ChannelFlags,
    ) -> Result<gvox_rs::RegionFlags, gvox_rs::GvoxError> {
        Ok(gvox_rs::RegionFlags::empty())
    }

    fn load_region(
        &mut self,
        blit_ctx: &gvox_rs::ParseBlitContext,
        range: &gvox_rs::RegionRange,
        channel_flags: gvox_rs::ChannelFlags,
    ) -> Result<gvox_rs::Region<Self::RegionData>, gvox_rs::GvoxError> {
        let available_channels = gvox_rs::ChannelId::COLOR
            | gvox_rs::ChannelId::NORMAL
            | gvox_rs::ChannelId::MATERIAL_ID;
        if (channel_flags & !available_channels) != gvox_rs::ChannelFlags::empty() {
            // gvox_sys::gvox_adapter_push_error(
            //     ctx,
            //     gvox_rs::RESULT_ERROR_PARSE_ADAPTER_REQUESTED_CHANNEL_NOT_PRESENT,
            //     cstr!("procedural 'parser' does not generate anything other than color & normal"),
            // );
        }
        Ok(gvox_rs::Region::new(
            *range,
            channel_flags & available_channels,
            gvox_rs::RegionFlags::empty(),
            (),
        ))
    }

    fn unload_region(
        &mut self,
        blit_ctx: &gvox_rs::ParseBlitContext,
        region: gvox_rs::Region<Self::RegionData>,
    ) -> Result<(), gvox_rs::GvoxError> {
        Ok(())
    }

    fn sample_region(
        &mut self,
        blit_ctx: &gvox_rs::ParseBlitContext,
        region: &gvox_rs::Region<Self::RegionData>,
        offset: &gvox_rs::Offset3D,
        channel_id: gvox_rs::ChannelId,
    ) -> Result<u32, gvox_rs::GvoxError> {
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

        Ok(if channel_id == gvox_rs::ChannelId::COLOR {
            color
        } else if channel_id == gvox_rs::ChannelId::NORMAL {
            normal
        } else if channel_id == gvox_rs::ChannelId::MATERIAL_ID {
            id
        } else {
            0
        })
    }
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
