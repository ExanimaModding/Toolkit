use winapi::shared::minwindef::DWORD;

use crate::memory::{sigscanner::SigScanner, Ptr};
use std::result::Result;

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
        *current_stamina = health;
        Ok(())
    }

    //     let max_health: *mut f32 = Ptr::offset(health_ptr as DWORD, 0x0b44);
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

    //     let red: *mut f32 = Ptr::offset(health_ptr as DWORD, 0x0b34);
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
}

#[allow(unused)]
impl Player {
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
    PLAYER_STATE.player.health.init();
}
