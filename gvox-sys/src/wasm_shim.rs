use std::alloc::*;
use std::cmp::*;
use std::mem::*;
use std::os::raw::*;
use std::ptr::*;

#[no_mangle]
pub extern "C" fn malloc(size: usize) -> *mut c_void {
    unsafe {
        let mut result = MaybeUninit::uninit();
        posix_memalign(result.as_mut_ptr(), align_of::<usize>(), size);
        result.assume_init_read()
    }
}

#[no_mangle]
pub extern "C" fn calloc(nmemb: usize, size: usize) -> *mut c_void {
    unsafe {
        let bytes = size * nmemb;
        let mut result = MaybeUninit::uninit();
        posix_memalign(result.as_mut_ptr(), align_of::<usize>(), bytes);
        let byte_data = std::slice::from_raw_parts_mut(result.assume_init_read().cast::<u8>(), bytes);
        byte_data.fill(0);
        result.assume_init_read()
    }
}

#[no_mangle]
pub unsafe extern "C" fn posix_memalign(
    memptr: *mut *mut c_void,
    alignment: usize,
    size: usize,
) -> c_int {
    unsafe {
        const USER_OFFSET: usize = size_of::<AllocationTag>() + size_of::<usize>();

        if size == 0 {
            *memptr = null_mut();
            0
        }
        else {
            let tagged_size = (size + USER_OFFSET).next_multiple_of(alignment);
            let layout = Layout::from_size_align_unchecked(tagged_size, alignment);
            let ptr = alloc(layout);
    
            if ptr.is_null() {
                12
            }
            else {
                ptr.cast::<AllocationTag>().write(AllocationTag { size: tagged_size, align: alignment });
                let user_start_position = (ptr as usize + USER_OFFSET).next_multiple_of(alignment) as *mut *const AllocationTag;
                user_start_position.sub(1).write(ptr.cast());
                *memptr = user_start_position.cast();
                0
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    if !ptr.is_null() {
        let tag_ptr = ptr.cast::<*mut AllocationTag>().sub(1).read();
        let tag = tag_ptr.read();
        let layout = Layout::from_size_align_unchecked(tag.size, tag.align);
        dealloc(tag_ptr.cast(), layout);
    }
}

#[no_mangle]
pub unsafe extern "C" fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char {
    let mut i = 0;
    loop {
        let char = *src.add(i);
        if char == 0 {
            break;
        } else {
            *dst.add(i) = char;
        }

        i += 1;
    }

    dst
}

#[no_mangle]
pub unsafe extern "C" fn strcmp(str1: *const c_char, str2: *const c_char) -> c_int {
    let a = std::ffi::CStr::from_ptr(str1);
    let b = std::ffi::CStr::from_ptr(str2);
    match a.cmp(b) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn strcasecmp(str1: *const c_char, str2: *const c_char) -> c_int {
    let a = std::ffi::CStr::from_ptr(str1)
        .to_string_lossy()
        .to_lowercase();
    let b = std::ffi::CStr::from_ptr(str2)
        .to_string_lossy()
        .to_lowercase();
    match a.cmp(&b) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn atof(str: *const c_char) -> f64 {
    std::ffi::CStr::from_ptr(str)
        .to_str()
        .ok()
        .and_then(|x| x.parse().ok())
        .unwrap_or_default()
}

#[no_mangle]
pub unsafe extern "C" fn atoi(str: *const c_char) -> c_int {
    std::ffi::CStr::from_ptr(str)
        .to_str()
        .ok()
        .and_then(|x| x.parse().ok())
        .unwrap_or_default()
}

#[no_mangle]
pub unsafe extern "C" fn abort() {
    std::process::abort();
}

#[no_mangle]
pub unsafe extern "C" fn memchr(str: *const c_void, ch: c_int, count: usize) -> *const c_void {
    let cstr = std::slice::from_raw_parts(str as *const u8, count);
    let mut index = 0;
    loop {
        if index >= count {
            return std::ptr::null();
        }
        let c = cstr[index];
        if c == ch as u8 {
            return (str as *const c_char).add(index) as *const c_void;
        }
        index += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn aligned_alloc(alignment: usize, size: usize) -> *mut c_void {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size, alignment);
        alloc(layout).cast()
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
struct AllocationTag {
    pub size: usize,
    pub align: usize
}