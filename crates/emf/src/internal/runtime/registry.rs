use std::{
	collections::HashMap,
	env, fs,
	sync::{Arc, LazyLock, Mutex, OnceLock},
};

use emcore::{
	Error, Result,
	plugin::{self, Manifest},
};
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

		let buffer = fs::read_to_string(cwd.join(format!("mods/{plugin_name}/{}", plugin::LUA)))
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

		let maybe_lua_table: Option<Table> = lua
			.load(&buffer)
			.set_environment(environment.clone())
			.eval()
			.map_err(runtime::Error::from)
			.map_err(Error::msg(format!(
				"failed to evaluate {} as lua source code",
				plugin::LUA
			)))?;

		let lua_table = maybe_lua_table
			.ok_or(runtime::Error::NoTableReturned(plugin_name.to_string()))
			.map_err(Error::msg("failed to get lua table"))?;

		let manifest = {
			let manifest_table = lua_table
				.get::<Table>("manifest")
				.map_err(runtime::Error::from)
				.map_err(Error::msg("failed to get manifest from plugin"))?;

			parse_manifest(plugin_name, manifest_table)?
		};

		let exports = lua_table.get("exports").ok();
		let onstart = lua_table.get("onstart").ok();
		let onstop = lua_table.get("onstop").ok();

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

pub fn parse_manifest(plugin_name: &str, table: Table) -> Result<plugin::Manifest> {
	let name = table
		.get(Manifest::NAME)
		.map_err(runtime::Error::from)
		.map_err(Error::msg(format!(
			"failed to get {} from {}'s manifest",
			Manifest::NAME,
			plugin_name
		)))?;
	let version = table
		.get(Manifest::VERSION)
		.map_err(runtime::Error::from)
		.map_err(Error::msg(format!(
			"failed to get {} from {}'s manifest",
			Manifest::VERSION,
			plugin_name
		)))?;
	let author = table
		.get(Manifest::AUTHOR)
		.map_err(runtime::Error::from)
		.map_err(Error::msg(format!(
			"failed to get {} from {}'s manifest",
			Manifest::AUTHOR,
			plugin_name
		)))?;

	let dependencies = table
		.get::<Option<Vec<String>>>(Manifest::DEPENDENCIES)
		.map_err(runtime::Error::from)
		.map_err(Error::msg(format!(
			"failed to get {}'s dependencies",
			plugin_name
		)))?;

	let dependencies = dependencies
		.unwrap_or_default()
		.into_iter()
		.map(plugin::Id::try_from)
		.collect::<std::result::Result<Vec<_>, _>>()
		.map_err(Error::msg(format!(
			"failed to collect {}'s dependencies",
			plugin_name
		)))?;

	let conflicts = table
		.get::<Option<Vec<String>>>(Manifest::CONFLICTS)
		.map_err(runtime::Error::from)
		.map_err(Error::msg(format!(
			"failed to get {}'s conflicts",
			plugin_name
		)))?;

	let conflicts = conflicts
		.unwrap_or_default()
		.into_iter()
		.map(plugin::Id::try_from)
		.collect::<std::result::Result<Vec<_>, _>>()
		.map_err(Error::msg(format!(
			"failed to collect {}'s conflicts",
			plugin_name
		)))?;

	Ok(plugin::Manifest {
		name,
		version,
		author,
		dependencies,
		conflicts,
	})
}
