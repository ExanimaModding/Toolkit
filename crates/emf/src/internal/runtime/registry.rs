use std::{
	collections::HashMap,
	env, fs,
	sync::{Arc, LazyLock, Mutex, OnceLock},
};

use emcore::{Error, Result, plugin};
use mlua::{Function, Lua, Table, Value, Variadic};
use tracing::instrument;

use crate::runtime;

#[derive(Debug, Clone)]
pub struct PluginRegistry {
	plugins: HashMap<plugin::Id, Plugin>,
}

#[derive(Debug, Clone)]
pub struct Plugin {
	#[allow(unused)]
	manifest: plugin::Manifest,
	onstart: Option<Function>,
	/// TODO: Implement this
	onstop: Option<Function>,
	exports: Option<Table>,
}

impl PluginRegistry {
	#[instrument(level = "trace")]
	fn register_plugin(&mut self, id: plugin::Id, plugin: Plugin) {
		self.plugins.insert(id, plugin);
	}

	#[instrument(level = "trace")]
	pub fn start(id: &plugin::Id) -> Result<()> {
		let plugin = REGISTRY
			.lock()
			.unwrap()
			.plugins
			.get(id)
			.cloned()
			.ok_or(runtime::Error::UnregisteredId(id.to_string()))
			.map_err(Error::msg("failed to get plugin from registry"))?;
		if let Some(onstart) = &plugin.onstart {
			onstart
				.call::<()>(())
				.map_err(runtime::Error::from)
				.map_err(Error::msg("failed to call lua plugin's onstart function"))?;
		}
		Ok(())
	}

	#[instrument(level = "trace")]
	pub fn stop(id: &plugin::Id) -> Result<()> {
		let plugin = REGISTRY
			.lock()
			.unwrap()
			.plugins
			.get(id)
			.cloned()
			.ok_or(runtime::Error::UnregisteredId(id.to_string()))
			.map_err(Error::msg("failed to get plugin from registry"))?;
		if let Some(onstop) = &plugin.onstop {
			onstop
				.call::<()>(())
				.map_err(runtime::Error::from)
				.map_err(Error::msg("failed to call lua plugin's onstop function"))?;
		}
		Ok(())
	}

	#[instrument(level = "trace")]
	pub fn load_plugin(plugin_name: &str) -> Result<()> {
		let id = plugin::Id::try_from(plugin_name)
			.map_err(Error::msg("failed to get id while loading plugin"))?;
		let lua = &*LUA;

		// TODO: Improve this
		let cwd =
			env::current_exe().map_err(Error::msg("failed to get current executable path"))?;
		let cwd = cwd
			.parent()
			.expect("failed to get current working directory");

		let buffer = fs::read_to_string(cwd.join(format!("mods/{}/plugin.lua", plugin_name)))
			.map_err(Error::msg("failed to read into buffer for lua file"))?;

		let environment = lua
			.create_table()
			.map_err(runtime::Error::from)
			.map_err(Error::msg("failed to create lua table"))?;

		for global in lua.globals().pairs::<String, Value>() {
			if let Ok((key, value)) = global {
				environment
					.set(key, value)
					.map_err(runtime::Error::from)
					.map_err(Error::msg("failed to set global in lua table"))?;
			}
		}

		let response: Option<Table> = lua
			.load(&buffer)
			.set_environment(environment.clone())
			.eval()
			.map_err(runtime::Error::from)
			.map_err(Error::msg("failed to evaluate buffer as lua source code"))?;

		let response = response
			.ok_or(runtime::Error::NoTableReturned(plugin_name.to_string()))
			.map_err(Error::msg("failed to get lua table"))?;

		let manifest = {
			let table = response
				.get::<Table>("manifest")
				.map_err(runtime::Error::from)
				.map_err(Error::msg("failed to get manifest from plugin"))?;

			parse_manifest(plugin_name, table).map_err(Error::msg("failed to parse manifest"))?
		};

		let exports = response.get("exports").ok();
		let onstart = response.get("onstart").ok();
		let onstop = response.get("onstop").ok();

		environment
			.set("_G", environment.clone())
			.map_err(runtime::Error::from)
			.map_err(Error::msg("failed to set lua environment variable"))?;

		let plugin = Plugin {
			manifest,
			exports,
			onstart,
			onstop,
		};

		REGISTRY.lock().unwrap().register_plugin(id, plugin);

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
					let id = plugin::Id::try_from(name).unwrap();

					if let Some(plugin) = REGISTRY.lock().unwrap().plugins.get(&id).cloned() {
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

pub fn parse_manifest(
	plugin_name: &str,
	table: Table,
) -> std::result::Result<plugin::Manifest, runtime::Error> {
	macro_rules! manifest_key {
		($manifest:ident, $name:ident, $type:ty) => {{
			let Ok($name) = $manifest.get::<$type>(stringify!($name)) else {
				return Err(runtime::Error::MissingManifestKey(
					stringify!($name).to_string(),
					plugin_name.to_string(),
				));
			};
			$name
		}};
	}

	let name = manifest_key!(table, name, String);
	let version = manifest_key!(table, version, String);
	let author = manifest_key!(table, author, String);

	let dependencies = table.get::<Option<Vec<String>>>("dependencies").unwrap();

	let dependencies = dependencies
		.unwrap_or_default()
		.into_iter()
		.map(plugin::Id::try_from)
		.collect::<std::result::Result<Vec<_>, _>>()
		.unwrap();

	let conflicts = table.get::<Option<Vec<String>>>("conflicts").unwrap();

	let conflicts = conflicts
		.unwrap_or_default()
		.into_iter()
		.map(plugin::Id::try_from)
		.collect::<std::result::Result<Vec<_>, _>>()
		.unwrap();

	Ok(plugin::Manifest {
		name,
		version,
		author,
		dependencies,
		conflicts,
	})
}
