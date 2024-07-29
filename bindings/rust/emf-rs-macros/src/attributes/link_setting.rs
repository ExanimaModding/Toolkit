pub(crate) struct LinkSetting(Option<String>);

pub(crate) fn parse_link_setting(link_setting: syn::Attribute) -> Option<String> {
	let args: syn::Result<LinkSetting> = link_setting.parse_args();
	match args {
		Ok(attr) => attr.0,
		Err(_) => None,
	}
}

impl syn::parse::Parse for LinkSetting {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let value: syn::LitStr = input.parse()?;
		Ok(Self(Some(value.value())))
	}
}
