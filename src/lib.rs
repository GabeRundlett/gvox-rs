/// The set of default adapters that come built-in.
pub mod adapters;

#[cfg(test)]
mod tests;

use bitflags::*;
use fxhash::*;
use int_enum::*;
use std::any::*;
use std::error::*;
use std::ffi::*;
use std::marker::*;
use std::mem::*;
use std::ops::*;
use std::slice::*;
use std::sync::*;

/// Copies a range of voxel data from the specified input
/// to the specified output, parsing and then serializing
/// the data using the provided format adapters.
pub fn blit_region(
    input_ctx: &mut AdapterContext<'_, Input>,
    output_ctx: &mut AdapterContext<'_, Output>,
    parse_ctx: &mut AdapterContext<'_, Parse>,
    serialize_ctx: &mut AdapterContext<'_, Serialize>,
    range: &RegionRange,
    channel_flags: ChannelFlags
) -> Result<(), GvoxError> {
    unsafe {
        input_ctx.context().execute_inner(|ctx| {
            gvox_sys::gvox_blit_region(
                input_ctx.as_mut_ptr(),
                output_ctx.as_mut_ptr(),
                parse_ctx.as_mut_ptr(),
                serialize_ctx.as_mut_ptr(),
                range as *const RegionRange as *const gvox_sys::GvoxRegionRange,
                channel_flags.into()
            );

            ctx.get_error()
        })
    }
}

/// Stores the capabilities, information, and state about a set of voxel blitting operations.
/// Adapters can be created or obtained from contexts.
#[derive(Clone, Debug, Default)]
pub struct Context(Arc<Mutex<ContextInner>>);

impl Context {
    /// Creates a new context for voxel format operations.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the adapter of the provided type and description, or returns an error if it could not be found.
    pub fn get_adapter<K: AdapterKind, A: AdapterDescriptor<K> + NamedAdapter>(&self) -> Result<Adapter<K, A>, GvoxError> {
        let ptr = self.execute_inner(|ctx| ctx.get_raw_adapter::<K, A>())?;

        Ok(Adapter { ctx: self.clone(), ptr, data: PhantomData::default() })
    }

    pub fn register_adapter<K: AdapterKind, A: AdapterDescriptor<K> + NamedAdapter + private::RegisterableAdapter<K>>(&self) -> Result<Adapter<K, A>, GvoxError> {
        self.execute_inner(|ctx| ctx.register_adapter::<K, A>())?;
        self.get_adapter::<K, A>()
    }

    /// Retrieves a raw handle to the context.
    pub fn as_mut_ptr(&self) -> *mut gvox_sys::GvoxContext {
        self.execute_inner(|ctx| ctx.ptr)
    }

    /// Executes the provided function synchronously on the context's inner data, and returns the result.
    fn execute_inner<T>(&self, f: impl FnOnce(&mut ContextInner) -> T) -> T {
        f(&mut self.0.lock().expect("Could not acquire context mutex."))
    }
}

impl PartialEq for Context {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Context {}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

/// Stores the inner, synchronized state of a context.
#[derive(Clone, Debug)]
struct ContextInner {
    ptr: *mut gvox_sys::GvoxContext,
    registered_adapter_types: FxHashMap<AdapterIdentifier, TypeId>
}

impl ContextInner {
    /// Gets a raw, non-null pointer to the adapter of the given type and name. Returns an
    /// error if the adapter could not be found or was not of the correct type.
    pub fn get_raw_adapter<K: AdapterKind, A: NamedAdapter>(&self) -> Result<*mut gvox_sys::GvoxAdapter, GvoxError> {
        unsafe {
            let adapter_type = self.registered_adapter_types.get(&AdapterIdentifier::new::<K, A>());
            if adapter_type == Some(&TypeId::of::<A>()) {
                let c_name = CString::new(A::name()).expect("Failed to convert Rust string to C string");
                let kind = TypeId::of::<K>();

                let adapter = if kind == TypeId::of::<Input>() {
                    gvox_sys::gvox_get_input_adapter(self.ptr, c_name.as_ptr())
                }
                else if kind == TypeId::of::<Output>() {
                    gvox_sys::gvox_get_output_adapter(self.ptr, c_name.as_ptr())
                }
                else if kind == TypeId::of::<Parse>() {
                    gvox_sys::gvox_get_parse_adapter(self.ptr, c_name.as_ptr())
                }
                else if kind == TypeId::of::<Serialize>() {
                    gvox_sys::gvox_get_serialize_adapter(self.ptr, c_name.as_ptr())
                }
                else {
                    return Err(GvoxError::new(ErrorType::Unknown, "Unrecognized adapter type.".to_string()));
                };

                self.get_error().and((!adapter.is_null()).then_some(adapter).ok_or_else(|| GvoxError::new(ErrorType::Unknown, "Adapter not found.".to_string())))
            }
            else if adapter_type.is_some() {
                Err(GvoxError::new(ErrorType::InvalidParameter, "The provided adapter was not of the correct type.".to_string()))
            }
            else {
                Err(GvoxError::new(ErrorType::InvalidParameter, "The provided adapter was not found.".to_string()))
            }
        }
    }

