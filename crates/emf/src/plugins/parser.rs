use anyhow::*;
use emf_types::rust::config::PluginConfig;

pub fn parse_plugin_config(config: &str) -> Result<PluginConfig> {
	let mut config: PluginConfig = toml::from_str(config)?;

	// If the value is not set, use the default value instead.
	for setting in &mut config.settings {
		if setting.value.is_none() {
			setting.value = Some(setting.default.clone());
		}
	}

	Ok(config)
}
