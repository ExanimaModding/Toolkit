pub(crate) struct PluginAttribute {
	pub id: String,
}

impl syn::parse::Parse for PluginAttribute {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let mut id = String::new();
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

			let value: syn::LitStr = input.parse()?;
			match key.to_string().as_str() {
				"id" => id = value.value(),
				name => {
					return Err(syn::Error::new(
						key.span(),
						format!(r#"unknown attribute "{}""#, name),
					))
				}
			}
		}

		match id.as_str() {
			"" => Err(syn::Error::new(input.span(), "id attribute is required")),
			_ => Ok(Self { id }),
		}
	}
}
