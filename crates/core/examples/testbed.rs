use std::{
	fs,
	io::{self, Write},
	path::PathBuf,
};

use emcore::prelude::*;
use tracing::error;

fn main() {
	let file_appender = tracing_appender::rolling::never(
		std::path::absolute(PathBuf::from("./")).unwrap(),
		"testing.log",
	);
	let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
	tracing_subscriber::fmt()
		.with_max_level(tracing::Level::DEBUG)
		.with_writer(non_blocking)
		.init();
}

#[allow(dead_code)]
async fn testing_instance() {
	let path = PathBuf::from("C:/Program Files (x86)/Steam/steamapps/common/Exanima");
	let instance = Instance::with_path(path).unwrap().build().await.unwrap();
	dbg!(&instance);
}

#[allow(dead_code)]
async fn testing_profile() {
	if let Err(profile::error::Builder::LoadOrder(e)) =
		Profile::with_path("./Test/.emtk/profiles/TestProfile").await
	{
		error!("{}", e.to_string());
	};
}

#[allow(dead_code)]
fn testing_cache_metadata() {
	let file_path = std::path::absolute(PathBuf::from("metadata.ron"))
		.unwrap()
		.canonicalize()
		.unwrap();
	let file = fs::File::create("metadata.ron").unwrap();
}