    /// Registers an adapter for voxel conversion operations, and returns a raw pointer to the adapter.
    fn register_adapter<K: AdapterKind, A: private::RegisterableAdapter<K>>(&mut self) -> Result<*mut gvox_sys::GvoxAdapter, GvoxError> {
        unsafe {
            let adapter = A::register_adapter(self.ptr)?;
            self.add_external_adapter::<K, A>()?;
            Ok(adapter)
        }
    }

    /// Obtains a raw pointer to a new adapter context, using the given adapter and configuration.
    /// 
    /// # Safety
    /// 
    /// Adapter must be a valid adapter associated with this context, and config must point to a datastructure
    /// of the correct layout for the given adapter.
    pub unsafe fn create_raw_adapter_context(&mut self, adapter: *mut gvox_sys::GvoxAdapter, config: *const c_void) -> Result<*mut gvox_sys::GvoxAdapterContext, GvoxError> {
        let result = gvox_sys::gvox_create_adapter_context(self.ptr, adapter, config);
        self.get_error()?;
        Ok(result)
    }

    /// Adds an external adapter (one that was already registered with the context outside of this API)
    /// to this context, so that it may be safely retrieved and used.
    /// 
    /// # Safety
    /// 
    /// For this call to be sound, the provided adapter must have already been registered
    /// with the given name on the underlying context. The adapter must support operations
    /// for the selected adapter kind, and the configuration structure that the adapter accepts
    /// must match that of the underlying context.
    pub unsafe fn add_external_adapter<K: AdapterKind, A: AdapterDescriptor<K> + NamedAdapter>(&mut self) -> Result<(), GvoxError> {
        let id = AdapterIdentifier::new::<K, A>();
        if self.registered_adapter_types.contains_key(&id) {
            Err(GvoxError::new(ErrorType::InvalidParameter, "Attempted to register duplicate adapter.".to_string()))
        }
        else {
            self.registered_adapter_types.insert(id, TypeId::of::<A>());
            Ok(())
        }
    }

    /// Adds all builtin adapters to the context, so that they may be queried and used.
    fn add_default_adapters(&mut self) -> Result<(), GvoxError> {
        unsafe {
            self.add_external_adapter::<Input, adapters::ByteBuffer>()?;
            self.add_external_adapter::<Output, adapters::ByteBuffer>()?;
            self.add_external_adapter::<Output, adapters::StdOut>()?;
            self.add_external_adapter::<Parse, adapters::GvoxPalette>()?;
            self.add_external_adapter::<Parse, adapters::GvoxRaw>()?;
            self.add_external_adapter::<Parse, adapters::MagicaVoxel>()?;
            self.add_external_adapter::<Parse, adapters::Voxlap>()?;
            self.add_external_adapter::<Serialize, adapters::ColoredText>()?;
            self.add_external_adapter::<Serialize, adapters::GvoxPalette>()?;
            self.add_external_adapter::<Serialize, adapters::GvoxRaw>()?;

            Ok(())
        }
    }

    /// Flushes the context error stack, and returns the topmost error.
    fn get_error(&self) -> Result<(), GvoxError> {
        unsafe {
            Self::get_error_from_raw_ptr(self.ptr)
        }
    }

    /// Flushes the error stack of the provided context, and returns the topmost error.
    pub unsafe fn get_error_from_raw_ptr(ptr: *mut gvox_sys::GvoxContext) -> Result<(), GvoxError> {
        let mut result = Ok(());

        let mut code = gvox_sys::gvox_get_result(ptr);
        let mut buf = Vec::new();
        while code != gvox_sys::GvoxResult_GVOX_RESULT_SUCCESS {
            let mut msg_size = 0;
            gvox_sys::gvox_get_result_message(ptr, 0 as *mut i8, &mut msg_size);
            buf.resize(msg_size, 0);
            gvox_sys::gvox_get_result_message(
                ptr,
                buf.as_mut_ptr() as *mut i8,
                &mut msg_size,
            );

            result = Err(GvoxError::new(ErrorType::from_int(code).unwrap_or(ErrorType::Unknown), std::str::from_utf8(buf.as_slice()).unwrap_or_default().to_string()));

            gvox_sys::gvox_pop_result(ptr);
            code = gvox_sys::gvox_get_result(ptr);
        }
        
        result
    }
}

impl Default for ContextInner {
    fn default() -> Self {
        unsafe {
            let ptr = gvox_sys::gvox_create_context();
            let registered_adapter_types = FxHashMap::default();
            let mut res = Self { ptr, registered_adapter_types };
            res.add_default_adapters().expect("Could not add default adapters to gvox context.");

            res
        }
    }
}

impl Drop for ContextInner {
    fn drop(&mut self) {
        unsafe {
            gvox_sys::gvox_destroy_context(self.ptr)
        }
    }
}

/// Uniquely identifies an adapter registration by name and kind.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct AdapterIdentifier {
    name: &'static str,
    kind: TypeId
}

