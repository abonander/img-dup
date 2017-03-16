use img_hash::HashType as TrueHashType;

macro_rules! hash_types {
    ($($name:ident, $dispnm:expr, $serializenm:expr, $desc:expr);+;) => {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
        pub enum HashType {
            $(
                #[serde(rename = $serializenm)]
                $name
            ),+
        }

        pub const HASH_TYPES: &'static [HashType] = &[$(HashType::$name),+];

        impl HashType {
            pub fn display_name(&self) -> &'static str {
                match *self {
                    $(HashType::$name => $dispnm),+
                }
            }

            pub fn serialize_name(&self) -> &'static str {
                match *self {
                    $(HashType::$name => $serializenm),+
                }
            }

            pub fn description(&self) -> &'static str {
                match *self {
                    $(HashType::$name => $desc),+
                }
            }
        }

        impl Into<TrueHashType> for HashType {
            fn into(self) -> TrueHashType {
                match self {
                    $(HashType::$name => TrueHashType::$name),+
                }
            }
        }

        impl From<TrueHashType> for HashType {
            fn into(hash_type: TrueHashType) -> HashType {
                match hash_type {
                    $(TrueHashType::$name => HashType::$name),+,
                    _ => panic!("Unsupported hash type: {:?}", hash_type),
                }
            }
        }

        impl ::std::str::FromStr for HashType {
            type Err = String;

            fn from_str(str: &str) -> Result<Self, String> {
                match str {
                    $($serializenm => Ok(HashType::$name),)+
                    _ => Err(format!("{:?} is not a valid hash type; run \
                                      `img-dup --list-hash-types` to list the currently supported \
                                      hash types", str))
                }
            }
        }


    }
}

hash_types! {
    Mean, "Mean", "mean",
    "Averages the pixels of the reduced-size and color image, and then compares each pixel to \
     the average.\nFast, but inaccurate. Really only useful for finding duplicates.";

    Block, "Blockhash.io", "block",
    "The Blockhash.io (http://blockhash.io) algorithm.\n\
     Faster than `Mean` but also prone to more collisions and suitable only for \
     finding duplicates. Does not require resizing or filtering the image (reports a hash \
     size of 0).";

    Gradient, "Gradient", "grad",
    "Compares each pixel in a row to its neighbor and registers changes in gradients (e.g. edges \
     and color boundaries).\nSlower and more accurate than `Mean` but much faster than `DCT`.";

    DoubleGradient, "Double-Gradient", "dblgrad",
    "A version of `Gradient` that adds an extra hash pass orthogonal to the first (i.e. on \
     columns in addition to rows).\nSlower than `Gradient` and produces a double-sized hash, \
     but much more accurate.";

    DCT, "DCT", "dct",
    "Runs a Discrete Cosine Transform on the reduced-color and size image, \
     then compares each datapoint in the transform to the average.\n\
     Slowest by far, but can detect changes in color gamut and sometimes relatively \
     significant edits.";
}

impl Default for HashType {
    fn default() -> Self {
        HashType::Gradient
    }
}

impl HashType {
    pub fn hash_size(&self, hash_size: u32) -> u32 {
        match *self {
            HashSize::Block => 0,
            HashSize::DoubleGradient => hash_size * 2,
            HashSize::DCT => hash_size * 4,
        }
    }
}

pub fn print_all() {
    println!("`img-dup` currently supports the following hash types:");
    for hash_type in HASH_TYPES {
        println!("{} --hash-type={} \n{}\n", hash_type.display_name(),
                 hash_type.serialize_name(), hash_type.description());
    }
}

pub fn validate(hash_type: String) -> Result<(), String> {
    hash_type.parse::<HashType>().and(Ok(()))
}