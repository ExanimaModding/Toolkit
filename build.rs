// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

pub fn main() {
    println!("cargo:rustc-link-lib=kernel32");
    println!("cargo:rustc-link-lib=ucrt");
}
