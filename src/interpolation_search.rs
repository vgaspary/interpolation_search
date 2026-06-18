use crate::InterpolationFactor;
use std::cmp::Ord;
use std::cmp::Ordering::Equal;
use std::cmp::Ordering::Greater;
use std::cmp::Ordering::Less;

pub trait InterpolationSearch<T> {
    /// Interpolation searches this slice for a given element. If the slice is not sorted, the returned result is unspecified and meaningless.
    ///
    /// The interface of this funciton is similar to its `binary_search` counterpart. If the value is found then `Result::Ok` is returned, containing the index of the matching element. If there are multiple matches, then any one of the matches could be returned. The index is chosen deterministically, but is subject to change in future versions of the crate. If the value is not found then `Result::Err` is returned, containing the index where a matching element could be inserted while maintaining sorted order.
    ///
    /// **Examples**
    ///
    /// ```
    /// use interpolation_search::InterpolationSearch;
    ///
    /// let arr = [0, 1, 1, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55];
    ///
    /// assert_eq!(arr.interpolation_search(&13), Ok(9));
    /// assert_eq!(arr.interpolation_search(&14), Err(10));
    /// assert_eq!(arr.interpolation_search(&4), Err(7));
    /// assert_eq!(arr.interpolation_search(&100), Err(13));
    /// assert!(matches!(arr.interpolation_search(&1) , Ok(1..=4)));
    /// ```
    fn interpolation_search(&self, target: &T) -> Result<usize, usize>
    where
        T: InterpolationFactor;

    /// Interpolation searches this slice with a key extaction function. If the slice is not sorted by keys, the returned result is unspecified and meaningless.
    ///
    /// The interface of this funciton is similar to its `binary_search_by_key` counterpart. If the value is found then `Result::Ok` is returned, containing the index of the matching element. If there are multiple matches, then any one of the matches could be returned. The index is chosen deterministically, but is subject to change in future versions of the crate. If the value is not found then `Result::Err` is returned, containing the index where a matching element could be inserted while maintaining sorted by key order.
    ///
    /// **Examples**
    ///
    /// ```
    /// use interpolation_search::InterpolationSearch;
    ///
    /// let s = [(0, 0), (2, 1), (4, 1), (5, 1), (3, 1),
    ///         (1, 2), (2, 3), (4, 5), (5, 8), (3, 13),
    ///         (1, 21), (2, 34), (4, 55)];
    ///
    /// assert_eq!(s.interpolation_search_by_key(&13, |(a, b)| b),  Ok(9));
    /// assert_eq!(s.interpolation_search_by_key(&4, |(a, b)| b),   Err(7));
    /// assert_eq!(s.interpolation_search_by_key(&100, |(a, b)| b), Err(13));
    /// assert!(matches!(s.interpolation_search_by_key(&1, |(a, b)| b), Ok(1..=4)));
    /// ```
    fn interpolation_search_by_key<K, F>(&self, target: &K, f: F) -> Result<usize, usize>
    where
        K: InterpolationFactor,
        F: FnMut(&T) -> &K;
}

impl<T> InterpolationSearch<T> for [T] {
    fn interpolation_search(&self, target: &T) -> Result<usize, usize>
    where
        T: InterpolationFactor,
    {
        self.interpolation_search_by_key(target, |x| x)
    }

    fn interpolation_search_by_key<K, F>(&self, target: &K, mut key: F) -> Result<usize, usize>
    where
        K: InterpolationFactor,
        F: FnMut(&T) -> &K,
    {
        let mut first_idx = 0;
        let mut last_idx = self.len();
        loop {
            match &self[first_idx..last_idx] {
                [] => return Err(first_idx),
                [first, ..] if target < key(first) => return Err(first_idx),
                [single] if key(single) == target => return Ok(first_idx),
                [.., last] if key(last) < target => return Err(last_idx),
                [first, .., last] => {
                    let f = target.interpolation_factor(key(first), key(last));
                    let mid_idx = lerp_idx(first_idx, last_idx, f);
                    let mid = &self[mid_idx];
                    match key(mid).cmp(target) {
                        Equal => return Ok(mid_idx),
                        Greater => last_idx = mid_idx,
                        Less => first_idx = mid_idx + 1,
                    }
                }
                [_] => return Err(0), // Should not happen if the array is sorted
            }
        }
    }
}

