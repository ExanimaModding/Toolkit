// Exanima Modding Toolkit
// Copyright (C) 2023 ProffDea <deatea@riseup.net>, Megumin <megumin@megu.dev>
// SPDX-License-Identifier: GPL-3.0-only

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_string_to_ex_str() {
		let test = ExanimaString::try_from(String::from("7 chars"));
		let expect = ExanimaString([55, 32, 99, 104, 97, 114, 115, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

		assert!(test.is_ok());
		assert_eq!(test.unwrap(), expect);
	}

	#[test]
	fn test_ex_str_to_string() {
		let test = String::try_from(ExanimaString([
			55, 32, 99, 104, 97, 114, 115, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		]));
		let expect = String::from("7 chars");

		assert!(test.is_ok());
		assert_eq!(test.unwrap(), expect);
	}

	#[test]
	fn test_empty_string() {
		let test = ExanimaString::try_from(String::new());
		let expect = ExanimaString([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

		assert!(test.is_ok());
		assert_eq!(test.unwrap(), expect);
	}

	#[test]
	fn test_empty_ex_str() {
		let test = String::try_from(ExanimaString([
			0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		]));
		let expect = String::new();

		assert!(test.is_ok());
		assert_eq!(test.unwrap(), expect);
	}

	#[test]
	fn test_max_len() {
		let test = ExanimaString::try_from(String::from("This is 16 chars"));
		let expect = ExanimaString([
			84, 104, 105, 115, 32, 105, 115, 32, 49, 54, 32, 99, 104, 97, 114, 115,
		]);

		assert!(test.is_ok());
		assert_eq!(test.unwrap(), expect);
	}

	#[test]
	fn test_bad_string() {
		let test = ExanimaString::try_from(String::from("This is 17 chars."));

		assert!(test.is_err());
	}
}
