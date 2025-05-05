use std::{
	collections::HashMap,
	env, fs,
	sync::{Arc, LazyLock, Mutex, OnceLock},
};

use emcore::plugin::Manifest;
use mlua::{Function, Lua, Table, Value, Variadic};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Invalid plugin ID '{0}'")]
	InvalidId(String),
	#[error("No plugin.lua for plugin '{0}'")]
	NoPluginLuaFile(String),
	#[error("{0}")]
	PluginCrashed(#[from] mlua::Error),
	#[error("Plugin '{0}' had no returned table")]
	NoTableReturned(String),
	#[error("Failed to get manifest from plugin")]
	Manifest,
	#[error("Missing key '{0}' in manifest for plugin '{1}")]
	MissingManifestKey(String, String),
}

#[derive(Debug, Clone)]
pub struct PluginRegistry {
	plugins: HashMap<String, Plugin>,
}

#[derive(Debug, Clone)]
pub struct Plugin {
	#[allow(unused)]
	manifest: Manifest,
	onstart: Option<Function>,
	/// TODO: Implement this
	onstop: Option<Function>,
	exports: Option<Table>,
}

impl PluginRegistry {
	fn register_plugin(&mut self, name: String, plugin: Plugin) {
		self.plugins.insert(name, plugin);
	}

	pub fn get_plugin(&self, name: &str) -> Option<Plugin> {
		self.plugins.get(name).cloned()
	}

	pub fn start(id: &str) -> Result<(), Error> {
		let plugin = REGISTRY
			.lock()
			.unwrap()
			.get_plugin(id)
			.ok_or(Error::InvalidId(id.to_string()))?;
		if let Some(onstart) = &plugin.onstart {
			onstart.call::<()>(())?;
		}
		Ok(())
	}

	pub fn stop(id: &str) -> Result<(), Error> {
		let plugin = REGISTRY
			.lock()
			.unwrap()
			.get_plugin(id)
			.ok_or(Error::InvalidId(id.to_string()))?;
		if let Some(onstop) = &plugin.onstop {
			onstop.call::<()>(())?;
		}
		Ok(())
	}

	pub fn load_plugin(plugin_name: &str) -> Result<(), Error> {
		let lua = &*LUA;

		// TODO: Improve this
		let cwd = env::current_exe().expect("Failed to get current executable");
		let cwd = cwd.parent().expect("Failed to get current directory");

		let plugin_str = fs::read_to_string(cwd.join(format!("mods/{}/plugin.lua", plugin_name)))
			.map_err(|_| Error::NoPluginLuaFile(plugin_name.to_string()))?;

		let environment = lua.create_table().expect("Failed to create lua table");

		for global in lua.globals().pairs::<String, Value>() {
			if let Ok((key, value)) = global {
				environment
					.set(key, value)
					.expect("Failed to set global in lua table");
			}
		}

		let response: Option<Table> = lua
			.load(&plugin_str)
			.set_environment(environment.clone())
			.eval()
			.map_err(Error::PluginCrashed)?;

		let Some(response) = response else {
			return Err(Error::NoTableReturned(plugin_name.to_string()));
		};

		let manifest = {
			let table = response
				.get::<Table>("manifest")
				.map_err(|_| Error::Manifest)?;

			parse_manifest(plugin_name, table)?
		};

		let exports: Option<Table> = response.get("exports").ok();

		let onstart = response.get("onstart").ok();
		let onstop = response.get("onstop").ok();

		environment.set("_G", environment.clone())?;

		let plugin = Plugin {
			manifest,
			exports,
			onstart,
			onstop,
		};

		REGISTRY
			.lock()
			.unwrap()
			.register_plugin(plugin_name.to_string(), plugin);

		Ok(())
	}
}

static LUA: LazyLock<Lua> = LazyLock::new(|| unsafe {
	let lua = Lua::unsafe_new();

	lua_hook_require(&lua);

	lua_hook_print(&lua);

	lua
});

