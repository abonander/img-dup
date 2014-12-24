use std::fmt::{Formatter, Show};
use std::fmt::Error as FmtError;

use std::io::IoResult;

#[deriving(Copy)]
pub struct FormatBytes(u64);

impl FormatBytes { 
    #[inline]
    fn to_kb(self) -> f64 {
        (self.0 as f64) / 1.0e3   
    }

    #[inline]
    fn to_mb(self) -> f64 {
        (self.0 as f64) / 1.0e6
    }

    #[inline]
    fn to_gb(self) -> f64 {
        (self.0 as f64) / 1.0e9
    }
}


impl Show for FormatBytes {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        match self.0 {
            0 ... 999 => format_args!(|args| fmt.write_fmt(args), "{} B", self.0),
            1_000 ... 999_999 => format_args!(|args| fmt.write_fmt(args), "{:.02} KB", self.to_kb()),
            1_000_000 ... 999_999_999 => format_args!(|args| fmt.write_fmt(args), "{:.02} MB", self.to_mb()),
            _ => format_args!(|args| fmt.write_fmt(args), "{:.02} GB", self.to_gb()),
        }
    }        
}
