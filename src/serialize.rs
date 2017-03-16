use img::Image;
use compare::{SimilarImage, UniqueImage};

use img_hash::ImageHash;

use rustc_serialize::json::{self,  Json};

use std::io;
use std::io::prelude::*;
use std::iter::IntoIterator;
use std::time::Duration;

#[derive(Serialize, Deserialize)]
struct SerializeImage {
	path: PathBuf,
	hash: String,
	dimensions: (u32, u32),
	load_time: u64,
	hash_time: u64,
}

impl SerializeImage {
	fn from_img(img: &Image) -> SerializeImage {
		SerializeImage {
			path: img.path.display().to_string(),
			hash: img.hash.to_base64(),
			dimensions: img.dimensions,
			load_time: duration_millis(img.load_time),
			hash_time: duration_millis(img.hash_time),
		}
	}

	fn to_image(&self) -> Image {
		Image {
			path: From::from(&self.path.clone()),
			hash: ImageHash::from_base64(&self.hash).unwrap(),
			dimensions: self.dimensions,
			load_time: Duration::from_millis(self.load_time),
			hash_time: Duration::from_millis(self.hash_time),
		}
	}
}

#[derive(RustcEncodable, RustcDecodable)]
struct SerializeUnique {
	img: SerializeImage,
	similars: Vec<SerializeSimilar>,
}

impl SerializeUnique {
	fn from_unique(unique: &UniqueImage) -> SerializeUnique {
		SerializeUnique {
			img: SerializeImage::from_img(&unique.img),
			similars: unique.similars.iter()
				.map(SerializeSimilar::from_similar).collect(), 
		}
	}

	fn to_unique(&self) -> UniqueImage {
		UniqueImage {
			img: self.img.to_image(),
			similars: self.similars.iter()
				.map(|similar| similar.to_similar()).collect(),
		}
	}
}

#[derive(RustcEncodable, RustcDecodable)]
struct SerializeSimilar {
	img: SerializeImage,
	dist: u32,
}

impl SerializeSimilar {
	fn from_similar(similar: &SimilarImage) -> SerializeSimilar {
		SerializeSimilar {
			img: SerializeImage::from_img(&similar.img),
			dist: similar.dist,
		}
	}

	fn to_similar(&self) -> SimilarImage {
		SimilarImage {
			img: self.img.to_image(),
			dist: self.dist,
		}
	}
}

#[derive(RustcEncodable, RustcDecodable)]
pub struct SerializeSession {
	pub hash_size: u32,
	images: Vec<SerializeImage>,
}

impl SerializeSession {
	pub fn from_images<'a, I>(
		into_iter: I,
		hash_size: u32
	) -> SerializeSession where I: IntoIterator<Item=&'a Image> {
		SerializeSession {
			hash_size: hash_size,
			images: into_iter.into_iter()
				.map(SerializeImage::from_img).collect(),
		}
	}

	pub fn write_json(&self, wrt: &mut Write, pretty_indent: Option<u32>) -> io::Result<()> {
		match pretty_indent {
			Some(indent) => {
				let pretty = json::as_pretty_json(self).indent(indent);
				write!(wrt, "{}", pretty)
			},
			None => write!(wrt, "{}", json::as_json(self)),
		}
	}

	pub fn read_json(read: &mut Read) -> Result<Self, json::DecoderError> {
		use rustc_serialize::Decodable;
		
		let json = try!(
			Json::from_reader(read)
				.map_err(|err| json::DecoderError::ParseError(err))
		);

		let ref mut decoder = json::Decoder::new(json);
		<SerializeSession as Decodable>::decode(decoder)
	}

	pub fn get_images(&self) -> Vec<Image> {
		self.images.iter().map(|img| img.to_image()).collect()	
	}
}

