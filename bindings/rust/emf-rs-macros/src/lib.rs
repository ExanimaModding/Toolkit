mod attributes;
mod parsers;
mod utils;

use attributes::plugin::PluginAttribute;
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn plugin(attr: TokenStream, item: TokenStream) -> TokenStream {
	let input = syn::parse_macro_input!(item as syn::ItemMod);
	let plugin_id = syn::parse_macro_input!(attr as PluginAttribute).id;

	let name = &input.ident;
	let body = &input.content.unwrap().1;

	let mut definitions_tokens = Vec::new();
	let mut registration_tokens = Vec::new();

	for item in body {
		let (definition, registration) = utils::parse_module_body(item.clone());
		definitions_tokens.push(definition);
		registration_tokens.push(registration);
	}

	let expanded = quote! {
		pub(crate) mod #name {
			use std::sync::{Arc, Mutex};
			use emf_rs::once_cell::sync::Lazy;
			use emf_rs::macros::{hook_signature, hook_pointer, patch_signature, patch_pointer, link_setting};
			use emf_rs::*;
			use super::*;

			#(#definitions_tokens)*

			static mut PLUGIN: Lazy<Arc<Mutex<Plugin>>> = Lazy::new(|| unsafe {
				let mut plugin = Plugin::new(#plugin_id);

				#(#registration_tokens)*

				Arc::new(Mutex::new(plugin))
			});

			pub(crate) unsafe fn get() -> std::sync::MutexGuard<'static, Plugin> {
				PLUGIN.lock().unwrap()
			}
		}
	};

	TokenStream::from(expanded)
}

/// A function hook that takes a pointer in the register macro.
///
/// # Examples
///
/// ```
/// #[plugin(id = "dev.megu.god-mode")]
/// mod plugin {
///     #[hook_pointer]
///     extern "C" fn hooked_fn(arg1: *mut c_void, _: f32) -> c_char {
///         // the register!() macro returns a pointer to the target function
///         register!(
///             let sig = "DE AD BE EF ?? ?? ?? ??";
///             Memory::sig_scan(sig)
///         );
///
///         // this will print every time the original function is called
///         println!("Arg 1: {:p}", arg1);
///
///         // calling hooked_fn from inside the hook runs the original function
///         hooked_fn(arg1, 0.0)
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn hook_pointer(_attr: TokenStream, item: TokenStream) -> TokenStream {
	item
}

/// A function hook that takes a signature in the register macro.
///
/// Optionally, you can provide an `offset` argument to specify the pointer offset from the signature.
///
/// # Examples
///
/// ```
/// #[plugin(id = "dev.megu.god-mode")]
/// mod plugin {
///     // without an offset...
///     #[hook_signature]
///     // or optionally with an offset...
///     #[hook_signature(offset = 0x20)]
///     extern "C" fn hooked_fn(arg1: *mut c_void, _: f32) -> c_char {
///         // the register!() macro returns a signature for the target function
///         register!(
///             "DE AD BE EF ?? ?? ?? ??"
///         );
///
///         // this will print every time the original function is called
///         println!("Arg 1: {:p}", arg1);
///
///         // calling hooked_fn from inside the hook runs the original function
///         hooked_fn(arg1, 0.0)
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn hook_signature(_attr: TokenStream, item: TokenStream) -> TokenStream {
	item
}

/// A byte patch that takes a pointer in the register macro.
///
/// # Examples
///
/// ```
/// #[plugin(id = "dev.megu.god-mode")]
/// mod plugin {
///     #[patch_pointer]
///     fn ignore_range_limit_for_placement(_address: *mut u8) -> Vec<u8> {
///         register!("DE AD BE EF ?? ?? ?? ??");
///         vec![0x90]
///     }
/// }
/// ```
///
#[proc_macro_attribute]
pub fn patch_pointer(_attr: TokenStream, item: TokenStream) -> TokenStream {
	item
}

/// A byte patch that takes a signature in the register macro.
///
/// Optionally, you can provide an `offset` argument to specify the pointer offset from the signature.
///
/// # Examples
///
/// ```
/// #[plugin(id = "dev.megu.god-mode")]
/// mod plugin {
///     // without an offset...
///     #[patch_signature]
///     // or optionally with an offset...
///     #[patch_signature(offset = 0x20)]
///     fn ignore_range_limit_for_placement(_address: *mut u8) -> Vec<u8> {
///         register!("DE AD BE EF ?? ?? ?? ??");
///         vec![0x90]
///     }
/// }
#[proc_macro_attribute]
pub fn patch_signature(_attr: TokenStream, item: TokenStream) -> TokenStream {
	item
}

/// Link this setting to another setting.
///
/// # Examples
///
/// ```
/// #[patch_signature]
/// #[link_setting("godmode_enabled")]
/// extern "C" fn patch_thing() {
///     register!("DE AD BE EF ?? ?? ?? ??");
///     vec![0x90]
/// }
/// ```
///
/// In this example, if the setting `godmode_enabled` is edited, it will also edit the state for `patch::patch_thing`.
#[proc_macro_attribute]
pub fn link_setting(_attr: TokenStream, item: TokenStream) -> TokenStream {
	item
}
