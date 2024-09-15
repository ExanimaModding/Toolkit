pub static CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The animation duration for fade transitions in milliseconds.
pub static FADE_DURATION: u64 = 100;

pub static ARROW_LEFT: &[u8] = include_bytes!("../../../../assets/images/arrow-left.svg");
pub static FOLDER: &[u8] = include_bytes!("../../../../assets/images/folder.svg");
pub static LAYERS: &[u8] = include_bytes!("../../../../assets/images/layers-3.svg");
pub static PLAY: &[u8] = include_bytes!("../../../../assets/images/play.svg");
pub static SETTINGS: &[u8] = include_bytes!("../../../../assets/images/settings.svg");
pub static SQUARE_ARROW_OUT: &[u8] =
	include_bytes!("../../../../assets/images/square-arrow-out-up-right.svg");