// Returns an index in a given inclusive-exclusive index range (`[first, last)`).
fn lerp_idx(first: usize, last: usize, f: f32) -> usize {
    if first >= last {
        return first;
    }
    (first + ((last - first) as f32 * normalize(f)) as usize).min(last - 1)
}

fn normalize(f: f32) -> f32 {
    if !f.is_normal() && f != 0.0 {
        0.5
    } else {
        f.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use std::time::SystemTime;

    #[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
    struct Item {
        id: usize,
        value: i32,
        name: String,
    }

    #[test]
    fn test_against_binary_search() {
        let arr = [1, 2, 3, 3, 4, 5, 6, 6, 6, 7, 8, 8, 8, 8, 9, 10];
        for n in 0..=11 {
            match arr.interpolation_search(&n) {
                Ok(idx) => {
                    assert!(arr.binary_search(&n).is_ok());
                    assert_eq!(arr[idx], n);
                }
                Err(idx) => assert_eq!(Err(idx), arr.binary_search(&n)),
            }
        }
    }

    #[test]
    fn test_empty_array() {
        let arr = [];
        assert_eq!(arr.interpolation_search(&0), Err(0));
        assert_eq!(arr.interpolation_search(&1), Err(0));
        assert_eq!(arr.interpolation_search(&-1), Err(0));
    }

    #[test]
    fn test_single_element() {
        let arr = [0];
        assert_eq!(arr.interpolation_search(&0), Ok(0));
        assert_eq!(arr.interpolation_search(&1), Err(1));
        assert_eq!(arr.interpolation_search(&-1), Err(0));
    }

    #[test]
    fn test_repeating_element() {
        let arr = [0, 0, 0, 0, 0];
        assert!(arr.interpolation_search(&0).is_ok_and(|n| n < 5));
        assert_eq!(arr.interpolation_search(&1), Err(5));
        assert_eq!(arr.interpolation_search(&-1), Err(0));
    }

    #[test]
    fn test_integer_types() {
        let arr = [0_i8];
        assert_eq!(arr.interpolation_search(&0), Ok(0));
        let arr = [0_i16];
        assert_eq!(arr.interpolation_search(&0), Ok(0));
        let arr = [0_i32];
        assert_eq!(arr.interpolation_search(&0), Ok(0));
        let arr = [0_i64];
        assert_eq!(arr.interpolation_search(&0), Ok(0));
        let arr = [0_i128];
        assert_eq!(arr.interpolation_search(&0), Ok(0));
        let arr = [0_isize];
        assert_eq!(arr.interpolation_search(&0), Ok(0));

        let arr = [0_u8];
        assert_eq!(arr.interpolation_search(&0), Ok(0));
        let arr = [0_u16];
        assert_eq!(arr.interpolation_search(&0), Ok(0));
        let arr = [0_u32];
        assert_eq!(arr.interpolation_search(&0), Ok(0));
        let arr = [0_u64];
        assert_eq!(arr.interpolation_search(&0), Ok(0));
        let arr = [0_u128];
        assert_eq!(arr.interpolation_search(&0), Ok(0));
        let arr = [0_usize];
        assert_eq!(arr.interpolation_search(&0), Ok(0));
    }

    #[test]
    fn test_time_points() {
        let t0 = SystemTime::now();
        let arr = (0..10)
            .map(|n| Duration::from_secs(n))
            .map(|delay| t0 + delay)
            .collect::<Vec<_>>();
        assert_eq!(arr.interpolation_search(&t0), Ok(0));
        assert_eq!(
            arr.interpolation_search(&(t0 + Duration::from_secs(5))),
            Ok(5)
        );
        assert_eq!(
            arr.interpolation_search(&(t0 + Duration::from_secs(15))),
            Err(10)
        );
    }

    #[test]
    fn test_chars() {
        let arr = ('a'..='z').collect::<Vec<_>>();
        assert_eq!(arr.interpolation_search(&'a'), Ok(0));
        assert_eq!(arr.interpolation_search(&'z'), Ok(25));
        assert_eq!(arr.interpolation_search(&'A'), Err(0));
        assert_eq!(arr.interpolation_search(&'{'), Err(26));
        assert_eq!(arr.interpolation_search(&'ա'), Err(26));

        let arr = ('ա'..='և').collect::<Vec<_>>();
        assert_eq!(arr.interpolation_search(&'ա'), Ok(0));
        assert_eq!(arr.interpolation_search(&'և'), Ok(38));
        assert_eq!(arr.interpolation_search(&'Ա'), Err(0));
    }

    #[test]
    fn test_normalize() {
        assert_eq!(normalize(0.0), 0.0);
        assert_eq!(normalize(1.0), 1.0);
        assert_eq!(normalize(0.5), 0.5);
        assert_eq!(normalize(0.25), 0.25);
        assert_eq!(normalize(0.75), 0.75);

        assert_eq!(normalize(-1.0), 0.0);
        assert_eq!(normalize(2.0), 1.0);

        assert_eq!(normalize(f32::NAN), 0.5);
        assert_eq!(normalize(f32::INFINITY), 0.5);
        assert_eq!(normalize(f32::NEG_INFINITY), 0.5);
        assert_eq!(normalize(f32::MIN_POSITIVE), f32::MIN_POSITIVE);

        assert_eq!(normalize(f32::MIN_POSITIVE / 2.0), 0.5);
        assert_eq!(normalize(f32::MIN_POSITIVE * -1.0 / 2.0), 0.5);
    }

    #[test]
    fn test_lerp_idx() {
        assert_eq!(lerp_idx(0, 10, 0.0), 0);
        assert_eq!(lerp_idx(0, 10, 1.0), 9);
        assert_eq!(lerp_idx(0, 10, 0.5), 5);
        assert_eq!(lerp_idx(0, 10, 0.25), 2);
        assert_eq!(lerp_idx(0, 10, 0.75), 7);

        assert_eq!(lerp_idx(0, 1, 0.0), 0);
        assert_eq!(lerp_idx(0, 1, 1.0), 0);

        assert_eq!(lerp_idx(0, 0, 0.0), 0);
        assert_eq!(lerp_idx(0, 0, 1.0), 0);

        // Testing out-of-bounds factors.
        assert_eq!(lerp_idx(0, 10, -1.0), 0);
        assert_eq!(lerp_idx(0, 10, 2.0), 9);
        assert_eq!(lerp_idx(0, 10, f32::NAN), 5);
        assert_eq!(lerp_idx(0, 10, f32::INFINITY), 5);
        assert_eq!(lerp_idx(0, 10, f32::NEG_INFINITY), 5);
        assert_eq!(lerp_idx(0, 10, f32::MIN_POSITIVE / 2.0), 5);
        assert_eq!(lerp_idx(0, 10, f32::MIN_POSITIVE * -1.0 / 2.0), 5);

        assert_eq!(lerp_idx(5, 15, 0.0), 5);
        assert_eq!(lerp_idx(5, 15, 1.0), 14);
        assert_eq!(lerp_idx(5, 15, 0.5), 10);

        assert_eq!(lerp_idx(10, 5, 0.0), 10);
    }

    #[test]
    fn test_str_interpolation_search() {
        let strings = vec!["apple", "banana", "cherry", "date", "elderberry"];

        assert_eq!(strings.interpolation_search(&"apple"), Ok(0));
        assert_eq!(strings.interpolation_search(&"date"), Ok(3));
        assert_eq!(strings.interpolation_search(&"grape"), Err(5));
        assert_eq!(strings.interpolation_search(&"apricot"), Err(1));
        assert_eq!(strings.interpolation_search(&"bat"), Err(2));

        let empty_strings: Vec<&str> = Vec::new();
        assert_eq!(empty_strings.interpolation_search(&"anything"), Err(0));

        let single_string = vec!["only"];
        assert_eq!(single_string.interpolation_search(&"only"), Ok(0));
        assert_eq!(single_string.interpolation_search(&"aaa"), Err(0));
        assert_eq!(single_string.interpolation_search(&"zzz"), Err(1));

        let repeated_strings = vec!["same", "same", "same"];
        assert!(repeated_strings
            .interpolation_search(&"same")
            .is_ok_and(|n| n < 3));
    }

    #[test]
    fn test_interpolation_search_by_key_tuple() {
        // Sorted by the second element (i32)
        let data = [
            (1, 10),
            (5, 20),
            (2, 30),
            (8, 30), // Duplicate key
            (3, 40),
            (7, 50),
            (4, 60),
        ];

        // Search for existing keys
        assert_eq!(data.interpolation_search_by_key(&10, |pair| &pair.1), Ok(0));
        assert_eq!(data.interpolation_search_by_key(&20, |pair| &pair.1), Ok(1));
        assert!(matches!(
            data.interpolation_search_by_key(&30, |pair| &pair.1),
            Ok(2..=3)
        ));
        assert_eq!(data.interpolation_search_by_key(&40, |pair| &pair.1), Ok(4));
        assert_eq!(data.interpolation_search_by_key(&50, |pair| &pair.1), Ok(5));
        assert_eq!(data.interpolation_search_by_key(&60, |pair| &pair.1), Ok(6));

        // Search for non-existing keys
        assert_eq!(data.interpolation_search_by_key(&5, |pair| &pair.1), Err(0)); // Before first
        assert_eq!(
            data.interpolation_search_by_key(&15, |pair| &pair.1),
            Err(1)
        ); // Between 10 and 20
        assert_eq!(
            data.interpolation_search_by_key(&35, |pair| &pair.1),
            Err(4)
        ); // Between 30 and 40
        assert_eq!(
            data.interpolation_search_by_key(&65, |pair| &pair.1),
            Err(7)
        ); // After last
    }

    #[test]
    fn test_interpolation_search_by_key_struct() {
        // Sorted by the 'value' field
        let data = [
            Item {
                id: 1,
                value: 10,
                name: "apple".to_string(),
            },
            Item {
                id: 5,
                value: 20,
                name: "banana".to_string(),
            },
            Item {
                id: 2,
                value: 30,
                name: "cherry".to_string(),
            },
            Item {
                id: 8,
                value: 30,
                name: "date".to_string(),
            }, // Duplicate key
            Item {
                id: 3,
                value: 40,
                name: "elderberry".to_string(),
            },
            Item {
                id: 7,
                value: 50,
                name: "fig".to_string(),
            },
            Item {
                id: 4,
                value: 60,
                name: "grape".to_string(),
            },
        ];

        // Search for existing keys
        assert_eq!(
            data.interpolation_search_by_key(&10, |item| &item.value),
            Ok(0)
        );
        assert_eq!(
            data.interpolation_search_by_key(&20, |item| &item.value),
            Ok(1)
        );
        // Should find either index 2 or 3
        assert!(matches!(
            data.interpolation_search_by_key(&30, |item| &item.value),
            Ok(2..=3)
        ));
        assert_eq!(
            data.interpolation_search_by_key(&40, |item| &item.value),
            Ok(4)
        );
        assert_eq!(
            data.interpolation_search_by_key(&50, |item| &item.value),
            Ok(5)
        );
        assert_eq!(
            data.interpolation_search_by_key(&60, |item| &item.value),
            Ok(6)
        );

        // Search for non-existing keys
        assert_eq!(
            data.interpolation_search_by_key(&5, |item| &item.value),
            Err(0)
        ); // Before first
        assert_eq!(
            data.interpolation_search_by_key(&15, |item| &item.value),
            Err(1)
        ); // Between 10 and 20
        assert_eq!(
            data.interpolation_search_by_key(&35, |item| &item.value),
            Err(4)
        ); // Between 30 and 40
        assert_eq!(
            data.interpolation_search_by_key(&65, |item| &item.value),
            Err(7)
        ); // After last
    }

    #[test]
    fn test_interpolation_search_by_key_empty() {
        let data: Vec<Item> = Vec::new();
        assert_eq!(
            data.interpolation_search_by_key(&10, |item| &item.value),
            Err(0)
        );
    }

    #[test]
    fn test_interpolation_search_by_key_single() {
        let data = [Item {
            id: 1,
            value: 10,
            name: "single".to_string(),
        }];
        assert_eq!(
            data.interpolation_search_by_key(&10, |item| &item.value),
            Ok(0)
        );
        assert_eq!(
            data.interpolation_search_by_key(&5, |item| &item.value),
            Err(0)
        ); // Before
        assert_eq!(
            data.interpolation_search_by_key(&15, |item| &item.value),
            Err(1)
        ); // After
    }

    #[test]
    fn test_interpolation_search_by_key_chars() {
        // Sorted by the char element
        let data = [
            (1, 'a'),
            (2, 'c'),
            (3, 'f'),
            (4, 'f'), // Duplicate key
            (5, 'm'),
            (6, 'z'),
        ];

        // Search for existing keys
        assert_eq!(
            data.interpolation_search_by_key(&'a', |pair| &pair.1),
            Ok(0)
        );
        assert_eq!(
            data.interpolation_search_by_key(&'c', |pair| &pair.1),
            Ok(1)
        );
        assert!(matches!(
            data.interpolation_search_by_key(&'f', |pair| &pair.1),
            Ok(2..=3)
        ));
        assert_eq!(
            data.interpolation_search_by_key(&'m', |pair| &pair.1),
            Ok(4)
        );
        assert_eq!(
            data.interpolation_search_by_key(&'z', |pair| &pair.1),
            Ok(5)
        );

        // Search for non-existing keys
        assert_eq!(
            data.interpolation_search_by_key(&'9', |pair| &pair.1),
            Err(0)
        ); // Before first ('a')
        assert_eq!(
            data.interpolation_search_by_key(&'b', |pair| &pair.1),
            Err(1)
        ); // Between 'a' and 'c'
        assert_eq!(
            data.interpolation_search_by_key(&'d', |pair| &pair.1),
            Err(2)
        ); // Between 'c' and 'f'
        assert_eq!(
            data.interpolation_search_by_key(&'g', |pair| &pair.1),
            Err(4)
        ); // Between 'f' and 'm'
        assert_eq!(
            data.interpolation_search_by_key(&'y', |pair| &pair.1),
            Err(5)
        ); // Between 'm' and 'z'
        assert_eq!(
            data.interpolation_search_by_key(&'{', |pair| &pair.1),
            Err(6)
        ); // After last ('z')
    }

    #[test]
    fn test_interpolation_search_by_key_strings() {
        // Sorted by the string element
        let data = [
            (1, "apple"),
            (2, "banana"),
            (3, "banana"), // Duplicate key
            (4, "cherry"),
            (5, "date"),
            (6, "date"), // Duplicate key
            (7, "elderberry"),
        ];

        // Search for existing keys
        assert_eq!(
            data.interpolation_search_by_key(&"apple", |pair| &pair.1),
            Ok(0)
        );
        let result_banana = data.interpolation_search_by_key(&"banana", |pair| &pair.1);
        assert!(result_banana == Ok(1) || result_banana == Ok(2));
        assert_eq!(
            data.interpolation_search_by_key(&"cherry", |pair| &pair.1),
            Ok(3)
        );
        let result_date = data.interpolation_search_by_key(&"date", |pair| &pair.1);
        assert!(result_date == Ok(4) || result_date == Ok(5));
        assert_eq!(
            data.interpolation_search_by_key(&"elderberry", |pair| &pair.1),
            Ok(6)
        );

        // Search for non-existing keys
        assert_eq!(
            data.interpolation_search_by_key(&"apricot", |pair| &pair.1),
            Err(1)
        ); // Between apple and banana
        assert_eq!(
            data.interpolation_search_by_key(&"blueberry", |pair| &pair.1),
            Err(3)
        ); // Between banana and cherry
        assert_eq!(
            data.interpolation_search_by_key(&"dragonfruit", |pair| &pair.1),
            Err(6)
        ); // Between date and elderberry
        assert_eq!(
            data.interpolation_search_by_key(&"zebra", |pair| &pair.1),
            Err(7)
        ); // After elderberry
        assert_eq!(
            data.interpolation_search_by_key(&"a", |pair| &pair.1),
            Err(0)
        ); // Before apple
    }
}
