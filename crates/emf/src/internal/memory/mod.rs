pub(crate) mod sigscanner;

#[allow(unused)]
trait AsPtr<T> {
	fn as_ptr(&self) -> *const T;
	fn as_mut_ptr(&mut self) -> *mut T;
}

#[allow(unused)]
trait AsNum<T> {
	fn as_num(&self) -> T;
}

pub struct Ptr;

#[allow(unused)]
impl Ptr {
	pub fn as_const<T>(ptr: usize) -> *const T {
		ptr as *const T
	}

	pub fn as_mut<T>(ptr: usize) -> *mut T {
		ptr as *mut T
	}

	pub fn as_i32(ptr: *const usize) -> i64 {
		ptr as i64
	}

	pub unsafe fn deref(ptr: usize) -> *mut usize {
		*(ptr as *mut *mut usize)
	}

	pub fn offset<T>(ptr: usize, offset: i64) -> *mut T {
		(ptr as i64 + offset) as *mut T
	}
}

#[allow(non_camel_case_types, clippy::enum_variant_names)]
pub enum _MEMORY_INFORMATION_CLASS {
	MemoryBasicInformation,          // MEMORY_BASIC_INFORMATION
	MemoryWorkingSetInformation,     // MEMORY_WORKING_SET_INFORMATION
	MemoryMappedFilenameInformation, // UNICODE_STRING
	MemoryRegionInformation,         // MEMORY_REGION_INFORMATION
	MemoryWorkingSetExInformation,   // MEMORY_WORKING_SET_EX_INFORMATION
	MemorySharedCommitInformation,   // MEMORY_SHARED_COMMIT_INFORMATION
	MemoryImageInformation,          // MEMORY_IMAGE_INFORMATION
}
