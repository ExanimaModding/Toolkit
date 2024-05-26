use super::config::PluginConfig;
use anyhow::*;

pub fn parse_plugin_config(config: &str) -> Result<PluginConfig> {
	Ok(toml::from_str(config)?)
}
