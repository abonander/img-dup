use std::fmt::{self, Display, Formatter, Write};
use std::time::Duration;

pub struct Number<N>(pub N);

macro_rules! number_impl {
    ($basety:ty; $($ty:ty),+) => (
        $(impl Display for Number<$ty> {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                Number(self.0 as $basety).fmt(f)
            }
        })+
    )
}

number_impl! { u64; u8, u16, u32, usize }

impl Display for Number<u64> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut num = self.0;

        if num < 1000 {
            return write!(f, "{}", num);
        }

        let mut digits = [0u16; 8];

        let mut idx = 0;

        while {
            digits[idx] = (num % 1000) as u16;
            num >= 1000
        } {
            num /= 1000;
            idx += 1;
        }

        write!(f, "{},", digits[idx])?;
        idx -= 1;

        while {
            write!(f, "{:03}", digits[idx])?;
            idx > 0
        } {
            f.write_char(',')?;
            idx -= 1;
        }

        Ok(())
    }
}

pub struct Time(pub Duration);

impl Display for Time {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let total_secs = self.0.as_secs();

        let total_mins = total_secs / 60;
        let secs = total_secs % 60;

        let hrs = total_mins / 60;
        let mins = total_mins % 60;

        if hrs == 0 {
            write!(f, "{}:{:02}", mins, secs)
        } else {
            write!(f, "{}:{:02}:{:02}", hrs, mins, secs)
        }
    }
}

pub struct Bytes(pub u64);

impl Display for Bytes {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Upgrade to the next unit at 10x the conversion rate so we don't get
        // single-digit numbers
        if self.0 >= 1_000_000_000 {
            write!(f, "{}.{:02} GB", Number(self.0 / 1_000_000_000), self.0 / 10_000_000 % 100)
        } else if self.0 >= 1_000_000 {
            write!(f, "{} MB", Number(self.0 / 1_000_000))
        } else if self.0 >= 10_000 {
            write!(f, "{} KB", Number(self.0 / 1_000))
        } else {
            write!(f, "{} B", Number(self.0))
        }
    }
}

pub struct ByteRate(pub u64);

impl Display for ByteRate {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}/s", Bytes(self.0))

    }
}
