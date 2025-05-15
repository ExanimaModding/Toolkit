use emcore::ErrorKind;

pub(crate) mod registry;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("{0}")]
	Mlua(#[from] mlua::Error),
	#[error("'{0}' does not return a lua table")]
	NoTableReturned(String),
	#[error("{0} does not exist in registry")]
	UnregisteredId(String),
}

impl From<Error> for ErrorKind {
	fn from(value: Error) -> Self {
		ErrorKind::Other(anyhow::Error::new(value))
	}
}
