// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

use libmem_sys::LM_SigScan;
use pelite::pe::Pe;
use serde::{Deserialize, Serialize};
use winapi::shared::ntdef::DWORDLONG;

use crate::internal::utils::pe64::PE64;

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum SigScannerResult {
	Found(usize),
	NotFound,
}

#[allow(unused)]
impl SigScannerResult {
	pub fn value(&self) -> Option<usize> {
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

	pub fn shift(&mut self, shift: usize) {
		match self {
			SigScannerResult::Found(ptr) => *ptr += shift,
			SigScannerResult::NotFound => {}
		}
	}
}

#[derive(Debug)]
pub struct SigScanner {
	signature: String,
	search_start: DWORDLONG,
	search_length: usize,
}

#[allow(unused)]
impl SigScanner {
	pub unsafe fn exec(&self) -> SigScannerResult {
		let cstr = std::ffi::CString::new(self.signature.clone()).unwrap();

		let ptr = LM_SigScan(cstr.as_ptr(), self.search_start as _, self.search_length);

		if ptr == self.search_start as _ || ptr == 0 {
			return SigScannerResult::NotFound;
		}

		SigScannerResult::Found(ptr)
	}

	pub unsafe fn new(signature: &str) -> Self {
		let h_module = PE64::get_module_information();
		let sections = h_module.section_headers();

		let text_section = sections.iter().find(|section| {
			if let Ok(name) = section.name() {
				name == ".text"
			} else {
				false
			}
		});

		let text_section = if let Some(text_section) = text_section {
			text_section
		} else {
			panic!("Failed to find .text section");
		};

		// dbg!(text_section);

		let image_base = h_module.optional_header().ImageBase;
		let search_start = h_module.optional_header().BaseOfCode as u64;
		let search_length = h_module.optional_header().SizeOfCode as usize;

		let search_start = image_base + text_section.VirtualAddress as u64;
		let search_length = text_section.VirtualSize as usize;

		Self {
			signature: signature.to_string(),
			search_start,
			search_length,
		}
	}

	pub unsafe fn new_ex(signature: &str, search_start: DWORDLONG, search_length: usize) -> Self {
		Self {
			signature: signature.to_string(),
			search_start,
			search_length,
		}
	}
}
