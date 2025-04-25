#![allow(clippy::missing_safety_doc)]

mod patches;
mod utils;

use std::sync::Once;

use tracing::{error, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use utils::{get_setting_bool, patch_is_applied};

use crate::{
	patches::PATCHES,
	utils::{patch_apply, patch_revert},
};

static mut FIRST_RUN: bool = true;
static mut PLUGIN_ID: &str = "dev.megu.godhand";

#[no_mangle]
pub unsafe extern "C" fn enable() -> bool {
	if FIRST_RUN {
		tracing_subscriber::registry()
			.with(
				fmt::layer().with_filter(
					EnvFilter::builder()
						.from_env()
						.unwrap()
						.add_directive("godhand=debug".parse().unwrap()),
				),
			)
			.init();
		FIRST_RUN = false;
	}

	register_hooks();

	true
}

#[no_mangle]
pub extern "C" fn disable() -> bool {
	unsafe {
		for (name, patch) in PATCHES.iter_mut() {
			if patch_is_applied(patch) {
				if patch_revert(patch) {
					info!("Reverted patch: {}", name);
				} else {
					error!("Failed to revert patch: {}", name);
				}
			}
		}
	}
	true
}

#[no_mangle]
pub extern "C" fn on_init() -> bool {
	true
}

static FIRST_REGISTER: Once = Once::new();

unsafe fn register_hooks() {
	println!("Registering hooks");

	FIRST_REGISTER.call_once(|| {
		macro_rules! add_patch_to_map {
			($patch:expr, $name:expr) => {
				if let Some(patch) = $patch {
					PATCHES.push(($name.into(), patch));
				} else {
					println!("Failed to register patch: {}", $name);
				}
			};
		}

		add_patch_to_map!(
			patches::ignore_range_limit_for_placement(),
			"ignore_range_limit_for_placement"
		);

		add_patch_to_map!(
			patches::ignore_range_limit_for_reach(),
			"ignore_range_limit_for_reach"
		);

		add_patch_to_map!(
			patches::ignore_range_limit_for_door_and_lever_reach(),
			"ignore_range_limit_for_door_and_lever_reach"
		);

		add_patch_to_map!(patches::ignore_weight(), "ignore_weight");

		add_patch_to_map!(patches::interact_while_fallen(), "interact_while_fallen");

		add_patch_to_map!(
			patches::dont_interrupt_if_fallen(),
			"dont_interrupt_if_fallen"
		);
	});

	let (
		enable_range_limit_for_placement,
		enable_range_limit_for_reach,
		enable_range_limit_for_door_and_lever_reach,
		enable_weight,
		enable_interact_while_fallen,
		enable_dont_interrupt_if_fallen,
	) = (
		get_setting_bool(PLUGIN_ID.into(), "range_limit_for_placement".into()),
		get_setting_bool(PLUGIN_ID.into(), "range_limit_for_reach".into()),
		get_setting_bool(
			PLUGIN_ID.into(),
			"range_limit_for_door_and_lever_reach".into(),
		),
		get_setting_bool(PLUGIN_ID.into(), "weight".into()),
		get_setting_bool(PLUGIN_ID.into(), "interact_while_fallen".into()),
		get_setting_bool(PLUGIN_ID.into(), "dont_interrupt_if_fallen".into()),
	);

	macro_rules! is_enabled {
		($setting:expr) => {
			$setting.found && $setting.value
		};
	}

	for (name, patch) in PATCHES.iter_mut() {
		let apply = match name.as_str() {
			"ignore_range_limit_for_placement" => is_enabled!(enable_range_limit_for_placement),
			"ignore_range_limit_for_reach" => is_enabled!(enable_range_limit_for_reach),
			"ignore_range_limit_for_door_and_lever_reach" => {
				is_enabled!(enable_range_limit_for_door_and_lever_reach)
			}
			"ignore_weight" => is_enabled!(enable_weight),
			"interact_while_fallen" => is_enabled!(enable_interact_while_fallen),
			"dont_interrupt_if_fallen" => is_enabled!(enable_dont_interrupt_if_fallen),
			_ => false,
		};
		if apply && patch_apply(patch) {
			info!("Applied patch: {}", name);
		} else {
			error!("Failed to apply patch: {}", name);
		}
	}
}
