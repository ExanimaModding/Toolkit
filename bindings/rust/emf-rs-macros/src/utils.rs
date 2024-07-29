use crate::{
	attributes::link_setting::parse_link_setting,
	parsers::{hook, patch},
};
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) enum AddressingType {
	Pointer,
	Signature(isize),
}

macro_rules! return_syn_err {
	($i:expr, $msg:expr) => {
		return (
			syn::Error::new($i.ident.span(), $msg).to_compile_error(),
			quote::quote! {},
		);
	};
}

pub(crate) fn parse_module_body(item: syn::Item) -> (TokenStream, TokenStream) {
	let definition_token: TokenStream;
	let registration_token: TokenStream;

	if let syn::Item::Fn(func) = item {
		let name = &func.sig.ident;
		let args = &func.sig.inputs;
		let ret = &func.sig.output;

		let attributes: Vec<(&str, &syn::Attribute)> = func
			.attrs
			.iter()
			.filter_map(|attr| {
				let path = attr.path();
				if path.is_ident("hook_signature") {
					Some(("hook_signature", attr))
				} else if path.is_ident("hook_pointer") {
					Some(("hook_pointer", attr))
				} else if path.is_ident("patch_signature") {
					Some(("patch_signature", attr))
				} else if path.is_ident("patch_pointer") {
					Some(("patch_pointer", attr))
				} else if path.is_ident("link_setting") {
					Some(("link_setting", attr))
				} else {
					None
				}
			})
			.collect();

		let link_setting = attributes.iter().find_map(|(name, attr)| match *name {
			"link_setting" => Some(*attr),
			_ => None,
		});

		let hook_signature = attributes.iter().find_map(|(name, attr)| match *name {
			"hook_signature" => Some(*attr),
			_ => None,
		});

		let hook_pointer = attributes.iter().find_map(|(name, attr)| match *name {
			"hook_pointer" => Some(*attr),
			_ => None,
		});

		let patch_signature = attributes.iter().find_map(|(name, attr)| match *name {
			"patch_signature" => Some(*attr),
			_ => None,
		});

		let patch_pointer = attributes.iter().find_map(|(name, attr)| match *name {
			"patch_pointer" => Some(*attr),
			_ => None,
		});

		let link_setting = match link_setting {
			Some(link_setting) => parse_link_setting(link_setting.clone()),
			None => None,
		};

		let is_hook = hook_signature.is_some() || hook_pointer.is_some();
		let is_patch = patch_signature.is_some() || patch_pointer.is_some();

		if is_hook {
			(definition_token, registration_token) = hook::parse(
				hook_signature,
				hook_pointer,
				link_setting,
				&func,
				name,
				args,
				ret,
			);
		} else if is_patch {
			(definition_token, registration_token) = patch::parse(
				patch_signature,
				patch_pointer,
				link_setting,
				&func,
				name,
				args,
				ret,
			);
		} else {
			return_syn_err!(func.sig, "One of #[hook_signature], #[hook_pointer], #[patch_signature], or #[patch_pointer] must be provided");
		}
	} else {
		definition_token = quote! {};
		registration_token = quote! {};
	}

	(definition_token, registration_token)
}
