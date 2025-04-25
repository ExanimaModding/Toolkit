use crate::{attributes::signature::get_patch_offset, utils::AddressingType};
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
	patch_signature: Option<&syn::Attribute>,
	patch_pointer: Option<&syn::Attribute>,
	link_setting: Option<String>,
	func: &syn::ItemFn,
	name: &syn::Ident,
	args: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
	ret: &syn::ReturnType,
) -> (TokenStream, TokenStream) {
	let definition_token: TokenStream;
	let registration_token: TokenStream;

	let patch_type = match (patch_signature, patch_pointer) {
		(Some(patch_signature), None) => {
			AddressingType::Signature(get_patch_offset(patch_signature.clone()).unwrap_or(0))
		}
		(None, Some(_patch_pointer)) => AddressingType::Pointer,
		(Some(_patch_signature), Some(_patch_pointer)) => {
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
			plugin.link_setting(&format!("patch::{}", stringify!(#name)), #link_setting);
		}
	} else {
		quote! {}
	};

	match parse_register_fn(&mut body) {
		Some(register_macro) => {
			registration_token = match patch_type {
				AddressingType::Pointer => quote! {{
					let address = { #register_macro } as *mut u8;
					let bytes = #name::get_replacement_bytes(address);
					let patch = plugin.create_patch(stringify!(#name), address, bytes);
					match patch {
						Ok(_) => { #link_setting }
						Err(e) => {
							tracing::error!("Failed to create patch for {}: {:?}", stringify!(#name), e);
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
						let address = Memory::sig_scan(signature) as *mut u8;
						#offset
						let bytes = #name::get_replacement_bytes(address).to_owned();
						let patch = plugin.create_patch(stringify!(#name), address, bytes);
						match patch {
							Ok(_) => { #link_setting }
							Err(e) => {
								tracing::error!("Failed to create patch for {}: {:?}", stringify!(#name), e);
							}
						}
					}}
				}
			}
		}
		None => {
			return_syn_err!(
				func.sig,
				"register!() macro is required for a patch function"
			);
		}
	}

	definition_token = quote! {
		mod #name {
			use super::*;

			pub(crate) unsafe fn get_replacement_bytes(#args) #ret {
				#body
			}
		}
	};

	(definition_token, registration_token)
}
