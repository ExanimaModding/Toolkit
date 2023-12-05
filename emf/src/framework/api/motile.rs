// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

use winapi::shared::minwindef::DWORD;

use crate::internal::memory::Ptr;

pub const PTR_STAMINA: DWORD = 0xb30;
pub const PTR_MAX_STAMINA: DWORD = 0xb34;
pub const PTR_SHIELD_STAMINA: DWORD = 0xb38;
pub const PTR_SHIELD_STAMINA_RECOVERY: DWORD = 0xb3c;
pub const PTR_SHIELD_RECOVERY_TIME: DWORD = 0xb40;
pub const PTR_P_STAMINA: DWORD = 0xb44;
pub const PTR_V_STAMINA: DWORD = 0xb48;
pub const PTR_CHARACTER_FOCUS: DWORD = 0xb4c;
pub const PTR_MAX_FOCUS: DWORD = 0xb50;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Motile {
	ptr: DWORD,
	stamina: f32,
	max_stamina: f32,
	shield_stamina: f32,
	shield_stamina_recovery: f32,
	shield_recovery_time: f32,
	p_stamina: f32,
	v_stamina: f32,
	character_focus: f32,
	max_focus: f32,
}

#[allow(dead_code)]
impl Motile {
	pub unsafe fn get_from_ptr(ptr: DWORD) -> Option<Self> {
		if Ptr::as_const::<*mut f32>(ptr).is_null() {
			return None;
		}
		Some(Self {
			ptr,
			stamina: *Ptr::offset::<f32>(ptr, PTR_STAMINA as _),
			max_stamina: *Ptr::offset::<f32>(ptr, PTR_MAX_STAMINA as _),
			shield_stamina: *Ptr::offset::<f32>(ptr, PTR_SHIELD_STAMINA as _),
			shield_stamina_recovery: *Ptr::offset::<f32>(ptr, PTR_SHIELD_STAMINA_RECOVERY as _),
			shield_recovery_time: *Ptr::offset::<f32>(ptr, PTR_SHIELD_RECOVERY_TIME as _),
			p_stamina: *Ptr::offset::<f32>(ptr, PTR_P_STAMINA as _),
			v_stamina: *Ptr::offset::<f32>(ptr, PTR_V_STAMINA as _),
			character_focus: *Ptr::offset::<f32>(ptr, PTR_CHARACTER_FOCUS as _),
			max_focus: *Ptr::offset::<f32>(ptr, PTR_MAX_FOCUS as _),
		})
	}

	pub unsafe fn refresh(&mut self) -> Result<&Self, String> {
		let motile = Motile::get_from_ptr(self.ptr);
		if let Some(motile) = motile {
			self.stamina = motile.stamina;
			self.max_stamina = motile.max_stamina;
			self.shield_stamina = motile.shield_stamina;
			self.shield_stamina_recovery = motile.shield_stamina_recovery;
			self.shield_recovery_time = motile.shield_recovery_time;
			self.p_stamina = motile.p_stamina;
			self.v_stamina = motile.v_stamina;
			self.character_focus = motile.character_focus;
			self.max_focus = motile.max_focus;
			return Ok(self);
		}
		Err("Failed to read motile data".to_string())
	}

	pub unsafe fn set_stamina(&mut self, stamina: f32) {
		*Ptr::offset::<f32>(self.ptr, PTR_STAMINA as _) = stamina;
		self.stamina = stamina;
	}

	pub unsafe fn set_max_stamina(&mut self, max_stamina: f32) {
		*Ptr::offset::<f32>(self.ptr, PTR_MAX_STAMINA as _) = max_stamina;
	}

	pub unsafe fn set_shield_stamina(&mut self, shield_stamina: f32) {
		*Ptr::offset::<f32>(self.ptr, PTR_SHIELD_STAMINA as _) = shield_stamina;
	}

	pub unsafe fn set_shield_stamina_recovery(&mut self, shield_stamina_recovery: f32) {
		*Ptr::offset::<f32>(self.ptr, PTR_SHIELD_STAMINA_RECOVERY as _) = shield_stamina_recovery;
	}

	pub unsafe fn set_shield_recovery_time(&mut self, shield_recovery_time: f32) {
		*Ptr::offset::<f32>(self.ptr, PTR_SHIELD_RECOVERY_TIME as _) = shield_recovery_time;
	}

	pub unsafe fn set_p_stamina(&mut self, p_stamina: f32) {
		*Ptr::offset::<f32>(self.ptr, PTR_P_STAMINA as _) = p_stamina;
	}

	pub unsafe fn set_v_stamina(&mut self, v_stamina: f32) {
		*Ptr::offset::<f32>(self.ptr, PTR_V_STAMINA as _) = v_stamina;
	}

	pub unsafe fn set_character_focus(&mut self, character_focus: f32) {
		*Ptr::offset::<f32>(self.ptr, PTR_CHARACTER_FOCUS as _) = character_focus;
	}

	pub unsafe fn set_max_focus(&mut self, max_focus: f32) {
		*Ptr::offset::<f32>(self.ptr, PTR_MAX_FOCUS as _) = max_focus;
	}

	pub fn stamina(&self) -> f32 {
		self.stamina
	}

	pub fn max_stamina(&self) -> f32 {
		self.max_stamina
	}

	pub fn shield_stamina(&self) -> f32 {
		self.shield_stamina
	}

	pub fn shield_stamina_recovery(&self) -> f32 {
		self.shield_stamina_recovery
	}

	pub fn shield_recovery_time(&self) -> f32 {
		self.shield_recovery_time
	}

	pub fn p_stamina(&self) -> f32 {
		self.p_stamina
	}

	pub fn v_stamina(&self) -> f32 {
		self.v_stamina
	}

	pub fn character_focus(&self) -> f32 {
		self.character_focus
	}

	pub fn max_focus(&self) -> f32 {
		self.max_focus
	}
}
