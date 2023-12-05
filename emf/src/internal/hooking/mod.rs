pub mod hooks;

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
#[allow(unused)]
pub enum HookName {
	Internal(String),
	Framework(String),
	User(String),
}

#[allow(unused)]
impl HookName {
	pub fn internal(source: &str, name: &str) -> Self {
		Self::Internal(format!("{}::{}", source, name))
	}

	pub fn framework(source: &str, name: &str) -> Self {
		Self::Framework(format!("{}::{}", source, name))
	}

	pub fn user(plugin: &str, name: &str) -> Self {
		Self::User(format!("{}::{}", plugin, name))
	}
}

impl std::fmt::Display for HookName {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Internal(s) => write!(f, "Internal::{}", s),
			Self::Framework(s) => write!(f, "Framework::{}", s),
			Self::User(s) => write!(f, "User::{}", s),
		}
	}
}