impl AdapterIdentifier {
    /// Creates a new identifier for the provided adapter name and kind.
    pub fn new<K: AdapterKind, A: NamedAdapter>() -> Self {
        Self {
            name: A::name(),
            kind: TypeId::of::<K>()
        }
    }
}

/// Acts as an abstract interface over the ability to read, write, parse, and serialize voxel data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Adapter<K: AdapterKind, A: AdapterDescriptor<K>> {
    ctx: Context,
    ptr: *mut gvox_sys::GvoxAdapter,
    data: PhantomData<(K, A)>
}

impl<K: AdapterKind, A: AdapterDescriptor<K>> Adapter<K, A> {
    /// The context to which this adapter belongs.
    pub fn context(&self) -> Context {
        self.ctx.clone()
    }

    /// Creates a new adapter context instance, with the given configuration, that can be utilized to perform voxel blitting operations.
    pub fn create_adapter_context<'a>(&self, config: &A::Configuration<'a>) -> Result<AdapterContext<'a, K>, GvoxError> {
        unsafe {
            let ctx = self.context();
            let ptr = self.ctx.execute_inner(|ctx| ctx.create_raw_adapter_context(self.ptr, config as *const A::Configuration<'a> as *const c_void))?;

            if !ExternalHandler::is_external::<K, A>() {
                AdapterContextHolder::from_raw(ptr).get_context_data().expect("No user data was associated with context.").ctx = self.ctx.as_mut_ptr();
            }
            
            Ok(AdapterContext { ctx, ptr, data: PhantomData::default() })
        }
    }

    /// Retrieves a raw handle to the adapter.
    pub fn as_mut_ptr(&mut self) -> *mut gvox_sys::GvoxAdapter {
        self.ptr
    }
}

/// One specific instance of a configured adapter that can be used to perform blitting operations.
#[derive(Debug, PartialEq, Eq)]
pub struct AdapterContext<'a, K: AdapterKind> {
    ctx: Context,
    ptr: *mut gvox_sys::GvoxAdapterContext,
    data: PhantomData<(&'a (), K)>
}

impl<'a, K: AdapterKind> AdapterContext<'a, K> {
    /// The context to which this adapter context belongs.
    pub fn context(&self) -> Context {
        self.ctx.clone()
    }

    /// Retrieves a raw handle to the adapter context.
    pub fn as_mut_ptr(&mut self) -> *mut gvox_sys::GvoxAdapterContext {
        self.ptr
    }
}

impl<'a, K: AdapterKind> Drop for AdapterContext<'a, K> {
    fn drop(&mut self) {
        unsafe {
            gvox_sys::gvox_destroy_adapter_context(self.as_mut_ptr());
        }
    }
}

/// Describes the purpose of a particular adapter.
pub trait AdapterKind: 'static + private::Sealed {}

/// Marks types that read voxel input data.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Input;

impl AdapterKind for Input {}

/// Marks types that write voxel output data.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Output;

impl AdapterKind for Output {}

/// Marks types that decode voxel data from a provided input stream.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Parse;

impl AdapterKind for Parse {}

/// Marks types that encode voxel data from a provided parser.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Serialize;

impl AdapterKind for Serialize {}

/// Marks types that which have blit callbacks handled externally.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExternalHandler;

impl ExternalHandler {
    /// Determines whether the provided adapter descriptor is an externally managed (native) adapter.
    pub fn is_external<K: AdapterKind, A: AdapterDescriptor<K>>() -> bool {
        TypeId::of::<Self>() == TypeId::of::<A::Handler>()
    }
}

/// Describes the layout of an adapter and its configuration type.
pub trait AdapterDescriptor<K: AdapterKind>: 'static {
    /// The datastructure that this adapter accepts during context creation.
    type Configuration<'a>;
    /// The datastructure that stores user state and handles adapter callbacks.
    type Handler: ?Sized;
}

/// Represents an adapter which may be queried by name from a context.
pub trait NamedAdapter: 'static {
    /// The name of this adapter.
    fn name() -> &'static str;
}

/// Stores adapter context data.
struct AdapterContextData {
    /// The context with which this data is associated.
    pub ctx: *mut gvox_sys::GvoxContext,
    /// The adapter context with which this data is associated.
    pub ptr: *mut gvox_sys::GvoxAdapterContext,
    /// The user data that is associated with the current context.
    pub user_data: Option<Box<dyn Any>>
}

/// Provides the ability to access adapter context data.
struct AdapterContextHolder(*mut gvox_sys::GvoxAdapterContext);

impl AdapterContextHolder {
    /// Creates a new adapter context holder from the provided pointer.
    /// 
    /// # Safety
    /// 
    /// For this function call to be sound, the pointer must point to a valid context
    /// that was initialized by `gvox_rs`, and no other context holder to this pointer
    /// must exist. Further, the `gvox_rs` context must be locked for the entirety of this
    /// holder's lifetime.
    pub unsafe fn from_raw(ctx: *mut gvox_sys::GvoxAdapterContext) -> Self {
        Self(ctx)
    }

    /// Retrieves a raw pointer to the underlying adapter context.
    pub fn as_mut_ptr(&self) -> *mut gvox_sys::GvoxAdapterContext {
        self.0
    }