static REGISTRY: LazyLock<Arc<Mutex<PluginRegistry>>> = LazyLock::new(|| {
	Arc::new(Mutex::new(PluginRegistry {
		plugins: HashMap::new(),
	}))
});

fn lua_hook_require(lua: &Lua) {
	static ORIGINAL_REQUIRE: OnceLock<Function> = OnceLock::new();

	ORIGINAL_REQUIRE.get_or_init(|| lua.globals().get::<Function>("require").unwrap());

	lua.globals()
		.set(
			"require",
			lua.create_function(|_, name: String| {
				if name.starts_with("plugin:") {
					let name = name.strip_prefix("plugin:").unwrap();

					if let Some(plugin) = REGISTRY.lock().unwrap().get_plugin(name) {
						if let Some(exports) = plugin.exports {
							Ok(Value::Table(exports))
						} else {
							Err(mlua::Error::RuntimeError(format!(
								"No exports for plugin {}",
								name
							)))
						}
					} else {
						Err(mlua::Error::RuntimeError(format!(
							"No plugin named {}",
							name
						)))
					}
				} else {
					ORIGINAL_REQUIRE.get().unwrap().call(name)
				}
			})
			.unwrap(),
		)
		.unwrap();
}

fn lua_hook_print(lua: &Lua) {
	let print_fn = lua
		.create_function(|_, args: Variadic<Value>| {
			let mut first = true;

			for arg in args {
				if !first {
					print!(" ");
				}
				first = false;

				match arg {
					Value::String(s) => print!("{}", s.to_string_lossy()),
					other => print!("{:?}", other),
				}
			}
			println!();
			Ok(())
		})
		.expect("FATAL ERROR: Failed to create Lua print function.");

	lua.globals()
		.set("print", print_fn)
		.expect("FATAL ERROR: Failed to set Lua print function.");

	let io = lua
		.globals()
		.get::<Table>("io")
		.expect("FATAL ERROR: Failed to get Lua io table.");

	// TODO: Does this need to support writing to anything other than stdout?
	let io_write_fn = lua
		.create_function(|_, args: Variadic<Value>| {
			for arg in args {
				match arg {
					Value::String(s) => print!("{}", s.to_string_lossy()),
					other => print!("{}", format!("{:?}", other)), // Write other types as their debug representation
				}
			}
			Ok(())
		})
		.expect("FATAL ERROR: Failed to create Lua io.write function.");

	io.set("write", io_write_fn)
		.expect("FATAL ERROR: Failed to set Lua io.write function.");
}

pub fn parse_manifest(plugin_name: &str, table: Table) -> Result<Manifest, Error> {
	macro_rules! manifest_key {
		($manifest:ident, $name:ident, $type:ty) => {{
			let Ok($name) = $manifest.get::<$type>(stringify!($name)) else {
				return Err(Error::MissingManifestKey(
					stringify!($name).to_string(),
					plugin_name.to_string(),
				));
			};
			$name
		}};
	}

	let Ok(id) = emcore::plugin::Id::try_from(plugin_name) else {
		return Err(Error::InvalidId(plugin_name.to_string()));
	};
	let name = manifest_key!(table, name, String);
	let version = manifest_key!(table, version, String);
	let author = manifest_key!(table, author, String);

	let dependencies = table.get::<Option<Vec<String>>>("dependencies").unwrap();

	let dependencies = dependencies
		.unwrap_or_default()
		.into_iter()
		.map(emcore::plugin::Id::try_from)
		.collect::<Result<Vec<_>, _>>()
		.unwrap();

	let conflicts = table.get::<Option<Vec<String>>>("conflicts").unwrap();

	let conflicts = conflicts
		.unwrap_or_default()
		.into_iter()
		.map(emcore::plugin::Id::try_from)
		.collect::<Result<Vec<_>, _>>()
		.unwrap();

	Ok(Manifest {
		id,
		name,
		version,
		author,
		dependencies,
		conflicts,
	})
}
