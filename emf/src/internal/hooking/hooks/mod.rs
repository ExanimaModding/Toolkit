use self::{detour::DetourHook, lua::LuaHook, method::MethodHook};

pub mod database;
pub mod detour;
pub mod lua;
pub mod method;

pub trait HookImpl {
	fn get_type(&self) -> HookType;
	fn get_original_ptr(&self) -> usize;
	fn get_detour_ptr(&self) -> usize;
	unsafe fn attach(&mut self) -> bool;
	unsafe fn detach(&mut self) -> bool;
}

pub enum HookType {
	Detour = 0,
	Method,
	Lua,
}

pub struct Hook {
	name: String,
	hook: Box<dyn HookImpl>,
}

pub trait NewHook<T> {
	fn new(name: String, hook: T) -> Self;
}

impl NewHook<DetourHook> for Hook {
	fn new(name: String, hook: DetourHook) -> Self {
		Self {
			name,
			hook: Box::new(hook),
		}
	}
}

impl NewHook<MethodHook> for Hook {
	fn new(name: String, hook: MethodHook) -> Self {
		Self {
			name,
			hook: Box::new(hook),
		}
	}
}

impl NewHook<LuaHook> for Hook {
	fn new(name: String, hook: LuaHook) -> Self {
		Self {
			name,
			hook: Box::new(hook),
		}
	}
}

impl Hook {
	/// TODO: Fix this warning.
	/// you seem to be trying to use `&Box<T>`. Consider using just `&T`
	#[allow(clippy::borrowed_box)]
	pub fn get_hook(&self) -> &Box<dyn HookImpl> {
		&self.hook
	}

	pub fn get_hook_mut(&mut self) -> &mut Box<dyn HookImpl> {
		&mut self.hook
	}

	pub fn get_name(&self) -> &str {
		&self.name
	}

	/// Transmute the original function pointer to a function pointer of any type.
	///
	/// # Example
	/// ```rs
	/// // Get the hook from storage.
	/// let hook = FunctionHooks::get_hook(HookName::internal("DllMain", "MainHook")).unwrap();
	/// // Transmute the original function pointer to a function pointer of type extern "C" fn().
	/// let main = hook.transmute::<extern "C" fn()>();
	/// // Call the original function.
	/// main();
	/// ```
	pub unsafe fn transmute<Dst>(&mut self) -> Dst {
		std::mem::transmute_copy(&self.get_hook().get_original_ptr())
	}
}