    /// Retrieves a raw pointer to the underlying context, or panics if no data was associated with the provided adapter.
    pub fn context_mut_ptr(&mut self) -> *mut gvox_sys::GvoxContext {
        self.get_context_data().expect("No data was associated with the given adapter context.").ctx
    }

    /// Gets the error associated with the most recent operation.
    pub fn get_result(&mut self) -> Result<(), GvoxError> {
        unsafe {
            ContextInner::get_error_from_raw_ptr(self.context_mut_ptr())
        }
    }

    /// Pushes a new error to the underlying context.
    pub fn push_error(&mut self, error: GvoxError) {
        unsafe {
            let message = CString::new(error.message.as_str()).unwrap_or_default();
            gvox_sys::gvox_adapter_push_error(self.0, error.error_type() as i32, message.as_ptr());
        }
    }

    /// Applies an operation to the held user data object, or panics if the user data object type did not match.
    pub fn user_data_operation<H: 'static>(&mut self, f: impl FnOnce(&mut H) -> Result<(), GvoxError>) {
        let mut result = Ok(());
        if let Some(data) = self.get_user_data_holder() {
            result = f(data.downcast_mut::<H>().expect("Context user data was not of correct type."));
        }
        
        if let Err(error) = result {
            self.push_error(error);
        }
    }

    /// Retrieves a reference to holder for adapter user data.
    pub fn get_user_data_holder(&mut self) -> &mut Option<Box<dyn Any>> {
        &mut self.get_context_data().expect("No data was associated with the given adapter context.").user_data
    }
    
    /// Retrieves a reference to the context's data, if it is set.
    fn get_context_data(&mut self) -> Option<&mut AdapterContextData> {
        unsafe {
            let res = gvox_sys::gvox_adapter_get_user_pointer(self.0) as *mut AdapterContextData;
            (!res.is_null()).then(|| &mut *res)
        }
    }

    /// Sets the context data to the provided value, returning the data that was previously overwritten.
    fn set_context_data(&mut self, data: Option<AdapterContextData>) -> Option<AdapterContextData> {
        unsafe {
            let res = gvox_sys::gvox_adapter_get_user_pointer(self.0) as *mut AdapterContextData;
            gvox_sys::gvox_adapter_set_user_pointer(self.0, data.map(|x| Box::into_raw(Box::new(x)) as *mut c_void).unwrap_or(std::ptr::null_mut()));
            (!res.is_null()).then(|| *Box::from_raw(res))
        }
    }

    /// Invokes the adapter context creation function for the given adapter type.
    /// 
    /// # Safety
    /// 
    /// The provided adapter context pointer must be initializable as a valid context holder,
    /// and config must point to a valid configuration object.
    unsafe extern "C" fn create<K: private::AdapterKindAssociation, D: AdapterDescriptor<K>>(ptr: *mut gvox_sys::GvoxAdapterContext, config: *const c_void) where D::Handler: BaseAdapterHandler<K, D> {
        let mut ctx = Self::from_raw(ptr);
        ctx.set_context_data(Some(AdapterContextData { ctx: std::ptr::null_mut(), ptr, user_data: None }));

        match D::Handler::create(&*(config as *const D::Configuration<'_>)) {
            Ok(value) => *ctx.get_user_data_holder() = Some(Box::new(value)),
            Err(error) => ctx.push_error(error)
        };
    }

    /// Invokes the adapter context deletion function for the given adapter type.
    /// 
    /// # Safety
    /// 
    /// The provided adapter context pointer must be initializable as a valid context holder,
    /// and config must point to a valid configuration object.
    unsafe extern "C" fn destroy<K: private::AdapterKindAssociation, D: AdapterDescriptor<K>>(ctx: *mut gvox_sys::GvoxAdapterContext) where D::Handler: BaseAdapterHandler<K, D> {
        let mut ctx = Self::from_raw(ctx);
        let data = take(ctx.get_user_data_holder());

        if let Some(value) = data {
            if let Err(error) = value.downcast::<D::Handler>().expect("Context user data was not of correct type.").destroy() {
                ctx.push_error(error);
            }
        }

        ctx.set_context_data(None);
    }    

    /// Invokes the adapter context blit beginning function for the given adapter type.
    /// 
    /// # Safety
    /// 
    /// The provided adapter context pointer must be initializable as a valid input context holder.
    unsafe extern "C" fn blit_begin<K: private::AdapterKindAssociation, D: AdapterDescriptor<K>>(blit_ctx: *mut gvox_sys::GvoxBlitContext, ctx: *mut gvox_sys::GvoxAdapterContext) where D::Handler: BaseAdapterHandler<K, D> {
        let mut ctx = Self::from_raw(ctx);
        let blit_ctx = K::BlitContext::new(ctx.context_mut_ptr(), blit_ctx);

        ctx.user_data_operation::<D::Handler>(|h| h.blit_begin(&blit_ctx));
    }

    /// Invokes the adapter context blit ending function for the given adapter type.
    /// 
    /// # Safety
    /// 
    /// The provided adapter context pointer must be initializable as a valid input context holder.
    unsafe extern "C" fn blit_end<K: private::AdapterKindAssociation, D: AdapterDescriptor<K>>(blit_ctx: *mut gvox_sys::GvoxBlitContext, ctx: *mut gvox_sys::GvoxAdapterContext) where D::Handler: BaseAdapterHandler<K, D> {
        let mut ctx = Self::from_raw(ctx);
        let blit_ctx = K::BlitContext::new(ctx.context_mut_ptr(), blit_ctx);

        ctx.user_data_operation::<D::Handler>(|h| h.blit_end(&blit_ctx));
    }
}

