use const_env::from_env;
use num::Float;
use std::fmt::Display;

#[from_env("CARGO_PKG_VERSION_MINOR")]
pub const ENGINE_VERSION: u16 = 0xFFFF;
pub const ENGINE_INFO: &'static str = concat!(
    "Audiobench is free and open source software. You are free to do anything you want with it, ",
    "including selling any audio, patches, or modules you make with or for it. If you make ",
    "modifications to the source code you must make those changes freely available under the GNU ",
    "General Public License, Version 3. Source code is available at ",
    "https://gitlab.com/Code_Cube/audio-bench."
);
#[cfg(debug_assertions)]
pub const ENGINE_UPDATE_URL: &'static str = "http://localhost:8000/latest.json";
#[cfg(not(debug_assertions))]
pub const ENGINE_UPDATE_URL: &'static str = "https://bit.ly/adb_update_check";

pub trait FloatUtil: Sized + Copy {
    /// Converts from the range [from_min,from_max] to [0,1]
    fn from_range(self, from_min: Self, from_max: Self) -> Self;
    /// Converts from the range [0,1] to [to_min, to_max]
    fn to_range(self, to_min: Self, to_max: Self) -> Self;
    fn from_range_to_range(
        self,
        from_min: Self,
        from_max: Self,
        to_min: Self,
        to_max: Self,
    ) -> Self {
        self.from_range(from_min, from_max).to_range(to_min, to_max)
    }
    /// Clamps the value to the specified range. Not called clamp because that causes an error.
    fn clam(self, min: Self, max: Self) -> Self;
    /// Snaps the value to equally-spaced steps within the given range.
    fn snap(self, min: Self, max: Self, num_steps: Self) -> Self;
    /// Returns a string which represents this value using metric units (e.g. k, m, M, G). sig_figs
    /// is how many significant figures should be displayed. For example, a value of 5 might produce
    /// an output like 1.2345kV or 123.45um. Values less than 3 are unsupported and will assert.
    fn format_metric(self, sig_figs: usize, unit: &str) -> String;
}

impl<T: Float + Display> FloatUtil for T {
    fn from_range(self, from_min: Self, from_max: Self) -> Self {
        (self - from_min) / (from_max - from_min)
    }

    fn to_range(self, to_min: Self, to_max: Self) -> Self {
        self * (to_max - to_min) + to_min
    }

    fn clam(self, min: Self, max: Self) -> Self {
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }

    fn snap(self, min: Self, max: Self, num_steps: Self) -> Self {
        // Unfortunately we can't make this const because of how the compiler works.
        let third: T = T::one() / (T::one() + T::one() + T::one());
        self.from_range(min, max)
            .to_range(-third, num_steps + third)
            .round()
            .from_range_to_range(T::zero(), num_steps, min, max)
    }

    fn format_metric(self, sig_figs: usize, unit: &str) -> String {
        assert!(sig_figs >= 3);
        // ._.
        let one: T = T::one();
        let five: T = one + one + one + one + one;
        let ten = five + five;
        let thousand = ten * ten * ten;
        let mut adjusted_value = self;

        let format_metric_sig_figs = |value: T| -> String {
            let mut precision = sig_figs;
            // We only format values that have between one and three sig figs before the decimal.
            if value >= ten * ten {
                precision -= 3;
            } else if value >= ten {
                precision -= 2;
            } else {
                precision -= 1;
            }
            format!("{0:.1$}", value, precision)
        };

        if adjusted_value.abs() < one {
            // Keep using smaller prefixes until it looks nice.
            const PREFIXES: [&str; 9] = ["", "m", "u", "n", "p", "f", "a", "z", "y"];
            for (index, prefix) in PREFIXES.iter().enumerate() {
                if adjusted_value.abs() >= one {
                    return format!(
                        "{}{}{}",
                        format_metric_sig_figs(adjusted_value),
                        prefix,
                        unit
                    );
                }
                adjusted_value = adjusted_value * thousand;
            }
            // If we get down to the smallest prefix and it still isn't enough, return 0.
            format!("{0:.1$}{2}", 0.0, sig_figs - 1, unit)
        } else {
            // Keep using bigger prefixes until it looks nice.
            const PREFIXES: [&str; 9] = ["", "k", "M", "G", "T", "P", "E", "Z", "Y"];
            for (index, prefix) in PREFIXES.iter().enumerate() {
                if adjusted_value.abs() <= thousand {
                    return format!(
                        "{}{}{}",
                        format_metric_sig_figs(adjusted_value),
                        prefix,
                        unit
                    );
                }
                adjusted_value = adjusted_value / thousand;
            }
            // If we get up to the largest prefix and it still isn't enough, return inf.
            format!("inf {}", unit)
        }
    }
}

