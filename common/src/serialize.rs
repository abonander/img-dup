use img::Image;
use compare::{SimilarImage, UniqueImage};

use rustc_serialize::json::{self, Json, ToJson};

use std::path::PathBuf;

#[derive(RustcEncodable, RustcDecodable)]
pub struct SerializeImage {
	path: PathBuf,
	hash: String,
	dimensions: (u32, u32),
	load_time: i64,
	hash_time: i64,
}

impl SerializeImage {
	pub fn from_img(img: &Image) -> SerializeImage {
		SerializeImage {
			path: img.path.clone(),
			hash: img.hash.to_base64(),
			dimensions: img.dimensions,
			load_time: img.load_time.num_milliseconds(),
			hash_time: img.hash_time.num_milliseconds(),
		}
	}
}

#[derive(RustcEncodable, RustcDecodable)]
pub struct SerializeUnique {
	img: SerializeImage,
	similars: Vec<SerializeSimilar>,
}

impl SerializeUnique {
	pub fn from_unique(unique: &UniqueImage) -> SerializeUnique {
		SerializeUnique {
			img: SerializeImage::from_img(&unique.img),
			similars: unique.similars.iter()
				.map(SerializeSimilar::from_similar).collect(), 
		}
	}
}

#[derive(RustcEncodable, RustcDecodable)]
pub struct SerializeSimilar {
	img: SerializeImage,
	dist: u32,
}

impl SerializeSimilar {
	pub fn from_similar(similar: &SimilarImage) -> SerializeSimilar {
		SerializeSimilar {
			img: SerializeImage::from_img(&similar.img),
			dist: similar.dist,
		}
	}
}

pub fn write_json<I: IntoIterator<Item==&UniqueImage>>(into_iter: I) -> Vec<SerializeUnique> {
	into_iter.into_iter().map(SerializeUnique::from_unique).collect()	
}