/// Provides the ability to access input adapter context data.
struct InputContextHolder(AdapterContextHolder);

impl InputContextHolder {
    /// Creates a new input adapter context holder from the provided pointer.
    /// 
    /// # Safety
    /// 
    /// For this function call to be sound, the pointer must point to a valid input
    /// adapter context, and all of the invariants required by `AdapterContextHolder::from_raw`
    /// must be satisfied.
    unsafe fn from_raw(ctx: *mut gvox_sys::GvoxAdapterContext) -> Self {
        Self(AdapterContextHolder::from_raw(ctx))
    }

    /// Invokes the adapter context reading function for the given adapter type.
    /// 
    /// # Safety
    /// 
    /// The provided adapter context pointer must be initializable as a valid input context holder,
    /// and size and data must describe a valid, writeable section of memory.
    unsafe extern "C" fn read<D: AdapterDescriptor<Input>>(ctx: *mut gvox_sys::GvoxAdapterContext, position: usize, size: usize, data: *mut c_void) where D::Handler: InputAdapterHandler<D> {
        let mut ctx = Self::from_raw(ctx);
        let blit_ctx = InputBlitContext();

        ctx.0.user_data_operation::<D::Handler>(|h| h.read(&blit_ctx, position, from_raw_parts_mut(data as *mut u8, size)));
    }
}

/// Provides the ability to access output adapter context data.
struct OutputContextHolder(AdapterContextHolder);

impl OutputContextHolder {
    /// Creates a new output adapter context holder from the provided pointer.
    /// 
    /// # Safety
    /// 
    /// For this function call to be sound, the pointer must point to a valid output
    /// adapter context, and all of the invariants required by `AdapterContextHolder::from_raw`
    /// must be satisfied.
    unsafe fn from_raw(ctx: *mut gvox_sys::GvoxAdapterContext) -> Self {
        Self(AdapterContextHolder::from_raw(ctx))
    }

    /// Invokes the adapter context writing function for the given adapter type.
    /// 
    /// # Safety
    /// 
    /// The provided adapter context pointer must be initializable as a valid output context holder,
    /// and size and data must describe a valid, readable section of memory.
    unsafe extern "C" fn write<D: AdapterDescriptor<Output>>(ctx: *mut gvox_sys::GvoxAdapterContext, position: usize, size: usize, data: *const c_void) where D::Handler: OutputAdapterHandler<D> {
        let mut ctx = Self::from_raw(ctx);
        let blit_ctx = OutputBlitContext();

        ctx.0.user_data_operation::<D::Handler>(|h| h.write(&blit_ctx, position, from_raw_parts(data as *const u8, size)));
    }

    /// Invokes the adapter context reservation function for the given adapter type.
    /// 
    /// # Safety
    /// 
    /// The provided adapter context pointer must be initializable as a valid output context holder.
    unsafe extern "C" fn reserve<D: AdapterDescriptor<Output>>(ctx: *mut gvox_sys::GvoxAdapterContext, size: usize) where D::Handler: OutputAdapterHandler<D> {
        let mut ctx = Self::from_raw(ctx);
        let blit_ctx = OutputBlitContext();

        ctx.0.user_data_operation::<D::Handler>(|h| h.reserve(&blit_ctx, size));
    }
}

/// Provides the ability for an input adapter to interact with other adapters during blit operations.
pub struct InputBlitContext();

impl BlitContextType for InputBlitContext {
    unsafe fn new(_: *mut gvox_sys::GvoxContext, _: *mut gvox_sys::GvoxBlitContext) -> Self {
        Self()
    }
}

/// Provides the ability for an output adapter to interact with other adapters during blit operations.
pub struct OutputBlitContext();

impl BlitContextType for OutputBlitContext {
    unsafe fn new(_: *mut gvox_sys::GvoxContext, _: *mut gvox_sys::GvoxBlitContext) -> Self {
        Self()
    }
}

trait BlitContextType: 'static + Sized {
    /// Creates a new blit context for the given context and blit pointers.
    /// 
    /// # Safety
    /// 
    /// For this function call to be sound, both parameters must point to valid contexts
    /// and this object must not outlive either of them.
    unsafe fn new(ctx: *mut gvox_sys::GvoxContext, blit_ctx: *mut gvox_sys::GvoxBlitContext) -> Self;
}

