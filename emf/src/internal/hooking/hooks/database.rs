use super::Hook;

#[allow(non_upper_case_globals)]
pub static mut HookDB: Hooks = Hooks { hooks: Vec::new() };

pub struct Hooks {
	hooks: Vec<Hook>,
}

#[allow(unused)]
impl Hooks {
	pub unsafe fn attach_hook(&mut self, name: &str) -> bool {
		let hook = self.get_hook_mut(name).unwrap();
		hook.get_hook_mut().attach()
	}

	pub unsafe fn detach_hook(&mut self, name: &str) -> bool {
		let hook = self.get_hook_mut(name).unwrap();
		hook.get_hook_mut().detach()
	}

	pub fn add_hook(&mut self, hook: Hook) -> &mut Hook {
		let name = hook.get_name().to_owned();
		self.hooks.push(hook);
		self.get_hook_mut(&name).unwrap()
	}

	pub fn get_hooks(&self) -> &Vec<Hook> {
		&self.hooks
	}

	pub fn get_hooks_mut(&mut self) -> &mut Vec<Hook> {
		&mut self.hooks
	}

	pub fn get_hook(&self, name: &str) -> Option<&Hook> {
		self.hooks.iter().find(|hook| hook.get_name() == name)
	}

	pub fn get_hook_mut(&mut self, name: &str) -> Option<&mut Hook> {
		self.hooks.iter_mut().find(|hook| hook.get_name() == name)
	}

	pub fn get_hook_by_original_ptr(&mut self, original_ptr: usize) -> Option<&Hook> {
		self.get_hooks()
			.iter()
			.find(|hook| hook.get_hook().get_original_ptr() == original_ptr)
	}

	pub fn get_hook_by_original_ptr_mut(&mut self, original_ptr: usize) -> Option<&mut Hook> {
		self.get_hooks_mut()
			.iter_mut()
			.find(|hook| hook.get_hook().get_original_ptr() == original_ptr)
	}
}
