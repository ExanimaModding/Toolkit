pub(crate) mod hook;
pub(crate) mod patch;

use quote::quote;

pub(crate) fn parse_register_fn(body: &mut syn::Block) -> Option<proc_macro2::TokenStream> {
	let mut found = None;
	for stmt in body.stmts.iter_mut() {
		if let syn::Stmt::Macro(macro_item) = stmt {
			if macro_item.mac.path.is_ident("register") {
				let register_macro = Some(macro_item.mac.tokens.clone());

				*stmt = syn::Stmt::Item(syn::Item::Verbatim(quote! {}));

				found = register_macro;
			}
		}
	}

	found
}
