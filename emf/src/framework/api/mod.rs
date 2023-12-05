// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

use winapi::shared::minwindef::DWORD;

use crate::internal::{
	lua,
	memory::{sigscanner::SigScanner, MemPatch, Ptr},
};
use std::result::Result;

pub mod motile;

static mut PLAYER_STATE: PlayerState = PlayerState {
	player: Player {
		health: PlayerHealth { ptr: None },
	},
};

#[derive(Default)]
struct PlayerState {
	pub player: Player,
}

#[derive(Default)]
pub struct Player {
	health: PlayerHealth,
}

#[derive(Default)]
struct PlayerHealth {
	ptr: Option<*mut *mut *mut f32>,
}

#[allow(unused)]
impl PlayerHealth {
	/// Initialise the PlayerHealth Pointer.
	pub unsafe fn init(&mut self) {
		let mut result = SigScanner::new("A1 ?? ?? ?? ?? 8B 00 D9 80 48 0B 00 00 A1").exec();
		result.shift(1);
		self.ptr = Some(*result.value().unwrap() as _);
	}

	/// Get the current Stamina of the player.
	pub unsafe fn get_current_stamina(&mut self) -> Result<f32, &'static str> {
		let stamina_ptr = self.get_health_ptr()?;
		let current_stamina: *mut f32 = Ptr::offset(stamina_ptr as DWORD, 0x0b30);
		Ok(*current_stamina)
	}

	/// Set the current Stamina of the player.
	pub unsafe fn set_current_stamina(&mut self, health: f32) -> Result<(), &'static str> {
		let stamina_ptr = self.get_health_ptr()?;
		let current_stamina: *mut f32 = Ptr::offset(stamina_ptr as DWORD, 0x0b30);

		// if !HANDLERS.contains_key(current_stamina as usize) {
		//     println!("Adding Health Handler {:#08x}", current_stamina as usize);
		//     HANDLERS.add(current_stamina as usize, Some(health_handler));
		// }

		// virtual_protect(current_stamina as _, 4, PAGE_EXECUTE_READWRITE);
		*current_stamina = health;
		// virtual_protect(current_stamina as _, 4, PAGE_READONLY);
		Ok(())
	}

	// PStamina
	pub unsafe fn get_max_stamina(&mut self) -> Result<f32, &'static str> {
		let stamina_ptr = self.get_health_ptr()?;
		let max_stamina: *mut f32 = Ptr::offset(stamina_ptr as DWORD, 0x0b44);
		Ok(*max_stamina)
	}

	pub unsafe fn set_max_stamina(&mut self, health: f32) -> Result<(), &'static str> {
		let stamina_ptr = self.get_health_ptr()?;
		let max_stamina: *mut f32 = Ptr::offset(stamina_ptr as DWORD, 0x0b44);
		*max_stamina = health;
		Ok(())
	}

	// MaxStamina
	pub unsafe fn get_damage(&mut self) -> Result<f32, &'static str> {
		let stamina_ptr = self.get_health_ptr()?;
		let damage: *mut f32 = Ptr::offset(stamina_ptr as DWORD, 0x0b34);
		Ok(*damage)
	}

	pub unsafe fn set_damage(&mut self, health: f32) -> Result<(), &'static str> {
		let stamina_ptr = self.get_health_ptr()?;
		let damage: *mut f32 = Ptr::offset(stamina_ptr as DWORD, 0x0b34);
		*damage = health;
		Ok(())
	}

	//     let current_focus: *mut f32 = Ptr::offset(health_ptr as DWORD, 0x0b4c);
	pub unsafe fn get_current_focus(&mut self) -> Result<f32, &'static str> {
		let stamina_ptr = self.get_health_ptr()?;
		let current_focus: *mut f32 = Ptr::offset(stamina_ptr as DWORD, 0x0b4c);
		Ok(*current_focus)
	}

	pub unsafe fn set_current_focus(&mut self, health: f32) -> Result<(), &'static str> {
		let stamina_ptr = self.get_health_ptr()?;
		let current_focus: *mut f32 = Ptr::offset(stamina_ptr as DWORD, 0x0b4c);
		*current_focus = health;
		Ok(())
	}

	unsafe fn get_health_ptr(&mut self) -> Result<*mut f32, &'static str> {
		let ref_health_ptr = *self.ptr.unwrap();
		if ref_health_ptr.is_null() {
			return Err("Player health not initialized");
		}

		let health_ptr = *ref_health_ptr;
		if health_ptr.is_null() {
			return Err("Player health not initialized");
		}

		Ok(health_ptr)
	}

	unsafe fn god_mode(&mut self) -> Result<(), String> {
		// Always do stamina damage
		let sig = "76 18 DB 2D ?? ?? ?? ?? D8 4D 08";
		let addr = SigScanner::new(sig).exec().value().unwrap() as *mut [u8; 2];
		*addr = [0x90, 0x90];

		// Disable stamina damage
		MemPatch::many(
			"D8 AE 30 0B 00 00 D9 9E 30 0B 00 00",
			12,
			&mut [0x90].repeat(12),
		)
		.unwrap();
		Ok(())
	}
}

