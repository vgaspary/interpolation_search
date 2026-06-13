use std::str::Chars;
use std::time::SystemTime;

/// Extends types with an `interpolation_factor` method to calculate the interpolation factor of a
/// value between two given values.
///
/// Interpolation search uses linear interpolation to estimate the index of the target. The items
/// of the array must be "linear" in some sense. Given an array `[first, .. , last]` and a `target`,
/// where `first < target < last`, the algorithm calculates the interpolation factor of `target` in
/// the `[first, last]` range to better reduce the search space.
///
/// This crate comes with trivial implementations for integer types and custom implementations for
/// well-known "linear" types, such as `char` and `SystemTime`.
///
/// # Examples
///
/// ```
/// use interpolation_search::InterpolationFactor;
/// use std::time::Duration;
/// use std::time::SystemTime;
///
/// assert_eq!(5.interpolation_factor(&0, &10), 0.5);
/// let t0 = SystemTime::now();
/// let t1 = t0 + Duration::from_secs(2);
/// let t2 = t0 + Duration::from_secs(10);
/// assert_eq!(t1.interpolation_factor(&t0, &t2), 0.2);
/// ```
pub trait InterpolationFactor {
    /// Returns the interpolation factor of `self` in the `[a, b]` linear range. `self` will be
    /// within the range if the slice provided to `interpolation_search` is sorted. This function
    /// must return a value in `[0.0, 1.0]` range.
    fn interpolation_factor(&self, a: &Self, b: &Self) -> f32;
}

macro_rules! trivially_interpolation_factor {
    ($t:ty) => {
        impl InterpolationFactor for $t {
            fn interpolation_factor(&self, a: &Self, b: &Self) -> f32 {
                if a == b {
                    0.5
                } else {
                    let mid = self.clamp(a, b);
                    a.abs_diff(*mid) as f32 / a.abs_diff(*b) as f32
                }
            }
        }
    };
}

trivially_interpolation_factor!(u8);
trivially_interpolation_factor!(u16);
trivially_interpolation_factor!(u32);
trivially_interpolation_factor!(u64);
trivially_interpolation_factor!(u128);
trivially_interpolation_factor!(usize);
trivially_interpolation_factor!(i8);
trivially_interpolation_factor!(i16);
trivially_interpolation_factor!(i32);
trivially_interpolation_factor!(i64);
trivially_interpolation_factor!(i128);
trivially_interpolation_factor!(isize);

impl InterpolationFactor for char {
    fn interpolation_factor(&self, a: &Self, b: &Self) -> f32 {
        u32::from(*self).interpolation_factor(&u32::from(*a), &u32::from(*b))
    }
}

impl InterpolationFactor for SystemTime {
    fn interpolation_factor(&self, a: &Self, b: &Self) -> f32 {
        if a == b {
            0.5
        } else {
            self.duration_since(*a)
                .unwrap_or_default()
                .div_duration_f32(b.duration_since(*a).unwrap_or_default())
        }
    }
}

impl InterpolationFactor for Chars<'_> {
    fn interpolation_factor(&self, a: &Self, b: &Self) -> f32 {
        match self
            .clone()
            .zip(a.clone())
            .zip(b.clone())
            .map(|((mid, a), b)| (mid, a, b))
            .find(|(_, a, b)| a != b)
        {
            Some((mid, a, b)) => mid.interpolation_factor(&a, &b),
            None => 0.5,
        }
    }
}

impl InterpolationFactor for &str {
    fn interpolation_factor(&self, a: &Self, b: &Self) -> f32 {
        self.chars().interpolation_factor(&a.chars(), &b.chars())
    }
}

