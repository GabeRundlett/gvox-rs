use crate::*;
use std::mem::*;
use std::ops::*;

/// Provides the ability to process data directly from memory.
pub struct ByteBuffer;

impl AdapterDescriptor<Input> for ByteBuffer {
    type Configuration<'a> = &'a [u8];
    type Handler = ExternalHandler;
}

impl AdapterDescriptor<Output> for ByteBuffer {
    type Configuration<'a> = ByteBufferOutputAdapterConfig<'a>;
    type Handler = ExternalHandler;
}

impl NamedAdapter for ByteBuffer {
    fn name() -> &'static str {
        "byte_buffer"
    }
}

/// Describes a reference to an output byte buffer.
#[derive(Debug)]
#[repr(C)]
pub struct ByteBufferOutputAdapterConfig<'a> {
    /// A configuration describing how the native adapter should write its output.
    config: gvox_sys::GvoxByteBufferOutputAdapterConfig,
    /// A reference to the output buffer to which the adapter should write.
    output: &'a mut Option<Box<[u8]>>,
    /// The original byte buffer that lived at the referenced output. The output will replace
    /// this buffer only if the adapter writes to the output.
    old: Box<[u8]>,
}

impl<'a> ByteBufferOutputAdapterConfig<'a> {
    /// Allocates a zeroed segment of memory with the default Rust allocator.
    extern "C" fn allocate(len: usize) -> *mut c_void {
        Box::into_raw(vec![0; len].into_boxed_slice()) as *mut c_void
    }
}

impl<'a> From<&'a mut Box<[u8]>> for ByteBufferOutputAdapterConfig<'a> {
    fn from(value: &'a mut Box<[u8]>) -> Self {
        unsafe {
            let old = take(value);
            let output: &mut Option<Box<[u8]>> = transmute(value);
            forget(take(output));
            let out_byte_buffer_ptr = output as *mut Option<Box<[u8]>> as *mut *mut u8;

            // Since boxed slices are a pointer and then a length, we extract the pointer
            // and length to pass to the gvox C API.
            let config = gvox_sys::GvoxByteBufferOutputAdapterConfig {
                out_size: out_byte_buffer_ptr.add(1) as *mut usize,
                out_byte_buffer_ptr,
                allocate: Some(Self::allocate),
            };

            Self {
                config,
                output,
                old,
            }
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

/// Provides the ability to process data from a storage device.
pub struct File;

impl AdapterDescriptor<Input> for File {
    type Configuration<'a> = FileInputAdapterConfig;
    type Handler = ExternalHandler;
}

impl AdapterDescriptor<Output> for File {
    type Configuration<'a> = FileOutputAdapterConfig;
    type Handler = ExternalHandler;
}

impl NamedAdapter for File {
    fn name() -> &'static str {
        "file"
    }
}

/// Specifies the way that a file adapter will read from storage.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct FileInputAdapterConfig {
    /// A configuration describing the file that the adapter should use. This member must come first
    /// in order for the native adapter to use it.
    config: gvox_sys::GvoxFileInputAdapterConfig,
    /// The name of this file. This must outlive `config`, which references the underlying buffer.
    file_name: CString,
}

impl FileInputAdapterConfig {
    /// Create a new file input for the given file name and byte offset.
    pub fn new(file_name: impl Into<String>, byte_offset: usize) -> Self {
        let name = Into::<String>::into(file_name);
        let file_name = CString::new(name).expect("Could not convert file name to C string.");
        let config = gvox_sys::GvoxFileInputAdapterConfig {
            filepath: file_name.as_ptr(),
            byte_offset,
        };

        Self { file_name, config }
    }
}

/// Specifies the way that a file adapter will write to storage.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct FileOutputAdapterConfig {
    /// A configuration describing the file that the adapter should use. This member must come first
    /// in order for the native adapter to use it.
    config: gvox_sys::GvoxFileOutputAdapterConfig,
    /// The name of this file. This must outlive `config`, which references the underlying buffer.
    file_name: CString,
}

impl FileOutputAdapterConfig {
    /// Create a new file output for the given file name.
    pub fn new(file_name: impl Into<String>) -> Self {
        let name = Into::<String>::into(file_name);
        let file_name = CString::new(name).expect("Could not convert file name to C string.");
        let config = gvox_sys::GvoxFileOutputAdapterConfig {
            filepath: file_name.as_ptr(),
        };

        Self { file_name, config }
    }
}

/// Converts voxels to a text visualization which may be displayed in a console.
pub struct ColoredText;

impl AdapterDescriptor<Serialize> for ColoredText {
    type Configuration<'a> = ColoredTextSerializeAdapterConfig;
    type Handler = ExternalHandler;
}

impl NamedAdapter for ColoredText {
    fn name() -> &'static str {
        "colored_text"
    }
}

/// Describes how voxels should be downscaled when creating a visualization.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ColoredTextSerializeAdapterDownscaleMode {
    /// The nearest voxel's value should be taken during filtering.
    Nearest = 0,
    /// Linear blending should be utilized to filter the voxels.
    Linear = 1,
}

/// Provides settings for controlling how voxels are visualized as colored text.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ColoredTextSerializeAdapterConfig {
    /// The factor by which voxels should be downscaled.
    pub downscale_factor: u32,
    /// The filtering mode that should be employed during downscaling.
    pub downscale_mode: ColoredTextSerializeAdapterDownscaleMode,
    /// The value that should be considered greatest when handling non-color data.
    pub non_color_max_value: u32,
    /// Whether each layer should be printed below the last, as opposed to the right of the last
    pub vertical: bool,
}