/// Represents the user data type that handles adapter context operations.
pub trait BaseAdapterHandler<K: AdapterKind + private::AdapterKindAssociation, D: AdapterDescriptor<K, Handler = Self>>: 'static + Sized {
    /// Creates a new adapter context handler with the supplied configuration.
    fn create(config: &D::Configuration<'_>) -> Result<Self, GvoxError>; 
    /// Destroys the provided adapter context handler.
    fn destroy(self) -> Result<(), GvoxError>;
    
    /// Called whenever a blit operation begins for the current context.
    fn blit_begin(&mut self, blit_ctx: &K::BlitContext) -> Result<(), GvoxError> { Ok(()) }
    /// Called whenever a blit operation ends for the current context.
    fn blit_end(&mut self, blit_ctx: &K::BlitContext) -> Result<(), GvoxError> { Ok(()) }
}

/// Represents the user data type that handles input adapter context operations.
pub trait InputAdapterHandler<D: AdapterDescriptor<Input, Handler = Self>>: BaseAdapterHandler<Input, D> {
    /// Fills the provided data slice with bytes from this input adapter, beginning at the specified source offset.
    fn read(&mut self, blit_ctx: &InputBlitContext, position: usize, data: &mut [u8]) -> Result<(), GvoxError>;
}

/// Represents the user data type that handles output adapter context operations.
pub trait OutputAdapterHandler<D: AdapterDescriptor<Output, Handler = Self>>: BaseAdapterHandler<Output, D> {
    /// Writes the provided data slice to this output adapter, beginning at the specified target offset.
    fn write(&mut self, blit_ctx: &OutputBlitContext, position: usize, data: &[u8]) -> Result<(), GvoxError>;
    /// Hints that the adapter should expect to have at least the given number of total bytes written to it.
    fn reserve(&mut self, blit_ctx: &OutputBlitContext, size: usize) -> Result<(), GvoxError>;
}

/*
pub trait OutputAdapter: AdapterContextHandler<Output> {
    fn write(&mut self, position: usize, data: &[u8]);
    fn reserve(&mut self, size: usize);
}

pub trait ParseAdapter: AdapterContextHandler<Parse> {
    fn query_region_flags(&mut self, blit_ctx: (), range: &RegionRange, channel_flags: ChannelFlags) -> RegionFlags;
    fn load_region(&mut self, blit_ctx: (), range: &RegionRange, channel_flags: ChannelFlags) -> ();
    fn sample_region(&mut self, blit_ctx: (), region: &(), offset: &Offset3D, channel_id: ChannelId);
}

pub trait SerializeAdapter: AdapterContextHandler<Serialize> {
    fn serialize_region(&mut self, blit_ctx: (), range: &RegionRange, channel_flags: ChannelFlags);
}*/

/// Represents an offset on a 3D grid.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct Offset3D {
    /// The x-component of the offset.
    pub x: i32,
    /// The y-component of the offset.
    pub y: i32,
    /// The z-component of the offset.
    pub z: i32,
}

/// Represents the dimensions of a volume on a 3D grid.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct Extent3D {
    /// The x-length of the dimensions.
    pub x: i32,
    /// The y-length of the dimensions.
    pub y: i32,
    /// The z-length of the dimensions.
    pub z: i32,
}

/// Represents a volume on a 3D grid.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct RegionRange {
    /// The position of the minimum corner of the region.
    pub offset: Offset3D,
    /// The dimensions of the region.
    pub extent: Extent3D,
}

/// Describes the type of error that occurred during voxel conversion operations.
#[derive(Copy, Clone, Debug, PartialEq, Eq, IntEnum)]
#[repr(i32)]
pub enum ErrorType {
    /// There is no information associated with this error type.
    Unknown = gvox_sys::GvoxResult_GVOX_RESULT_ERROR_UNKNOWN,
    /// A supplied parameter was invalid.
    InvalidParameter = gvox_sys::GvoxResult_GVOX_RESULT_ERROR_INVALID_PARAMETER,
    /// An issue occurred with an input adapter.
    InputAdapter = gvox_sys::GvoxResult_GVOX_RESULT_ERROR_INPUT_ADAPTER,
    /// An issue occurred with an output adapter.
    OutputAdapter = gvox_sys::GvoxResult_GVOX_RESULT_ERROR_OUTPUT_ADAPTER,
    /// An issue occurred with a parse adapter.
    ParseAdapter = gvox_sys::GvoxResult_GVOX_RESULT_ERROR_PARSE_ADAPTER,
    /// An issue occurred with a serialize adapter.
    SerializeAdapter = gvox_sys::GvoxResult_GVOX_RESULT_ERROR_SERIALIZE_ADAPTER,
    /// A parse adapter was provided invalid input.
    ParseAdapterInvalidInput = gvox_sys::GvoxResult_GVOX_RESULT_ERROR_PARSE_ADAPTER_INVALID_INPUT,
    /// A voxel channel was not available for a parse adapter to read.
    ParseAdapterRequestedChannelNotPresent = gvox_sys::GvoxResult_GVOX_RESULT_ERROR_PARSE_ADAPTER_REQUESTED_CHANNEL_NOT_PRESENT,
    /// A serialize adapter's format did not support the output data type.
    SerializeAdapterUnrepresentableData = gvox_sys::GvoxResult_GVOX_RESULT_ERROR_SERIALIZE_ADAPTER_UNREPRESENTABLE_DATA
}

