pub(crate) struct HookSignature {
	pub offset: isize,
}

pub(crate) struct PatchSignature {
	pub offset: isize,
}

pub(crate) fn get_hook_offset(attr: syn::Attribute) -> Option<isize> {
	let args: syn::Result<HookSignature> = attr.parse_args();
	match args {
		Ok(attr) => Some(attr.offset),
		Err(_) => None,
	}
}

pub(crate) fn get_patch_offset(attr: syn::Attribute) -> Option<isize> {
	let args: syn::Result<PatchSignature> = attr.parse_args();
	match args {
		Ok(attr) => Some(attr.offset),
		Err(_) => None,
	}
}

impl syn::parse::Parse for HookSignature {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let mut offset = 0;
		loop {
			if input.is_empty() {
				break;
			}

			let key: syn::Ident = input.parse()?;
			if input.is_empty() {
				break;
			}

			input.parse::<syn::Token![=]>()?;
			if input.is_empty() {
				break;
			}

			let value: syn::LitInt = input.parse()?;
			match key.to_string().as_str() {
				"offset" => offset = value.base10_parse::<isize>()?,
				name => {
					return Err(syn::Error::new(
						key.span(),
						format!(r#"unknown attribute "{}""#, name),
					))
				}
			}
		}

		Ok(Self { offset })
	}
}

impl syn::parse::Parse for PatchSignature {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let mut offset = 0;
		loop {
			if input.is_empty() {
				break;
			}

			let key: syn::Ident = input.parse()?;
			if input.is_empty() {
				break;
			}

			input.parse::<syn::Token![=]>()?;
			if input.is_empty() {
				break;
			}

			let value: syn::LitInt = input.parse()?;
			match key.to_string().as_str() {
				"offset" => offset = value.base10_parse::<isize>()?,
				name => {
					return Err(syn::Error::new(
						key.span(),
						format!(r#"unknown attribute "{}""#, name),
					))
				}
			}
		}

		Ok(Self { offset })
	}
}
