use crate::*;
use std::mem::*;
use std::ops::*;

pub struct ByteBuffer;

impl AdapterDescriptor<Input> for ByteBuffer {
    type Configuration<'a> = &'a [u8];
}

impl AdapterDescriptor<Output> for ByteBuffer {
    type Configuration<'a> = ByteBufferOutputAdapterConfig<'a>;
}

impl NamedAdapter for ByteBuffer {
    fn name() -> &'static str {
        "byte_buffer"
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct ByteBufferOutputAdapterConfig<'a> {
    config: gvox_sys::GvoxByteBufferOutputAdapterConfig,
    output: &'a mut Option<Box<[u8]>>,
    old: Box<[u8]>
}

impl<'a> ByteBufferOutputAdapterConfig<'a> {
    extern "C" fn allocate(len: usize) -> *mut c_void {
        vec![0; len].into_boxed_slice().as_mut_ptr() as *mut c_void
    }
}

impl<'a> From<&'a mut Box<[u8]>> for ByteBufferOutputAdapterConfig<'a> {
    fn from(value: &'a mut Box<[u8]>) -> Self {
        unsafe {
            let old = take(value);
            let output: &mut Option<Box<[u8]>> = transmute(value);
            let out_byte_buffer_ptr = output as *mut Option<Box<[u8]>> as *mut *mut u8;

            // Since boxed slices are a pointer and then a length, we extract the pointer
            // and length to pass to the gvox C API.
            let config = gvox_sys::GvoxByteBufferOutputAdapterConfig {
                out_size: out_byte_buffer_ptr.add(1) as *mut usize,
                out_byte_buffer_ptr,
                allocate: Some(Self::allocate)
            };

            Self { config, output, old }
        }
    }
}

impl<'a> Drop for ByteBufferOutputAdapterConfig<'a> {
    fn drop(&mut self) {
        if self.output.is_none() {
            let old = take(&mut self.old);
            *self.output = Some(old);
        }
    }
}

pub struct ColoredText;

impl AdapterDescriptor<Serialize> for ColoredText {
    type Configuration<'a> = ColoredTextSerializeAdapterConfig;
}

impl NamedAdapter for ColoredText {
    fn name() -> &'static str {
        "colored_text"
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ColoredTextSerializeAdapterDownscaleMode {
    Nearest = 0,
    Linear = 1
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ColoredTextSerializeAdapterConfig {
    pub downscale_factor: u32,
    pub downscale_mode: ColoredTextSerializeAdapterDownscaleMode,
    pub non_color_max_value: u32,
}

pub struct StdOut;

impl AdapterDescriptor<Output> for StdOut {
    type Configuration<'a> = ();
}

impl NamedAdapter for StdOut {
    fn name() -> &'static str {
        "stdout"
    }
}

pub struct GvoxPalette;

impl AdapterDescriptor<Parse> for GvoxPalette {
    type Configuration<'a> = ();
}

impl AdapterDescriptor<Serialize> for GvoxPalette {
    type Configuration<'a> = ();
}

impl NamedAdapter for GvoxPalette {
    fn name() -> &'static str {
        "gvox_palette"
    }
}

pub struct GvoxRaw;

impl AdapterDescriptor<Parse> for GvoxRaw {
    type Configuration<'a> = ();
}

impl AdapterDescriptor<Serialize> for GvoxRaw {
    type Configuration<'a> = ();
}

impl NamedAdapter for GvoxRaw {
    fn name() -> &'static str {
        "gvox_raw"
    }
}

pub struct MagicaVoxel;

impl AdapterDescriptor<Parse> for MagicaVoxel {
    type Configuration<'a> = ();
}

impl NamedAdapter for MagicaVoxel {
    fn name() -> &'static str {
        "magicavoxel"
    }
}

pub struct Voxlap;

impl AdapterDescriptor<Parse> for Voxlap {
    type Configuration<'a> = VoxlapParseAdapterConfig;
}

impl NamedAdapter for Voxlap {
    fn name() -> &'static str {
        "voxlap"
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VoxlapParseAdapterConfig {
    size: Extent3D,
    make_solid: bool,
    is_ace_of_spades: bool
}

impl Default for VoxlapParseAdapterConfig {
    fn default() -> Self {
        Self { size: Extent3D::default(), make_solid: true, is_ace_of_spades: Default::default() }
    }
}