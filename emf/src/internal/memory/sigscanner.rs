// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

use libmem_sys::{LM_SigScan, LM_ADDRESS_BAD};
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
		let result = LM_SigScan(
			self.signature.as_ptr() as _,
			self.search_start as usize,
			self.search_length,
		);

		if result == LM_ADDRESS_BAD {
			return SigScannerResult::NotFound;
		}

		SigScannerResult::Found(result as usize)
	}

	pub unsafe fn new(signature: &str) -> Self {
		let h_module = PE64::get_module_information();
		let sections = h_module.section_headers();

		let search_start = h_module.optional_header().ImageBase;
		let search_length = h_module.optional_header().SizeOfImage as usize;

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
