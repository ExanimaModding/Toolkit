use log::*;
use safer_ffi::prelude::*;

use crate::plugins::manager::PluginManager;
use emf_types::{ffi::GetSettingReturnValue, rust::config::PluginConfigSettingValue};

#[ffi_export]
pub fn get_setting_bool(
	id: repr_c::String,
	key: repr_c::String,
) -> repr_c::Box<GetSettingReturnValue<bool>> {
	let id = id.to_string().clone();
	let key = key.to_string().clone();

	let info = PluginManager::get_info_for(&id);

	if let Some(info) = info {
		let setting = info.config.settings.iter().find(|s| s.id == key);

		if let Some(setting) = setting {
			match setting.clone().value {
				Some(PluginConfigSettingValue::Boolean(value)) => {
					return Box::new(GetSettingReturnValue { value, found: true }).into();
				}
				Some(_) => {
					warn!("Setting {}::{} is not a boolean", id, key);
				}
				None => {}
			}
		}
	}

	Box::new(GetSettingReturnValue {
		value: false,
		found: false,
	})
	.into()
}

#[ffi_export]
pub fn get_setting_string(
	id: repr_c::String,
	key: repr_c::String,
) -> GetSettingReturnValue<char_p::Box> {
	let id = id.to_string();
	let key = key.to_string();

	let info = PluginManager::get_info_for(&id);

	if let Some(info) = info {
		let setting = info.config.settings.iter().find(|s| s.id == key);

		if let Some(setting) = setting {
			match setting.clone().value {
				Some(PluginConfigSettingValue::String(value)) => {
					return GetSettingReturnValue {
						value: char_p::new(value),
						found: true,
					};
				}
				Some(_) => {
					warn!("Setting {} is not a string", key);
				}
				None => {}
			}
		}
	}

	GetSettingReturnValue {
		value: char_p::new(""),
		found: false,
	}
}

#[ffi_export]
pub fn get_setting_integer(id: repr_c::String, key: repr_c::String) -> GetSettingReturnValue<i64> {
	let id = id.to_string();
	let key = key.to_string();

	let info = PluginManager::get_info_for(&id);

	if let Some(info) = info {
		let setting = info.config.settings.iter().find(|s| s.id == key);

		if let Some(setting) = setting {
			match setting.clone().value {
				Some(PluginConfigSettingValue::Integer(value)) => {
					return GetSettingReturnValue { value, found: true };
				}
				Some(_) => {
					warn!("Setting {} is not an integer", key);
				}
				None => {}
			}
		}
	}

	GetSettingReturnValue {
		value: 0,
		found: false,
	}
}

#[ffi_export]
pub fn get_setting_float(id: repr_c::String, key: repr_c::String) -> GetSettingReturnValue<f64> {
	let id = id.to_string();
	let key = key.to_string();

	let info = PluginManager::get_info_for(&id);

	if let Some(info) = info {
		let setting = info.config.settings.iter().find(|s| s.id == key);

		if let Some(setting) = setting {
			match setting.clone().value {
				Some(PluginConfigSettingValue::Float(value)) => {
					return GetSettingReturnValue { value, found: true };
				}
				Some(_) => {
					warn!("Setting {} is not a float", key);
				}
				None => {}
			}
		}
	}

	GetSettingReturnValue {
		value: 0.,
		found: false,
	}
}
