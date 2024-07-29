use crate::{attributes::signature::get_hook_offset, utils::AddressingType};
use proc_macro2::TokenStream;
use quote::quote;

use super::parse_register_fn;

macro_rules! return_syn_err {
	($i:expr, $msg:expr) => {
		return (
			syn::Error::new($i.ident.span(), $msg).to_compile_error(),
			quote::quote! {},
		);
	};
}

pub(crate) fn parse(
	hook_signature: Option<&syn::Attribute>,
	hook_pointer: Option<&syn::Attribute>,
	link_setting: Option<String>,
	func: &syn::ItemFn,
	name: &syn::Ident,
	args: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
	ret: &syn::ReturnType,
) -> (TokenStream, TokenStream) {
	let definition_token: TokenStream;
	let registration_token: TokenStream;

	let hook_type = match (hook_signature, hook_pointer) {
		(Some(hook_signature), None) => {
			AddressingType::Signature(get_hook_offset(hook_signature.clone()).unwrap_or(0))
		}
		(None, Some(_hook_pointer)) => AddressingType::Pointer,
		(Some(_hook_signature), Some(_hook_pointer)) => {
			return_syn_err!(
				func.sig,
				"Only one of #[patch_signature], #[patch_pointer], #[hook_signature], #[hook_pointer] must be provided"
			);
		}
		(None, None) => {
			return_syn_err!(
				func.sig,
				"One of #[patch_signature], #[patch_pointer], #[hook_signature], #[hook_pointer] must be provided"
			);
		}
	};

	let mut body = func.block.clone();

	let link_setting = if let Some(link_setting) = link_setting {
		quote! {
			plugin.link_setting(&format!("hook::{}", stringify!(#name)), #link_setting);
		}
	} else {
		quote! {}
	};

	// read the register!() macro from body
	match parse_register_fn(&mut body) {
		Some(register_macro) => {
			registration_token = match hook_type {
				AddressingType::Pointer => quote! {{
					let address = { #register_macro };
					#name::set_ptr(address);
					let target_ptr = std::ptr::addr_of_mut!(#name::TARGET_FN);
					let hook = plugin.create_hook(stringify!(#name), target_ptr, #name::func as _);
					match hook {
						Ok(_) => { #link_setting }
						Err(e) => {
							log::error!("Failed to create hook for {}: {:?}", stringify!(#name), e);
						}
					}
				}},
				AddressingType::Signature(offset) => {
					let offset = if offset == 0 {
						quote! {}
					} else {
						quote! {
							let address = address.byte_offset(#offset);
						}
					};
					quote! {{
						let signature = { #register_macro };
						let address = Memory::sig_scan(signature) as *mut std::ffi::c_void;
						#offset
						#name::set_ptr(address);
						let target_ptr = std::ptr::addr_of_mut!(#name::TARGET_FN);
						let hook = plugin.create_hook(stringify!(#name), target_ptr, #name::func as _);
						match hook {
							Ok(_) => { #link_setting }
							Err(e) => {
								log::error!("Failed to create hook for {}: {:?}", stringify!(#name), e);
							}
						}
					}}
				}
			}
		}
		None => {
			return_syn_err!(
				func.sig,
				"register!() macro is required for a hook function"
			);
		}
	}

	definition_token = quote! {
		pub(crate) mod #name {
			use super::*;

			pub(crate) type __FnSig = extern "C" fn(#args) #ret;
			pub(crate) static mut TARGET_FN: *mut std::ffi::c_void = 0 as *mut std::ffi::c_void;

			pub(crate) unsafe extern "C" fn func(#args) #ret {
				let #name = std::mem::transmute::<*mut std::ffi::c_void, __FnSig>(TARGET_FN);
				#body
			}

			pub(crate) unsafe fn set_ptr(new_ptr: *mut std::ffi::c_void) {
				TARGET_FN = new_ptr;
			}
		}
	};

	(definition_token, registration_token)
}
