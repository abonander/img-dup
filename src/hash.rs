use dct::{dct_2d, crop_dct};

use image::{GenericImage, DynamicImage,
    ImageBuf, Luma, Pixel, FilterType, Nearest, Rgba};
use image::imageops::{grayscale, resize};

use serialize::base64::{ToBase64, STANDARD};

use std::collections::Bitv;

const FILTER_TYPE: FilterType = Nearest;

#[derive(PartialEq, Eq, Hash, Show, Clone)]
pub struct ImageHash {
    size: u32,
    bitv: Bitv,
}

impl ImageHash {

    pub fn dist(&self, other: &ImageHash) -> usize {
        assert!(self.bitv.len() == other.bitv.len(),
                "ImageHashes must be the same length for proper comparison!");

        self.bitv.iter().zip(other.bitv.iter())
            .filter(|&(left, right)| left != right).count()
    }

    pub fn dist_ratio(&self, other: &ImageHash) -> f32 {
        self.dist(other) as f32 / self.size as f32
    }


    fn fast_hash<Img: GenericImage<Rgba<u8>>>(img: &Img, hash_size: u32) -> Bitv {
        let temp = square_resize_and_gray(img, hash_size);

        let hash_values: Vec<u8> = temp.pixels().map(|(_, _, x)| x.channel())
            .collect();

        let hash_sq = (hash_size * hash_size) as usize;

        let mean = hash_values.iter().fold(0u, |b, &a| a as usize + b)
            / hash_sq;

        hash_values.into_iter().map(|x| x as usize >= mean).collect()
    }

    fn dct_hash<Img: GenericImage<Rgba<u8>>>(img: &Img, hash_size: u32) -> Bitv {
        let large_size = hash_size * 4;

        // We take a bigger resize than fast_hash,
        // then we only take the lowest corner of the DCT
        let temp = square_resize_and_gray(img, large_size);

        // Our hash values are converted to doubles for the DCT
        let hash_values: Vec<f64> = temp.pixels()
            .map(|(_, _, x)| x.channel() as f64).collect();

        let dct = dct_2d(hash_values.as_slice(),
            large_size as usize, large_size as usize);

        let original = (large_size as usize, large_size as usize);
        let new = (hash_size as usize, hash_size as usize);

        let cropped_dct = crop_dct(dct, original, new);

        let mean = cropped_dct.iter().fold(0f64, |b, &a| a + b)
            / (hash_size * hash_size) as f64;

        cropped_dct.into_iter().map(|x| x >= mean).collect()
    }

    pub fn hash<Img: GenericImage<Rgba<u8>>>(img: &Img, hash_size: u32, fast: bool) -> ImageHash {
        let hash = if fast {
            ImageHash::fast_hash(img, hash_size)
        } else {
            ImageHash::dct_hash(img, hash_size)
        };

        assert!((hash_size * hash_size) as usize == hash.len());

        ImageHash {
            size: hash_size * hash_size,
            bitv: hash,
        }
    }

    pub fn to_base64(&self) -> String {
        let self_bytes = self.bitv.to_bytes();

        self_bytes.as_slice().to_base64(STANDARD)
    }
}

fn square_resize_and_gray<Img: GenericImage<Rgba<u8>>>(img: &Img, size: u32) -> ImageBuf<Luma<u8>> {
        let small = resize(img, size, size, FILTER_TYPE);
        grayscale(&small)
}

