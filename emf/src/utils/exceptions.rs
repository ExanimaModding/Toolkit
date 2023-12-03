// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, ffi::c_long};

use std::sync::Mutex;

use winapi::um::winnt::PVECTORED_EXCEPTION_HANDLER;
use winapi::{um::winnt::EXCEPTION_POINTERS, vc::excpt::EXCEPTION_CONTINUE_SEARCH};

use lazy_static::lazy_static;

pub struct Handler {
    pub handlers: Mutex<HashMap<usize, PVECTORED_EXCEPTION_HANDLER>>,
}

#[allow(unused)]
impl Handler {
    pub fn add(&self, addr: usize, handler: PVECTORED_EXCEPTION_HANDLER) {
        self.handlers.lock().unwrap().insert(addr, handler);
    }

    pub fn remove(&self, addr: usize) {
        self.handlers.lock().unwrap().remove(&addr);
    }

    pub fn contains_key(&self, addr: usize) -> bool {
        self.handlers.lock().unwrap().contains_key(&addr)
    }

    pub fn get(&self, addr: usize) -> PVECTORED_EXCEPTION_HANDLER {
        let binding = self.handlers.lock().unwrap();
        let func = binding.get(&addr).unwrap();
        // self.handlers.lock().unwrap().get(&addr)
        *func
    }
}

// TODO: There's probably a better way to do this.
lazy_static! {
    pub static ref HANDLERS: Handler = Handler {
        handlers: Mutex::new(HashMap::new()),
    };
}

#[macro_export]
macro_rules! make_handler {
    ($name:tt, $func:expr) => {
        pub unsafe extern "system" fn $name(
            exception_info: *mut winapi::um::winnt::EXCEPTION_POINTERS,
        ) -> std::ffi::c_long {
            $func(exception_info)
        }
    };
}

#[allow(unused)]
pub unsafe extern "system" fn error_handler(exception_info: *mut EXCEPTION_POINTERS) -> c_long {
    let context = *(*exception_info).ContextRecord;
    let exception = *(*exception_info).ExceptionRecord;

    let mut handler: PVECTORED_EXCEPTION_HANDLER = None;

    dbg!(exception.ExceptionRecord);
    dbg!(exception.ExceptionFlags);
    dbg!(exception.ExceptionCode);
    dbg!(exception.ExceptionAddress);
    dbg!(exception.ExceptionInformation[1]);

    dbg!(context.ContextFlags);

    for i in exception.ExceptionInformation {
        if HANDLERS.contains_key(i) {
            println!("Found: {:#08x}", i);
            handler = HANDLERS.get(i);
            break;
        } else {
            println!("Not Found: {:#08x}", i);
        }
    }

    if let Some(handler) = handler {
        handler(exception_info)
    } else {
        EXCEPTION_CONTINUE_SEARCH
    }
}
