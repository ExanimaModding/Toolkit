use winapi::um::{
	memoryapi::VirtualAlloc,
	winnt::{MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE},
};

use std::alloc::Layout;

use log::*;

// Allocate 1mb to hooks. Change as needed.
static PAGE_SIZE: usize = 1024 * 1024; // 1MB
static ALIGN_BY: usize = 16;

static mut PAGE_ADDRESS: *mut u8 = std::ptr::null_mut();

pub struct VecAllocator;

unsafe impl std::alloc::Allocator for VecAllocator {
	fn allocate(&self, layout: Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
		unsafe {
			let mut current_ptr = PAGE_ADDRESS;

			// Align to 16 bytes
			while (current_ptr as usize) & (ALIGN_BY - 1) != 0 {
				current_ptr = current_ptr.offset(1);
			}

			if (current_ptr as usize) + layout.size() > (PAGE_ADDRESS as usize) + PAGE_SIZE {
				// Allocation out of bounds
				return Err(std::alloc::AllocError);
			}

			let result = current_ptr;
			current_ptr = current_ptr.add(layout.size());
			PAGE_ADDRESS = current_ptr;

			return Ok(std::ptr::NonNull::new(std::slice::from_raw_parts_mut(
				result,
				layout.size(),
			))
			.unwrap());
		}
	}

	unsafe fn deallocate(&self, _ptr: std::ptr::NonNull<u8>, _layout: Layout) {}
}

#[allow(unused)]
pub unsafe fn alloc_page() {
	let page_ptr = VirtualAlloc(
		std::ptr::null_mut(),
		PAGE_SIZE,
		MEM_COMMIT | MEM_RESERVE,
		PAGE_EXECUTE_READWRITE,
	) as *mut u8;

	if page_ptr.is_null() {
		panic!("Failed to allocate memory page for hooks");
	}

	info!("Allocated memory page for hooks: {:p}", page_ptr);

	PAGE_ADDRESS = page_ptr;
}
