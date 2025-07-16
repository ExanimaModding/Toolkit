use std::{
	mem,
	num::NonZeroIsize,
	sync::{Once, OnceLock},
};

use bevy_derive::Deref;
use bevy_ecs::prelude::*;
use bevy_reflect::prelude::*;
use bevy_winit::{EventLoopProxy, WakeUp};
use crossbeam_channel::{Receiver, Sender};
use detours_sys::{DetourAttach, DetourTransactionBegin, DetourTransactionCommit};
use raw_window_handle::Win32WindowHandle;
use tracing::info;
use winapi::{
	shared::windef::HDC,
	um::{
		libloaderapi::{GetProcAddress, LoadLibraryA},
		wingdi::{wglCreateContext, wglGetCurrentContext, wglMakeCurrent},
		winnt::HRESULT,
		winuser::WindowFromDC,
	},
};

/// A pointer to the process' original opengl32 function, wglSwapBuffers, that
/// is initialized when the overlay runs.
pub static OPENGL32_WGL_SWAP_BUFFERS: OnceLock<unsafe extern "system" fn(HDC) -> HRESULT> =
	OnceLock::new();

/// A window handle that is initialized on the first wglSwapBuffers call made by
/// Exanima's process.
pub static WINDOW_HANDLE: OnceLock<Win32WindowHandle> = OnceLock::new();

pub static SENDER: OnceLock<RenderWorldSender> = OnceLock::new();

pub static EVENT_LOOP_PROXY: OnceLock<EventLoopProxy<OverlayEvent>> = OnceLock::new();
// pub static EVENT_LOOP_PROXY: OnceLock<EventLoopProxy<WakeUp>> = OnceLock::new();

pub static OVERLAY_CONTEXT: OnceLock<NonZeroIsize> = OnceLock::new();

/// This will receive asynchronously any data sent from the render world
#[derive(Debug, Resource, Deref)]
pub struct MainWorldReceiver(pub Receiver<Vec<u8>>);

/// This will send asynchronously any data to the main world
#[derive(Debug, Resource, Deref)]
pub struct RenderWorldSender(pub Sender<Vec<u8>>);

#[derive(Debug, Default, Event, Reflect)]
#[reflect(Debug, Default)]
pub enum OverlayEvent {
	#[default]
	WakeUp,
}

// #[derive(Event)]
// pub struct WindowHandle(Win32WindowHandle);

/// # Safety
///
/// Calls to this function should only ever be made by the Exanima process which
/// internally handles [`HDC`] to be non-null.
unsafe extern "system" fn detour_opengl32_wglswapbuffers(hdc: HDC) -> HRESULT {
	static HOOK: Once = Once::new();
	HOOK.call_once(|| {
		info!("opengl32 wglSwapBuffers hook running");

		let target_window = unsafe { WindowFromDC(hdc) };

		unsafe {
			// let context = wglGetCurrentContext();
			let overlay_context = wglCreateContext(hdc);
			OVERLAY_CONTEXT
				.set(NonZeroIsize::new(overlay_context as _).unwrap())
				.unwrap();
		}

		WINDOW_HANDLE
			.set(Win32WindowHandle::new(
				NonZeroIsize::new(target_window as _).unwrap(),
			))
			.unwrap();
	});

	if let Some(overlay_context) = OVERLAY_CONTEXT.get() {
		let context = unsafe {
			let context = wglGetCurrentContext();
			wglMakeCurrent(hdc, overlay_context.get() as _);
			context
		};

		if let Some(event_loop_proxy) = EVENT_LOOP_PROXY.get() {
			event_loop_proxy.send_event(OverlayEvent::WakeUp).unwrap();
			// event_loop_proxy.send_event(WakeUp).unwrap();
		}
		// TODO: render

		unsafe { wglMakeCurrent(hdc, context) };
	}

	let target_opengl32_wglswapbuffers = OPENGL32_WGL_SWAP_BUFFERS.get().unwrap();
	unsafe { target_opengl32_wglswapbuffers(hdc) }
}

pub fn run() {
	info!("Running {}", env!("CARGO_PKG_NAME"));

	// SAFETY:
	// - opengl is loaded first to guarantee pointers as non-null
	OPENGL32_WGL_SWAP_BUFFERS
		.set(unsafe {
			let opengl32 = LoadLibraryA(c"opengl32.dll".as_ptr());
			let mut target_opengl32_wglswapbuffers =
				GetProcAddress(opengl32, c"wglSwapBuffers".as_ptr());

			DetourTransactionBegin();
			DetourAttach(
				&raw mut target_opengl32_wglswapbuffers as _,
				detour_opengl32_wglswapbuffers as _,
			);
			DetourTransactionCommit();

			mem::transmute(target_opengl32_wglswapbuffers)
		})
		.unwrap();
}