#[allow(unused)]
impl Player {
	pub unsafe fn get_player_motile() -> Option<motile::Motile> {
		let health_ptr = PLAYER_STATE.player.health.get_health_ptr();
		if health_ptr.is_err() {
			return None;
		}
		motile::Motile::get_from_ptr(health_ptr.unwrap() as _)
	}

	/// Get the current Stamina (yellow health) of the player.
	///
	/// Minimum value: 0.0
	/// Maximum value: 0.25
	pub unsafe fn get_stamina() -> std::result::Result<f32, &'static str> {
		PLAYER_STATE.player.health.get_current_stamina()
	}

	/// Set the current Stamina (yellow health) of the player.
	///
	/// Minimum value: 0.0
	/// Maximum value: 0.25
	pub unsafe fn set_stamina(stamina: f32) -> std::result::Result<(), &'static str> {
		PLAYER_STATE.player.health.set_current_stamina(stamina)
	}

	/// Get the current Damage (red health) of the player.
	///
	/// Minimum value: 0.0,
	/// Maximum value: 0.25
	pub unsafe fn get_damage() -> std::result::Result<f32, &'static str> {
		PLAYER_STATE.player.health.get_damage()
	}

	/// Set the current Damage (red health) of the player.
	///
	/// Minimum value: 0.0,
	/// Maximum value: 0.25
	pub unsafe fn set_damage(stamina: f32) -> std::result::Result<(), &'static str> {
		PLAYER_STATE.player.health.set_damage(stamina)
	}

	/// Get the current Focus (blue bar) of the player.
	///
	/// Minimum value: 0.0
	/// Maximum value: 1.0
	pub unsafe fn get_current_focus() -> std::result::Result<f32, &'static str> {
		PLAYER_STATE.player.health.get_current_focus()
	}

	/// Set the current Focus (blue bar) of the player.
	///
	/// Minimum value: 0.0
	/// Maximum value: 1.0
	pub unsafe fn set_current_focus(stamina: f32) -> std::result::Result<(), &'static str> {
		PLAYER_STATE.player.health.set_current_focus(stamina)
	}
}

pub unsafe fn init_api() {
	// let error_handler: PVECTORED_EXCEPTION_HANDLER = Some(exceptions::error_handler);
	// AddVectoredExceptionHandler(1, error_handler);
	PLAYER_STATE.player.health.init();
	// let view = PE32::get_module_information();
	// dbg!(view.optional_header());
	println!("[EMF] API Initialized");

	if let Err(e) = lua::init_lua() {
		println!("[EMF] LuaJIT Failed to Initialize");
		eprint!("{:?}", e);
	} else {
		println!("[EMF] LuaJIT Initialized");
	}
}