/// Describes an error that occurred during voxel conversion operations.
#[derive(Clone, Debug)]
pub struct GvoxError {
    ty: ErrorType,
    message: String,
}

impl GvoxError {
    /// Creates a new error with the provided type and reason message.
    pub fn new(ty: ErrorType, message: impl Into<String>) -> Self {
        let message = Into::<String>::into(message);
        Self { ty, message }
    }

    /// The type of error that occurred.
    pub fn error_type(&self) -> ErrorType {
        self.ty
    }
}

impl Error for GvoxError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for GvoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}: {}", self.ty, self.message))
    }
}

/// Identifies a specific property associated with a voxel volume.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ChannelId(u32);

impl ChannelId {
    /// The color of voxels.
    pub const COLOR: Self = Self(gvox_sys::GVOX_CHANNEL_ID_COLOR);
    /// The normal vector of voxels.
    pub const NORMAL: Self = Self(gvox_sys::GVOX_CHANNEL_ID_NORMAL);
    /// The material IDs of voxels.
    pub const MATERIAL_ID: Self = Self(gvox_sys::GVOX_CHANNEL_ID_MATERIAL_ID);
    /// The roughness coefficient of voxels.
    pub const ROUGHNESS: Self = Self(gvox_sys::GVOX_CHANNEL_ID_ROUGHNESS);
    /// The metalness coefficient of voxels.
    pub const METALNESS: Self = Self(gvox_sys::GVOX_CHANNEL_ID_METALNESS);
    /// The alpha value of volumes.
    pub const TRANSPARENCY: Self = Self(gvox_sys::GVOX_CHANNEL_ID_TRANSPARENCY);
    /// The IOR coefficient of voxels.
    pub const IOR: Self = Self(gvox_sys::GVOX_CHANNEL_ID_IOR);
    /// The emissive color of voxels.
    pub const EMISSIVE_COLOR: Self = Self(gvox_sys::GVOX_CHANNEL_ID_EMISSIVITY);
    /// The hardness coefficient of voxels.
    pub const HARDNESS: Self = Self(gvox_sys::GVOX_CHANNEL_ID_HARDNESS);

    /// Retrieves an iterator over all voxel channel IDs.
    pub fn iter() -> impl Iterator<Item = ChannelId> {
        (0..=gvox_sys::GVOX_CHANNEL_ID_LAST).map(|x| ChannelId(x))
    }
}

impl TryFrom<u32> for ChannelId {
    type Error = GvoxError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        (value <= gvox_sys::GVOX_CHANNEL_ID_LAST).then_some(Self(value))
            .ok_or_else(|| GvoxError::new(ErrorType::InvalidParameter, format!("Channel ID {value} is out of range 0..=31.")))
    }
}

impl From<ChannelId> for u32 {
    fn from(value: ChannelId) -> Self {
        value.0
    }
}

impl BitAnd for ChannelId {
    type Output = ChannelFlags;

    fn bitand(self, rhs: Self) -> Self::Output {
        ChannelFlags::from(self) & rhs
    }
}

impl BitOr for ChannelId {
    type Output = ChannelFlags;

    fn bitor(self, rhs: Self) -> Self::Output {
        ChannelFlags::from(self) | rhs
    }
}

impl BitXor for ChannelId {
    type Output = ChannelFlags;

    fn bitxor(self, rhs: Self) -> Self::Output {
        ChannelFlags::from(self) ^ rhs
    }
}

impl BitAnd<ChannelFlags> for ChannelId {
    type Output = ChannelFlags;

    fn bitand(self, rhs: ChannelFlags) -> Self::Output {
        ChannelFlags::from(self) & rhs
    }
}

impl BitOr<ChannelFlags> for ChannelId {
    type Output = ChannelFlags;

    fn bitor(self, rhs: ChannelFlags) -> Self::Output {
        ChannelFlags::from(self) | rhs
    }
}

impl BitXor<ChannelFlags> for ChannelId {
    type Output = ChannelFlags;

    fn bitxor(self, rhs: ChannelFlags) -> Self::Output {
        ChannelFlags::from(self) ^ rhs
    }
}

/// A set of binary flags which denotes a collection of channel IDs.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct ChannelFlags(u32);

impl ChannelFlags {
    /// Provides a set of flags that contains all possible channel IDs.
    pub const fn all() -> Self {
        Self(u32::MAX)
    }

    /// Returns whether the provided channel is contained in this ID set.
    pub fn contains(&self, x: ChannelId) -> bool {
        (self.0 & (1 << u32::from(x))) != 0
    }

    /// Provides a set of flags that contains no channel IDs.
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Creates an iterator over the set of channels contained in these flags.
    pub fn into_iter(self) -> impl Iterator<Item = ChannelId> {
        ChannelId::iter().filter(move |&x| self.contains(x))
    }
}

impl From<ChannelId> for ChannelFlags {
    fn from(value: ChannelId) -> Self {
        Self(1 << value.0)
    }
}

