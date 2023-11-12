use libmem::LM_SigScan;
use winapi::shared::minwindef::DWORD;

use crate::utils::pe32::PE32;

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
        Self {
            signature: signature.to_string(),
            search_start: h_module.lpBaseOfDll as DWORD,
            search_length: h_module.SizeOfImage as usize,
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
