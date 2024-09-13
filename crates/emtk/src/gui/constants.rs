pub static CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The animation duration for fade transitions in milliseconds.
pub static FADE_DURATION: u64 = 100;
pub static SQUARE_ARROW_OUT: &[u8] =
	include_bytes!("../../../../assets/images/square-arrow-out-up-right.svg");