impl InterpolationFactor for String {
    fn interpolation_factor(&self, a: &Self, b: &Self) -> f32 {
        self.chars().interpolation_factor(&a.chars(), &b.chars())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_u8() {
        assert_eq!(5.interpolation_factor(&0, &10), 0.5);
        assert_eq!(0.interpolation_factor(&0, &10), 0.0);
        assert_eq!(10.interpolation_factor(&0, &10), 1.0);
        assert_eq!(5.interpolation_factor(&5, &5), 0.5); // Test when a == b
    }

    #[test]
    fn test_u16() {
        assert_eq!(5000.interpolation_factor(&0, &10000), 0.5);
        assert_eq!(0.interpolation_factor(&0, &10000), 0.0);
        assert_eq!(10000.interpolation_factor(&0, &10000), 1.0);
        assert_eq!(5000.interpolation_factor(&5000, &5000), 0.5);
    }

    #[test]
    fn test_u32() {
        assert_eq!(50000.interpolation_factor(&0, &100000), 0.5);
        assert_eq!(0.interpolation_factor(&0, &100000), 0.0);
        assert_eq!(100000.interpolation_factor(&0, &100000), 1.0);
        assert_eq!(50000.interpolation_factor(&50000, &50000), 0.5);
    }

    #[test]
    fn test_u64() {
        assert_eq!(500000.interpolation_factor(&0, &1000000), 0.5);
        assert_eq!(0.interpolation_factor(&0, &1000000), 0.0);
        assert_eq!(1000000.interpolation_factor(&0, &1000000), 1.0);
        assert_eq!(500000.interpolation_factor(&500000, &500000), 0.5);
    }

    #[test]
    fn test_u128() {
        assert_eq!(5000000.interpolation_factor(&0, &10000000), 0.5);
        assert_eq!(0.interpolation_factor(&0, &10000000), 0.0);
        assert_eq!(10000000.interpolation_factor(&0, &10000000), 1.0);
        assert_eq!(5000000.interpolation_factor(&5000000, &5000000), 0.5);
    }

    #[test]
    fn test_usize() {
        assert_eq!(500.interpolation_factor(&0, &1000), 0.5);
        assert_eq!(0.interpolation_factor(&0, &1000), 0.0);
        assert_eq!(1000.interpolation_factor(&0, &1000), 1.0);
        assert_eq!(500.interpolation_factor(&500, &500), 0.5);
    }

    #[test]
    fn test_i8() {
        assert_eq!((-5).interpolation_factor(&(-10), &0), 0.5);
        assert_eq!((-10).interpolation_factor(&(-10), &0), 0.0);
        assert_eq!(0.interpolation_factor(&(-10), &0), 1.0);
        assert_eq!((-5).interpolation_factor(&(-5), &(-5)), 0.5);
    }

    #[test]
    fn test_i16() {
        assert_eq!((-5000).interpolation_factor(&(-10000), &0), 0.5);
        assert_eq!((-10000).interpolation_factor(&(-10000), &0), 0.0);
        assert_eq!(0.interpolation_factor(&(-10000), &0), 1.0);
        assert_eq!((-5000).interpolation_factor(&(-5000), &(-5000)), 0.5);
    }

    #[test]
    fn test_i32() {
        assert_eq!((-50000).interpolation_factor(&(-100000), &0), 0.5);
        assert_eq!((-100000).interpolation_factor(&(-100000), &0), 0.0);
        assert_eq!(0.interpolation_factor(&(-100000), &0), 1.0);
        assert_eq!((-50000).interpolation_factor(&(-50000), &(-50000)), 0.5);
    }

    #[test]
    fn test_i64() {
        assert_eq!((-500000).interpolation_factor(&(-1000000), &0), 0.5);
        assert_eq!((-1000000).interpolation_factor(&(-1000000), &0), 0.0);
        assert_eq!(0.interpolation_factor(&(-1000000), &0), 1.0);
        assert_eq!((-500000).interpolation_factor(&(-500000), &(-500000)), 0.5);
    }

    #[test]
    fn test_i128() {
        assert_eq!((-5000000).interpolation_factor(&(-10000000), &0), 0.5);
        assert_eq!((-10000000).interpolation_factor(&(-10000000), &0), 0.0);
        assert_eq!(0.interpolation_factor(&(-10000000), &0), 1.0);
        assert_eq!(
            (-5000000).interpolation_factor(&(-5000000), &(-5000000)),
            0.5
        );
    }

    #[test]
    fn test_isize() {
        assert_eq!((-500).interpolation_factor(&(-1000), &0), 0.5);
        assert_eq!((-1000).interpolation_factor(&(-1000), &0), 0.0);
        assert_eq!(0.interpolation_factor(&(-1000), &0), 1.0);
        assert_eq!((-500).interpolation_factor(&(-500), &(-500)), 0.5);
    }

    #[test]
    fn test_char() {
        assert_eq!('c'.interpolation_factor(&'a', &'e'), 0.5);
        assert_eq!('a'.interpolation_factor(&'a', &'e'), 0.0);
        assert_eq!('e'.interpolation_factor(&'a', &'e'), 1.0);
        assert_eq!('c'.interpolation_factor(&'c', &'c'), 0.5);
    }

    #[test]
    fn test_system_time() {
        let t0 = SystemTime::now();
        let t1 = t0 + Duration::from_secs(1);
        let t2 = t0 + Duration::from_secs(2);
        assert_eq!(t1.interpolation_factor(&t0, &t2), 0.5);
        assert_eq!(t0.interpolation_factor(&t0, &t2), 0.0);
        assert_eq!(t2.interpolation_factor(&t0, &t2), 1.0);
        assert_eq!(t1.interpolation_factor(&t1, &t1), 0.5);

        // Test with zero duration to avoid potential division by zero
        let t3 = SystemTime::now();
        let t4 = t3;
        assert_eq!(t3.interpolation_factor(&t3, &t4), 0.5);
    }

    #[test]
    fn test_str() {
        assert_eq!("ccc".interpolation_factor(&"aaa", &"eee"), 0.5);
        assert_eq!("aaa".interpolation_factor(&"aaa", &"eee"), 0.0);
        assert_eq!("eee".interpolation_factor(&"aaa", &"eee"), 1.0);
        assert_eq!("ccc".interpolation_factor(&"ccc", &"ccc"), 0.5);

        assert_eq!("c".interpolation_factor(&"ab", &"cd"), 1.0);
        assert_eq!("cc".interpolation_factor(&"a", &"e"), 0.5);
        assert_eq!("ab".interpolation_factor(&"ab", &"cde"), 0.0);
        assert_eq!("cd".interpolation_factor(&"ab", &"cde"), 1.0);

        assert_eq!("xyz".interpolation_factor(&"abc", &"def"), 1.0);
    }

    #[test]
    fn test_string() {
        let s1 = String::from("ccc");
        let s2 = String::from("aaa");
        let s3 = String::from("eee");
        assert_eq!(s1.interpolation_factor(&s2, &s3), 0.5);
        assert_eq!(s2.interpolation_factor(&s2, &s3), 0.0);
        assert_eq!(s3.interpolation_factor(&s2, &s3), 1.0);
        assert_eq!(s1.interpolation_factor(&s1.clone(), &s1), 0.5);
    }
}
