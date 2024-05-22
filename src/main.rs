// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only
#![allow(clippy::missing_safety_doc)]

pub mod injector;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	unsafe {
		injector::inject(
			r"emf.dll",
			r"m:\Games\Steam Library\steamapps\common\Exanima\Exanima.exe",
		)
		.unwrap();
	}
	Ok(())
}
