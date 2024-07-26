use pelite::pe::Pe;
use serde::{Deserialize, Serialize};

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
	search_start: usize,
	search_length: usize,
}

#[allow(unused)]
impl SigScanner {
	pub unsafe fn exec(&self) -> SigScannerResult {
		match libmem::sig_scan(&self.signature, self.search_start as _, self.search_length) {
			Some(ptr) => SigScannerResult::Found(ptr),
			None => SigScannerResult::NotFound,
		}
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

		let image_base = h_module.optional_header().ImageBase as usize;
		let search_start = h_module.optional_header().BaseOfCode as usize;
		let search_length = h_module.optional_header().SizeOfCode as usize;

		let search_start = image_base + text_section.VirtualAddress as usize;
		let search_length = text_section.VirtualSize as usize;

		Self {
			signature: signature.to_string(),
			search_start,
			search_length,
		}
	}

	pub unsafe fn new_ex(signature: &str, search_start: usize, search_length: usize) -> Self {
		Self {
			signature: signature.to_string(),
			search_start,
			search_length,
		}
	}
}
