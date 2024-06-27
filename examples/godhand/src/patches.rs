use crate::utils::{
    patch_from_signature, patch_new, patch_offset_pointer, read_bytes,
    reassemble_instruction_at_offset, scan_memory, Patch,
};
use safer_ffi::prelude::repr_c;

// These patches were taken from Horse4Horse's God Hand Cheat Table.
// This is not indented to be used in stable gameplay, but rather
// as a proof-of-concept for modding with the Exanima Modding Framework.

pub static mut PATCHES: Vec<(String, repr_c::Box<Patch>)> = Vec::new();

pub unsafe fn ignore_range_limit_for_placement() -> Option<repr_c::Box<Patch>> {
    let signature = "48 8B 40 10 48 8D 48 20";
    let bytes = vec![
        0x66_u8, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00, 0x66, 0x0F, 0x1F, 0x84, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x45, 0x31, 0xED,
    ];

    patch_from_signature(signature.into(), bytes.into())
}

pub unsafe fn ignore_range_limit_for_reach() -> Option<repr_c::Box<Patch>> {
    let signature = "EB 0E 4C 89 E1";
    let bytes = vec![
        0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00, 0x66, 0x90, 0x45, 0x31, 0xED,
    ];

    let patch = patch_from_signature(signature.into(), bytes.into());

    if let Some(mut patch) = patch {
        patch_offset_pointer(&mut patch, 0x2);
        Some(patch)
    } else {
        None
    }
}

pub unsafe fn ignore_range_limit_for_door_and_lever_reach() -> Option<repr_c::Box<Patch>> {
    let signature = "E9 ?? ?? ?? ?? 48 8B 05 ?? ?? ?? ?? 8B 40 ?? 25 ?? ?? ?? ?? ?? ?? 48 8B 05 ?? ?? ?? ?? 48 8B 40 ?? 48 8B 80 ?? ?? ?? ?? 48 8D";

    let ptr = scan_memory(signature.into());
    if ptr.is_null() {
        return None;
    }

    let instruction = read_bytes(ptr as _, 0x5).to_owned();
    let instruction = reassemble_instruction_at_offset(instruction, 0xF).to_owned();

    let mut patch = patch_new(ptr as _, instruction);
    patch_offset_pointer(&mut patch, 0xF);

    Some(patch)
}

pub unsafe fn ignore_weight() -> Option<repr_c::Box<Patch>> {
    let signature = "25 ?? ?? ?? ?? 75 ?? F3 0F ?? ?? ?? ?? ?? ?? 66 0F ?? ?? ?? ?? ?? ?? 7A";
    let bytes = vec![0xEB];

    let patch = patch_from_signature(signature.into(), bytes.into());

    if let Some(mut patch) = patch {
        patch_offset_pointer(&mut patch, 0x19);
        Some(patch)
    } else {
        None
    }
}

pub unsafe fn interact_while_fallen() -> Option<repr_c::Box<Patch>> {
    let signature = "74 ?? 48 8B 05 ?? ?? ?? ?? 8B 80 ?? ?? ?? ?? 25 ?? ?? ?? ?? 0F 87 ?? ?? ?? ??";
    let bytes = vec![0x90, 0x90, 0x90, 0x90, 0x90, 0x90];

    let patch = patch_from_signature(signature.into(), bytes.into());

    if let Some(mut patch) = patch {
        patch_offset_pointer(&mut patch, 0x14);
        Some(patch)
    } else {
        None
    }
}

pub unsafe fn dont_interrupt_if_fallen() -> Option<repr_c::Box<Patch>> {
    let signature = "48 8B 05 ?? ?? ?? ?? 8B 80 ?? ?? ?? ?? 25 ?? ?? ?? ?? ?? ?? E8";
    let bytes = vec![0xEB];

    let patch = patch_from_signature(signature.into(), bytes.into());

    if let Some(mut patch) = patch {
        patch_offset_pointer(&mut patch, 0x12);
        Some(patch)
    } else {
        None
    }
}
