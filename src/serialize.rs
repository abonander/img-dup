use std::path::Path;

pub struct SerializeSettings<'a> {
	pub outfile: &'a Path,
	pub pretty_indent: Option<usize>,
}
