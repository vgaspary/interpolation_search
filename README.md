# Interpolation Search

Interpolation search is an algorithm for searching in a sorted array. It improves upon the famous binary search by using linear interpolation to better estimate the target's position within the array. Interpolation search reduces the asymptotic time complexity of the search to _O(log log N)_. However, in the worst case scenario (array elements grow exponentially) the complexity becomes linear (_O(N)_).

This crate provides and implements the `InterpolationSearch` trait for slices (and consequently `Vec`s) to provide an `interpolation_search()` alternative to the existing `binary_search()`.

## Usage

1.  Add the crate to your `Cargo.toml` dependencies:

    ```
    cargo add interpolation_search
    ```

2.  Import the trait and the implementation:

    ```
    use interpolation_search::InterpolationSearch;
    ```

3.  Use the `interpolation_search` method on sorted arrays:

    ```
    let arr = [1, 2, 3, 4, 5];
    let target = 3;

    match arr.interpolation_search(&target) {
        Ok(idx) => println!("Target found at index {}", idx),
        Err(idx) => println!("Target not found, possible insertion point: {}", idx),
    }
    ```
## Enabling Interpolation Search for user-defined types

`InterpolationSearch` requires the items in the array to be `Ord` and `InterpolationFactor`.

Consider a simple struct that represents points on a cartesian grid, an `(x, y)` pair. To have them sorted in an array they must be `Ord`, of course, which can be simply derived. Then we implement `InterpolationFactor` for this type.

```
use interpolation_search::{InterpolationSearch, InterpolationFactor};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Point2D {
    x: i32,
    y: i32,
}

impl InterpolationFactor for Point2D {
    fn interpolation_factor(&self, a: &Self, b: &Self) -> f32 {
        if a.x != b.x {
            self.x.interpolation_factor(&a.x, &b.x)
        } else {
            self.y.interpolation_factor(&a.y, &b.y)
        }
    }
}
```

Here's another example where the implementation depends on that on the basic types. Consider a tuple of 3 bytes that represent an RGB color.

```
use interpolation_search::{InterpolationSearch, InterpolationFactor};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Rgb(u8, u8, u8);

impl InterpolationFactor for Rgb {
    fn interpolation_factor(&self, a: &Self, b: &Self) -> f32 {
        if a.0 != b.0 {
            self.0.interpolation_factor(&a.0, &b.0)
        } else if a.1 != b.1 {
            self.1.interpolation_factor(&a.1, &b.1)
        } else {
            self.2.interpolation_factor(&a.2, &b.2)
        }
    }
}
```

>Note: we couldn't just implement `InterpolationFactor` for the tuple `(u8, u8, u8)` as it's a foreign type. We're using the well-known [newtype idiom](https://doc.rust-lang.org/rust-by-example/generics/new_types.html).

## Consistency

The `InterpolationFactor` property of a type must be consistent with its `Ord`. That is, for `a, b, c`, where `a <= b <= c`, `b.interpolation_factor(a, c)` must be in the `[0.0, 1.0]` range.
