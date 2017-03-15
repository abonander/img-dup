macro_rules! hash_type {
    $($($name:ident, $dispnm:expr, $serializenm:expr, $desc:expr);+) => {
        #[derive(Copy, Clone, Debug)]
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
                    $($HashType::$name => ::img_hash::HashType::$name),+
                }
            }
        }

        impl std::str::FromStr for HashType {
            type Error = ();

            fn from_str(str: &str) -> Result<Self, ()> {
                match str {
                    $($serializenm => Ok(HashType::$name)),+
                    _ => Err(())
                }
            }
        }
    }
}