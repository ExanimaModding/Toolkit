#![feature(raw_ref_op)]

mod patches;
mod utils;

use crate::{
    patches::PATCHES,
    utils::{patch_apply, patch_revert},
};

use log::*;

#[no_mangle]
pub extern "C" fn enable() -> bool {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    unsafe {
        register_hooks();
    }

    true
}

#[no_mangle]
pub extern "C" fn disable() -> bool {
    unsafe {
        for (name, patch) in PATCHES.iter_mut() {
            if patch_revert(patch) {
                info!("Reverted patch: {}", name);
            } else {
                error!("Failed to revert patch: {}", name);
            }
        }
    }
    true
}

#[no_mangle]
pub extern "C" fn on_init() -> bool {
    true
}

unsafe fn register_hooks() {
    println!("Registering hooks");

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

    for (name, patch) in PATCHES.iter_mut() {
        if patch_apply(patch) {
            info!("Applied patch: {}", name);
        } else {
            error!("Failed to apply patch: {}", name);
        }
    }
}
