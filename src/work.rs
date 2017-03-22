use image::{self, DynamicImage, GenericImage};

use img_hash::ImageHash;

use std::fs::File;
use std::path::PathBuf;
use std::io::{self, BufReader, BufRead, Read, Seek, SeekFrom};
use std::time::{Instant, Duration};
use std::mem;

use model::{Image, HashedImage};
use hash::HashSettings;

pub struct LoadedImage {
    data: DynamicImage,
    pub image: Image,
}

impl LoadedImage {
    pub fn hash(self, settings: &HashSettings) -> HashedImage {
        let (hash, hash_time) = time_span_ms(|| settings.hash(&self.data));

        HashedImage {
            image: self.image,
            hash: hash,
            hash_time: hash_time,
        }
    }
}

fn time_span_ms<T, F: FnOnce() -> T>(f: F) -> (T, u64) {
    let start = Instant::now();
    let val = f();
    (val, duration_millis(start.elapsed()))
}

fn duration_millis(duration: Duration) -> u64 {
    let ms_secs = duration.as_secs() * 1000;
    // 1 ms == 1M ns
    let ms_nanos = duration.subsec_nanos() as u64 / 1_000_000;

    ms_secs + ms_nanos
}

pub fn load(path: PathBuf) -> Result<LoadedImage, ::Error> {
    let (res, load_time) = time_span_ms(|| {
        let mut reader = BufReader::new(File::open(&path)?);
        let fmt = image::guess_format(reader.fill_buf()?)?;
        image::load(reader, fmt)
    });

    match res {
        Ok(data) => {
            let image = Image {
                path: path,
                dimensions: data.dimensions(),
                loaded_size: loaded_size_dyn(&data),
                load_time: load_time,
            };

            Ok(LoadedImage {
                data: data,
                image: image,
            })
        },
        Err(e) => Err(::Error {
            path: path,
            error: e.into(),
        })
    }
}

fn loaded_size_dyn(image: &DynamicImage) -> u64 {
    use self::DynamicImage::*;

    match *image {
        ImageLuma8(ref img) => loaded_size(&img),
        ImageLumaA8(ref img) => loaded_size(&img),
        ImageRgb8(ref img) => loaded_size(&img),
        ImageRgba8(ref img) => loaded_size(&img),
    }
}

fn loaded_size<SubPx>(subpx: &[SubPx]) -> u64 {
    subpx.len() as u64 * (mem::size_of::<SubPx>() as u64)
}
