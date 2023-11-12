pub mod sigscanner;

use winapi::shared::minwindef::DWORD;

pub struct Ptr;

#[allow(unused)]
impl Ptr {
    pub fn as_const<T>(ptr: DWORD) -> *const T {
        ptr as *const T
    }

    pub fn as_mut<T>(ptr: DWORD) -> *mut T {
        ptr as *mut T
    }

    pub unsafe fn deref(ptr: DWORD) -> *mut DWORD {
        *(ptr as *mut *mut DWORD)
    }

    pub fn offset<T>(ptr: DWORD, offset: i32) -> *mut T {
        (ptr as i32 + offset) as *mut T
    }
}
