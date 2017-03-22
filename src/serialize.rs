use std::path::Path;

pub struct SerializeSettings<'a> {
	pub outfile: &'a Path,
	pub pretty_indent: Option<usize>,
}

impl Default for SerializeSettings<'static> {
	fn default() -> Self {
		SerializeSettings {
			outfile: "img-dup.json".as_ref(),
			pretty_indent: None,
		}
	}
}


