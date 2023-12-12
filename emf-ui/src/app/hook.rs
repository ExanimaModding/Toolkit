use std::os::raw::c_void;

use detours_sys::{DetourAttach, DetourTransactionBegin, DetourTransactionCommit};
use egui::Context;
use parking_lot::Once;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::UI::WindowsAndMessaging::{
	CallWindowProcW, SetWindowLongPtrA, GWLP_WNDPROC, WNDPROC,
};
use windows::{core::HRESULT, Win32::Graphics::Gdi::WindowFromDC};

use crate::app::render;
use crate::opengl::OpenGLApp;

use super::app_state::AppState;

/// wglSwapBuffers function type.
pub type WglSwapBuffers = unsafe extern "stdcall" fn(HDC) -> HRESULT;

#[link(name = "opengl32")]
extern "stdcall" {
	/// The external wglSwapBuffers method to be hooked.
	fn wglSwapBuffers(hdc: HDC) -> HRESULT;
}

static mut original_wgl_swap_buffers: *mut c_void = 0 as _;

pub fn init() {
	unsafe {
		original_wgl_swap_buffers = wglSwapBuffers as _;

		DetourTransactionBegin();

		DetourAttach(&mut original_wgl_swap_buffers, wgl_swap_buffers as _);

		DetourTransactionCommit();
	}
}

pub static mut APP: OpenGLApp<AppState> = OpenGLApp::new();
pub static mut OLD_WND_PROC: Option<WNDPROC> = None;

pub unsafe extern "stdcall" fn wgl_swap_buffers(hdc: HDC) -> HRESULT {
	static INIT: Once = Once::new();
	INIT.call_once(|| {
		println!("wglSwapBuffers successfully hooked.");

		let window = WindowFromDC(hdc);
		APP.init_default(hdc, window, render);

		OLD_WND_PROC = Some(std::mem::transmute(SetWindowLongPtrA(
			window,
			GWLP_WNDPROC,
			hk_wnd_proc as usize as _,
		)));
	});

	APP.render(hdc);

	let wgl_swap_buffers: WglSwapBuffers = std::mem::transmute(original_wgl_swap_buffers);

	wgl_swap_buffers(hdc)
}

unsafe extern "stdcall" fn hk_wnd_proc(
	hwnd: HWND,
	msg: u32,
	wparam: WPARAM,
	lparam: LPARAM,
) -> LRESULT {
	static INIT: Once = Once::new();
	INIT.call_once(|| {
		println!("CallWindowProcW successfully hooked.");
	});

	let egui_wants_input = APP.wnd_proc(msg, wparam, lparam);
	if egui_wants_input {
		return LRESULT(1);
	}

	CallWindowProcW(OLD_WND_PROC.unwrap(), hwnd, msg, wparam, lparam)
}
