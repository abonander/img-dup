use std::fmt::{Formatter, Show};
use std::fmt::Error as FmtError;

use std::io::IoResult;

#[deriving(Copy)]
pub struct FormatBytes(u64);

impl FormatBytes {
    #[inline]
    fn write_size(self, w: &mut Writer) -> IoResult<()> {
         match self.0 {
            0 .. 999 => write!(w, "{} B", self.0),
            1_000 .. 999_999 => write!(w, "{.02f} KB", self.to_kb()),
            1_000_000 .. 999_999_999 => write!(w, "{.02f} MB", self.to_mb()),
            _ => write!(w, "{.02f} GB", self.to_gb()),
        }
    }

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
        self.write_size(fmt) 
    }        
}
