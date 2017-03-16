macro_rules! hash_types {
    ($($name:ident, $dispnm:expr, $serializenm:expr, $desc:expr);+;) => {
        #[derive(Copy, Clone, Debug, Serialize, Deserialize)]
        pub enum HashType {
            $(
                #[serde(rename = $serializenm)]
                $name
            ),+
        }

        pub const HASH_TYPES: &'static [HashType] = &[$(HashType::$name),+];

        impl HashType {
            fn display_name(&self) -> &'static str {
                match *self {
                    $(HashType::$name => $dispnm),+
                }
            }

            fn serialize_name(&self) -> &'static str {
                match *self {
                    $(HashType::$name => $serializenm),+
                }
            }

            fn description(&self) -> &'static str {
                match *self {
                    $(HashType::$name => $desc),+
                }
            }

            fn to_img_hash(&self) -> ::img_hash::HashType {
                match *self {
                    $(HashType::$name => ::img_hash::HashType::$name),+
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
    Mean, "Mean", "mean", "Averages the pixels of the reduced-size and \
                           color image, and then compares each pixel to the average. \n\
                           Fast, but inaccurate. Really only useful for finding duplicates.";

    Block, "Blockhash.io", "block", "The Blockhash.io (http://blockhash.io) algorithm. \n\
                            Faster than `Mean` but also prone to more collisions and suitable \
                            only for finding duplicates.";

    Gradient, "Gradient", "grad", "This algorithm compares each pixel in a row to its neighbor \
                                   and registers changes in gradients (e.g. edges and color \
                                   boundaries). \n\n\
                                   More accurate than `Mean` but much faster than `DCT`.";
}

pub fn print_all() {
    println!("`img-dup` currently supports the following hash types:");
    for hash_type in HASH_TYPES {
        println!("{} (--hash-type={}): {}", hash_type.display_name(),
                 hash_type.serialize_name(), hash_type.description());
    }
}

pub fn validate_hash_type(hash_type: String) -> Result<(), String> {
    hash_type.parse::<HashType>().and(Ok(()))
}