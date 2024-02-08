use crate::internal::utils::get_game_dir;

mod hooking;

#[allow(unused)]
fn get_mods_dir() -> Result<std::path::PathBuf, std::io::Error> {
	let game_dir = get_game_dir();
	let mods_dir = game_dir.join("mods");
	if !mods_dir.exists() {
		std::fs::create_dir(&mods_dir)?;
	}
	dbg!(&mods_dir);
	Ok(mods_dir)
}