impl Default for ColoredTextSerializeAdapterConfig {
    fn default() -> Self {
        Self {
            downscale_factor: 1,
            downscale_mode: ColoredTextSerializeAdapterDownscaleMode::Nearest,
            non_color_max_value: 0,
            vertical: false,
        }
    }
}

/// Prints voxel data directly to the standard console output.
pub struct StdOut;

impl AdapterDescriptor<Output> for StdOut {
    type Configuration<'a> = ();
    type Handler = ExternalHandler;
}

impl NamedAdapter for StdOut {
    fn name() -> &'static str {
        "stdout"
    }
}

/// Handles conversions from the tagged gvox format.
pub struct GvoxPalette;

impl AdapterDescriptor<Parse> for GvoxPalette {
    type Configuration<'a> = ();
    type Handler = ExternalHandler;
}

impl AdapterDescriptor<Serialize> for GvoxPalette {
    type Configuration<'a> = ();
    type Handler = ExternalHandler;
}

impl NamedAdapter for GvoxPalette {
    fn name() -> &'static str {
        "gvox_palette"
    }
}

/// Handles conversions from the raw gvox format.
pub struct GvoxRaw;

impl AdapterDescriptor<Parse> for GvoxRaw {
    type Configuration<'a> = ();
    type Handler = ExternalHandler;
}

impl AdapterDescriptor<Serialize> for GvoxRaw {
    type Configuration<'a> = ();
    type Handler = ExternalHandler;
}

impl NamedAdapter for GvoxRaw {
    fn name() -> &'static str {
        "gvox_raw"
    }
}

/// Handles conversions for Gvox Brickmap files.
pub struct GvoxBrickmap;

impl AdapterDescriptor<Parse> for GvoxBrickmap {
    type Configuration<'a> = ();
    type Handler = ExternalHandler;
}

impl AdapterDescriptor<Serialize> for GvoxBrickmap {
    type Configuration<'a> = ();
    type Handler = ExternalHandler;
}

impl NamedAdapter for GvoxBrickmap {
    fn name() -> &'static str {
        "gvox_brickmap"
    }
}

/// Handles conversions for Gvox Global Palette files.
pub struct GvoxGlobalPalette;

impl AdapterDescriptor<Parse> for GvoxGlobalPalette {
    type Configuration<'a> = ();
    type Handler = ExternalHandler;
}

impl AdapterDescriptor<Serialize> for GvoxGlobalPalette {
    type Configuration<'a> = ();
    type Handler = ExternalHandler;
}

impl NamedAdapter for GvoxGlobalPalette {
    fn name() -> &'static str {
        "gvox_global_palette"
    }
}

/// Handles conversions for Gvox Octree files.
pub struct GvoxOctree;

impl AdapterDescriptor<Parse> for GvoxOctree {
    type Configuration<'a> = ();
    type Handler = ExternalHandler;
}

impl AdapterDescriptor<Serialize> for GvoxOctree {
    type Configuration<'a> = ();
    type Handler = ExternalHandler;
}

impl NamedAdapter for GvoxOctree {
    fn name() -> &'static str {
        "gvox_octree"
    }
}

/// Handles conversions for Gvox RLE files.
pub struct GvoxRunLengthEncoding;

impl AdapterDescriptor<Parse> for GvoxRunLengthEncoding {
    type Configuration<'a> = ();
    type Handler = ExternalHandler;
}

impl AdapterDescriptor<Serialize> for GvoxRunLengthEncoding {
    type Configuration<'a> = ();
    type Handler = ExternalHandler;
}

impl NamedAdapter for GvoxRunLengthEncoding {
    fn name() -> &'static str {
        "gvox_run_length_encoding"
    }
}

/// Handles conversions for MagicaVoxel files.
pub struct MagicaVoxel;

impl AdapterDescriptor<Parse> for MagicaVoxel {
    type Configuration<'a> = ();
    type Handler = ExternalHandler;
}

impl NamedAdapter for MagicaVoxel {
    fn name() -> &'static str {
        "magicavoxel"
    }
}

/// Handles conversions for Voxlap and Ace of Spades files.
pub struct Voxlap;

impl AdapterDescriptor<Parse> for Voxlap {
    type Configuration<'a> = VoxlapParseAdapterConfig;
    type Handler = ExternalHandler;
}

impl NamedAdapter for Voxlap {
    fn name() -> &'static str {
        "voxlap"
    }
}

/// Describes how Voxlap data should be parsed.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VoxlapParseAdapterConfig {
    /// The dimensions of the input data.
    pub size: Extent3D,
    /// Whether to fill in the inside of objects, or leave them hollow.
    pub make_solid: bool,
    /// Whether this an Ace of Spades file. Ace of Spades files do not have a header.
    pub is_ace_of_spades: bool,
}

impl Default for VoxlapParseAdapterConfig {
    fn default() -> Self {
        Self {
            size: Extent3D {
                x: 1024,
                y: 1024,
                z: 256,
            },
            make_solid: true,
            is_ace_of_spades: Default::default(),
        }
    }
}

/// Handles conversions for Voxlap and Ace of Spades files.
pub struct Kvx;

impl AdapterDescriptor<Parse> for Kvx {
    type Configuration<'a> = KvxParseAdapterConfig;
    type Handler = ExternalHandler;
}

impl NamedAdapter for Kvx {
    fn name() -> &'static str {
        "kvx"
    }
}

/// Describes how Kvx data should be parsed.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct KvxParseAdapterConfig {
    pub mipmaplevels: u8,
}

impl Default for KvxParseAdapterConfig {
    fn default() -> Self {
        Self { mipmaplevels: 5 }
    }
}
