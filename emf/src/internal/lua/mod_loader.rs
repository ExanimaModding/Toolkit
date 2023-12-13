use std::{ffi::CString, ptr::null_mut};

use mlua::Result as LuaResult;
use winapi::{
	shared::minwindef::MAX_PATH,
	um::{
		processenv::GetCurrentDirectoryA,
		processthreadsapi::GetCurrentProcess,
		psapi::{GetMappedFileNameA, GetModuleFileNameExA},
	},
};

use crate::internal::utils::get_game_dir;

use super::luaRuntime;

pub fn strip_types_import(script: &str) -> String {
	let script = script.to_string();

	let mut lines = script.lines().collect::<Vec<_>>();

	for line in lines.iter_mut() {
		if *line == "require(\"types\")" {
			*line = "";
		}
	}

	lines.join("\n")
}

pub fn exec_lua_file(path: &str) -> LuaResult<()> {
	let lua = unsafe { luaRuntime.get() };

	let script = std::fs::read_to_string(path)?;

	let script = strip_types_import(&script);

	let script = lua.load(&script);

	script.exec()?;

	Ok(())
}

pub fn get_mods_list() -> Result<Vec<String>, std::io::Error> {
	let mut scripts = Vec::new();

	let game_dir = get_game_dir();

	let mods_dir = game_dir.join("mods");

	if !mods_dir.exists() {
		std::fs::create_dir(&mods_dir)?;
	}

	let mods = std::fs::read_dir(&mods_dir)?.filter(|entry| {
		let entry = entry.as_ref().unwrap();
		let path = entry.path();

		if path.is_file() {
			let extension = path.extension().unwrap().to_str().unwrap();

			if extension == "lua" {
				return true;
			}
		}

		false
	});

	for entry in mods {
		let entry = entry?;
		let path = entry.path();

		if path.is_file() {
			scripts.push(path.to_str().unwrap().to_string());
		}
	}

	Ok(scripts)
}
