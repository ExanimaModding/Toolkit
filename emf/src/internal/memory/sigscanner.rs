// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

use libmem::LM_SigScan;
use pelite::pe32::Pe;
use winapi::shared::minwindef::DWORD;

use crate::internal::utils::pe32::PE32;

use super::Ptr;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum SigScannerResult {
	Found(*mut DWORD),
	NotFound,
}

#[allow(unused)]
impl SigScannerResult {
	pub fn value(&self) -> Option<*mut DWORD> {
		match self {
			SigScannerResult::Found(ptr) => Some(*ptr),
			SigScannerResult::NotFound => None,
		}
	}
	pub fn found(&mut self) -> bool {
		match self {
			SigScannerResult::Found(_) => true,
			SigScannerResult::NotFound => false,
		}
	}

	pub fn shift(&mut self, shift: i32) {
		match self {
			SigScannerResult::Found(ptr) => {
				*ptr = Ptr::offset(*ptr as u32, shift);
			}
			SigScannerResult::NotFound => {}
		}
	}
}

#[derive(Debug)]
pub struct SigScanner {
	signature: String,
	search_start: DWORD,
	search_length: usize,
}

#[allow(unused)]
impl SigScanner {
	pub unsafe fn exec(&self) -> SigScannerResult {
		let result = LM_SigScan(
			self.signature.as_str(),
			self.search_start as usize,
			self.search_length,
		);
		if result.is_none() {
			return SigScannerResult::NotFound;
		}
		SigScannerResult::Found(result.unwrap() as *mut DWORD)
	}

	pub unsafe fn new(signature: &str) -> Self {
		let h_module = PE32::get_module_information();
		let sections = h_module.section_headers();

		let text = sections
			.iter()
			.find(|section| section.name().unwrap() == ".text")
			.unwrap();

		let search_start = h_module.optional_header().ImageBase as DWORD + text.VirtualAddress;
		let search_length = text.VirtualSize as usize;

		Self {
			signature: signature.to_string(),
			search_start,
			search_length,
		}
	}

	pub unsafe fn new_ex(signature: &str, search_start: DWORD, search_length: usize) -> Self {
		Self {
			signature: signature.to_string(),
			search_start,
			search_length,
		}
	}
}