pub trait TupleUtil: Sized + Copy {
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
    fn inside(self, bounds: Self) -> bool;
}

impl TupleUtil for (i32, i32) {
    #[inline]
    fn add(self, other: Self) -> Self {
        (self.0 + other.0, self.1 + other.1)
    }

    #[inline]
    fn sub(self, other: Self) -> Self {
        (self.0 - other.0, self.1 - other.1)
    }

    #[inline]
    fn inside(self, bounds: Self) -> bool {
        self.0 >= 0 && self.1 >= 0 && self.0 <= bounds.0 && self.1 <= bounds.1
    }
}

impl TupleUtil for (f32, f32) {
    #[inline]
    fn add(self, other: Self) -> Self {
        (self.0 + other.0, self.1 + other.1)
    }

    #[inline]
    fn sub(self, other: Self) -> Self {
        (self.0 - other.0, self.1 - other.1)
    }

    #[inline]
    fn inside(self, bounds: Self) -> bool {
        self.0 >= 0.0 && self.1 >= 0.0 && self.0 <= bounds.0 && self.1 <= bounds.1
    }
}

pub fn format_decimal(value: f32, digits: i32) -> String {
    let digits = match value {
        v if v <= 0.0 => digits,
        _ => digits - (value.abs().log10().min(digits as f32 - 1.0) as i32),
    };
    let digits = digits as usize;
    format!("{:.*}", digits, value)
}

pub use std::{cell::RefCell, rc::Rc, sync::Arc};

pub type Rcrc<T> = Rc<RefCell<T>>;
pub fn rcrc<T>(content: T) -> Rcrc<T> {
    Rc::new(RefCell::new(content))
}

pub trait IterMapCollect<Item> {
    fn imc<OutItem>(&self, fun: impl FnMut(&Item) -> OutItem) -> Vec<OutItem>;
}

impl<Item> IterMapCollect<Item> for Vec<Item> {
    fn imc<OutItem>(&self, fun: impl FnMut(&Item) -> OutItem) -> Vec<OutItem> {
        self.iter().map(fun).collect()
    }
}

impl<Item> IterMapCollect<Item> for &[Item] {
    fn imc<OutItem>(&self, fun: impl FnMut(&Item) -> OutItem) -> Vec<OutItem> {
        self.iter().map(fun).collect()
    }
}

impl<Item> IterMapCollect<Item> for std::collections::HashSet<Item> {
    fn imc<OutItem>(&self, fun: impl FnMut(&Item) -> OutItem) -> Vec<OutItem> {
        self.iter().map(fun).collect()
    }
}

pub trait RawDataSource {
    fn as_raw(&self) -> &[u8];
    fn as_raw_mut(&mut self) -> &mut [u8];
}

impl<T: Sized> RawDataSource for Vec<T> {
    fn as_raw(&self) -> &[u8] {
        unsafe {
            let out_len = self.len() * std::mem::size_of::<T>();
            std::slice::from_raw_parts(self.as_ptr() as *const u8, out_len)
        }
    }

    fn as_raw_mut(&mut self) -> &mut [u8] {
        unsafe {
            let out_len = self.len() * std::mem::size_of::<T>();
            std::slice::from_raw_parts_mut(self.as_mut_ptr() as *mut u8, out_len)
        }
    }
}
