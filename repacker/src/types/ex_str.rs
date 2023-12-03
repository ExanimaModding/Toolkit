// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct ExanimaString([u8; 16]);

impl std::fmt::Debug for ExanimaString {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let str = match std::str::from_utf8(&self.0) {
			Ok(str) => str,
			Err(error) => panic!("Couldn't convert ExanimaString to string: {:?}", error),
		};

		write!(f, "{}", str)
	}
}

impl std::convert::TryFrom<String> for ExanimaString {
	type Error = &'static str;

	fn try_from(s: String) -> std::result::Result<Self, Self::Error> {
		if s.len() > 16 {
			return Err("String exceeds 16 characters");
		}
		let mut name = ExanimaString([0; 16]);
		for (i, c) in s.chars().enumerate() {
			name.0[i] = c as u8;
		}

		Ok(name)
	}
}

impl std::convert::TryFrom<ExanimaString> for String {
	type Error = String;

	fn try_from(s: ExanimaString) -> std::result::Result<Self, Self::Error> {
		let mut name = String::new();
		for i in 0..16 {
			if s.0[i] == 0 {
				return Ok(name);
			}
			name.push(s.0[i] as char);
		}

		Ok(name)
	}
}
