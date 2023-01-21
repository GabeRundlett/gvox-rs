use std::alloc::*;
use std::cmp::*;
use std::os::raw::*;

#[no_mangle]
pub extern "C" fn malloc(size: usize) -> *mut c_void {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size, 1);
        alloc(layout).cast()
    }
}

#[no_mangle]
pub extern "C" fn calloc(
    nmemb: usize,
    size: usize,
) -> *mut c_void {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size * nmemb, 1);
        alloc(layout).cast()
    }
}

#[no_mangle]
pub unsafe extern "C" fn posix_memalign(memptr: *mut *mut c_void, alignment: usize, size: usize) -> c_int {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size, alignment);
        *memptr = alloc(layout).cast();
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    let layout = Layout::from_size_align_unchecked(1, 1);
    dealloc(ptr.cast(), layout);
}

#[no_mangle]
pub unsafe extern "C" fn strcpy(
    dst: *mut c_char, 
    src: *const c_char
) -> *mut c_char {
    let mut i = 0;
    loop {
        let char = *src.add(i);
        if char == 0 {
            break;
        }
        else {
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
        Ordering::Greater => 1
    }
}

#[no_mangle]
pub unsafe extern "C" fn strcasecmp(str1: *const c_char, str2: *const c_char) -> c_int {
    let a = std::ffi::CStr::from_ptr(str1).to_string_lossy().to_lowercase();
    let b = std::ffi::CStr::from_ptr(str2).to_string_lossy().to_lowercase();
    match a.cmp(&b) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1
    }
}

#[no_mangle]
pub unsafe extern "C" fn atof(str: *const c_char) -> f64 {
    std::ffi::CStr::from_ptr(str).to_str().ok().and_then(|x| x.parse().ok()).unwrap_or_default()
}

#[no_mangle]
pub unsafe extern "C" fn atoi(str: *const c_char) -> c_int {
    std::ffi::CStr::from_ptr(str).to_str().ok().and_then(|x| x.parse().ok()).unwrap_or_default()
}

#[no_mangle]
pub unsafe extern "C" fn abort() {
    std::process::abort();
}