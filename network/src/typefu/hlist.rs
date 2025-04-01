//! Module that holds HList data structures, implementations, and typeclasses.
//!
//! Typically, you would want to use the `hlist!` macro to make it easier
//! for you to use HList.
//!
//! # Examples
//!
//! ```
//! # fn main() {
//! use crate::typefu::hlist::{hlist, HList, poly_fn};
//!
//! let h = hlist![1, "hi"];
//! assert_eq!(h.len(), 2);
//! let (a, b) = h.into_tuple2();
//! assert_eq!(a, 1);
//! assert_eq!(b, "hi");
//!
//! // Reverse
//! let h1 = hlist![true, "hi"];
//! assert_eq!(h1.into_reverse(), hlist!["hi", true]);
//!
//! // foldr (foldl also available)
//! let h2 = hlist![1, false, 42f32];
//! let folded = h2.foldr(
//!             hlist![|acc, i| i + acc,
//!                    |acc, _| if acc > 42f32 { 9000 } else { 0 },
//!                    |acc, f| f + acc],
//!             1f32
//!     );
//! assert_eq!(folded, 9001);
//!
//! let h3 = hlist![9000, "joe", 41f32];
//! // Mapping over an HList with a polymorphic function,
//! // declared using the poly_fn! macro (you can choose to impl
//! // it manually)
//! let mapped = h3.map(
//!   poly_fn![
//!     |f: f32|   -> f32 { f + 1f32 },
//!     |i: isize| -> isize { i + 1 },
//!     ['a] |s: &'a str| -> &'a str { s }
//!   ]);
//! assert_eq!(mapped, hlist![9001, "joe", 42f32]);
//!
//! // Plucking a value out by type
//! let h4 = hlist![1, "hello", true, 42f32];
//! let (t, remainder): (bool, _) = h4.pluck();
//! assert!(t);
//! assert_eq!(remainder, hlist![1, "hello", 42f32]);
//!
//! // Resculpting an HList
//! let h5 = hlist![9000, "joe", 41f32, true];
//! let (reshaped, remainder2): (HList![f32, i32, &str], _) = h5.sculpt();
//! assert_eq!(reshaped, hlist![41f32, 9000, "joe"]);
//! assert_eq!(remainder2, hlist![true]);
//! # }
//! ```

/// Represents the right-most end of a heterogeneous list
///
/// # Examples
///
/// ```
/// # use frunk_core::hlist::{h_cons, HNil};
/// let h = h_cons(1, HNil);
/// let h = h.head;
/// assert_eq!(h, 1);
/// ```
#[derive(PartialEq, Debug, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub struct HNil;

/// Represents the most basic non-empty HList. Its value is held in `head`
/// while its tail is another HList.
#[derive(PartialEq, Debug, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub struct HCons<H, T> {
    pub head: H,
    pub tail: T,
}

/// Returns an `HList` based on the values passed in.
///
/// Helps to avoid having to write nested `HCons`.
///
/// # Examples
///
/// ```
/// # use frunk_core::hlist;
/// # fn main() {
/// let h = hlist![13.5f32, "hello", Some(41)];
/// let (h1, (h2, h3)) = h.into_tuple2();
/// assert_eq!(h1, 13.5f32);
/// assert_eq!(h2, "hello");
/// assert_eq!(h3, Some(41));
///
/// // Also works when you have trailing commas
/// let h4 = hlist!["yo",];
/// let h5 = hlist![13.5f32, "hello", Some(41),];
/// assert_eq!(h4, hlist!["yo"]);
/// assert_eq!(h5, hlist![13.5f32, "hello", Some(41)]);
///
/// // Use "...tail" to append an existing list at the end
/// let h6 = hlist![12, ...h5];
/// assert_eq!(h6, hlist![12, 13.5f32, "hello", Some(41)]);
/// # }
/// ```
macro_rules! __hlist {
    () => { $crate::typefu::hlist::HNil };
    (...$rest:expr) => { $rest };
    ($a:expr) => { $crate::typefu::hlist::hlist![$a,] };
    ($a:expr, $($tok:tt)*) => {
        $crate::typefu::hlist::HCons {
            head: $a,
            tail: $crate::typefu::hlist::hlist![$($tok)*],
        }
    };
}
#[allow(unused)]
pub(crate) use __hlist as hlist;