impl From<ChannelFlags> for u32 {
    fn from(value: ChannelFlags) -> Self {
        value.0
    }
}

impl BitAnd for ChannelFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for ChannelFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl BitOr for ChannelFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for ChannelFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl BitXor for ChannelFlags {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for ChannelFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl BitAnd<ChannelId> for ChannelFlags {
    type Output = Self;

    fn bitand(self, rhs: ChannelId) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign<ChannelId> for ChannelFlags {
    fn bitand_assign(&mut self, rhs: ChannelId) {
        *self = *self & rhs;
    }
}

impl BitOr<ChannelId> for ChannelFlags {
    type Output = Self;

    fn bitor(self, rhs: ChannelId) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign<ChannelId> for ChannelFlags {
    fn bitor_assign(&mut self, rhs: ChannelId) {
        *self = *self | rhs;
    }
}

impl BitXor<ChannelId> for ChannelFlags {
    type Output = Self;

    fn bitxor(self, rhs: ChannelId) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign<ChannelId> for ChannelFlags {
    fn bitxor_assign(&mut self, rhs: ChannelId) {
        *self = *self ^ rhs;
    }
}

bitflags! {
    /// Describes the group properties of a voxel region.
    pub struct RegionFlags: u32 {
        /// The given channel set has the same value over the entirety of the region.
        const UNIFORM = gvox_sys::GVOX_REGION_FLAG_UNIFORM;
    }
}

/// Private module utilized to hide private, externally-unimplementable traits.
mod private {
    use super::*;

    /// A trait which cannot be implemented from external crates, preventing end users
    /// from implementing subtraits for their own types.
    pub trait Sealed {}

    impl Sealed for Input {}
    impl Sealed for Output {}
    impl Sealed for Parse {}
    impl Sealed for Serialize {}

    /// Describes the types associated with a given operation kind.
    pub trait AdapterKindAssociation: AdapterKind {
        type BlitContext: BlitContextType;
    }
    
    impl AdapterKindAssociation for Input {
        type BlitContext = InputBlitContext;
    }
    impl AdapterKindAssociation for Output {
        type BlitContext = OutputBlitContext;
    }

    /// Provides the ability to register an adapter of the given type with a context. Automatically
    /// implemented for all adapter types with context handlers.
    pub trait RegisterableAdapter<K: AdapterKind>: AdapterDescriptor<K> + NamedAdapter {
        /// Registers the given adapter with the underlying context, and returns a pointer to it if the operation
        /// was successful.
        /// 
        /// # Safety
        /// 
        /// The provided pointer must be a valid reference to a context.
        unsafe fn register_adapter(ptr: *mut gvox_sys::GvoxContext) -> Result<*mut gvox_sys::GvoxAdapter, GvoxError>;
    }

    impl<T: AdapterDescriptor<Input> + NamedAdapter> RegisterableAdapter<Input> for T where T::Handler: InputAdapterHandler<T> {
        unsafe fn register_adapter(ptr: *mut gvox_sys::GvoxContext) -> Result<*mut gvox_sys::GvoxAdapter, GvoxError> {
            let name = CString::new(Self::name()).expect("Failed to convert Rust string to C string");
            let adapter_info = gvox_sys::GvoxInputAdapterInfo {
                base_info: gvox_sys::GvoxAdapterBaseInfo { name_str: name.as_ptr(), create: Some(AdapterContextHolder::create::<Input, Self>), destroy: Some(AdapterContextHolder::destroy::<Input, Self>),
                blit_begin: Some(AdapterContextHolder::blit_begin::<Input, Self>), blit_end: Some(AdapterContextHolder::blit_end::<Input, Self>) },
                read: Some(InputContextHolder::read::<Self>)
            };
            let adapter = gvox_sys::gvox_register_input_adapter(ptr, &adapter_info);
            ContextInner::get_error_from_raw_ptr(ptr).map(|()| adapter)
        }
    }

    impl<T: AdapterDescriptor<Output> + NamedAdapter> RegisterableAdapter<Output> for T where T::Handler: OutputAdapterHandler<T> {
        unsafe fn register_adapter(ptr: *mut gvox_sys::GvoxContext) -> Result<*mut gvox_sys::GvoxAdapter, GvoxError> {
            let name = CString::new(Self::name()).expect("Failed to convert Rust string to C string");
            let adapter_info = gvox_sys::GvoxOutputAdapterInfo {
                base_info: gvox_sys::GvoxAdapterBaseInfo { name_str: name.as_ptr(), create: Some(AdapterContextHolder::create::<Output, Self>), destroy: Some(AdapterContextHolder::destroy::<Output, Self>),
                blit_begin: Some(AdapterContextHolder::blit_begin::<Output, Self>), blit_end: Some(AdapterContextHolder::blit_end::<Output, Self>) },
                write: Some(OutputContextHolder::write::<Self>),
                reserve: Some(OutputContextHolder::reserve::<Self>),
            };
            let adapter = gvox_sys::gvox_register_output_adapter(ptr, &adapter_info);
            ContextInner::get_error_from_raw_ptr(ptr).map(|()| adapter)
        }
    }
}