//! Module that holds HList data structures, implementations, and typeclasses.
//!
//! Typically, you would want to use the `hlist!` macro to make it easier
//! for you to use HList.
#[derive(PartialEq, Debug, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub struct HNil;

/// Represents the most basic non-empty HList. Its value is held in `head`
/// while its tail is another HList.
#[derive(PartialEq, Debug, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub struct HCons<H, T> {
    pub head: H,
    pub tail: T,
}

impl<Head, Tail> Default for HCons<Head, Tail>
where
    Head: Default,
    Tail: Default,
{
    fn default() -> Self {
        HCons {
            head: Default::default(),
            tail: Default::default(),
        }
    }
}

impl Default for HNil {
    fn default() -> Self {
        HNil
    }
}

impl<'a> ToRef<'a> for HNil {
    type Output = HNil;

    #[inline(always)]
    fn to_ref(&'a self) -> Self::Output {
        HNil
    }
}

impl<'a, H, Tail> ToRef<'a> for HCons<H, Tail>
where
    H: 'a,
    Tail: ToRef<'a>,
{
    type Output = HCons<&'a H, <Tail as ToRef<'a>>::Output>;

    #[inline(always)]
    fn to_ref(&'a self) -> Self::Output {
        HCons {
            head: &self.head,
            tail: self.tail.to_ref(),
        }
    }
}

impl<'a> ToMut<'a> for HNil {
    type Output = HNil;

    #[inline(always)]
    fn to_mut(&'a mut self) -> Self::Output {
        HNil
    }
}

impl<'a, H, Tail> ToMut<'a> for HCons<H, Tail>
where
    H: 'a,
    Tail: ToMut<'a>,
{
    type Output = HCons<&'a mut H, <Tail as ToMut<'a>>::Output>;

    #[inline(always)]
    fn to_mut(&'a mut self) -> Self::Output {
        HCons {
            head: &mut self.head,
            tail: self.tail.to_mut(),
        }
    }
}

pub enum GetHead {}

impl<Head, Tail> TypeMap<HCons<Head, Tail>> for GetHead {
    type Output = Head;
}
impl<Head, Tail> Func<HCons<Head, Tail>> for GetHead {
    fn call(i: HCons<Head, Tail>) -> Self::Output {
        i.head
    }
}

/// Returns an `HList` based on the values passed in.
///
/// Helps to avoid having to write nested `HCons`.
#[macro_export]
macro_rules! hlist {
    () => { $crate::typefu::hlist::HNil };
    (...$rest:expr) => { $rest };
    ($a:expr) => { $crate::hlist![$a,] };
    ($a:expr, $($tok:tt)*) => {
        $crate::typefu::hlist::HCons {
            head: $a,
            tail: $crate::hlist![$($tok)*],
        }
    };
}

use super::{map::TypeMap, Func, ToMut, ToRef};


/// Macro for pattern-matching on HLists.
///
/// Taken from <https://github.com/tbu-/rust-rfcs/blob/master/text/0873-type-macros.md>
#[macro_export]
macro_rules! hlist_pat {
    () => { $crate::typefu::hlist::HNil };
    (...) => { _ };
    (...$rest:pat) => { $rest };
    (_) => { $crate::hlist_pat![_,] };
    ($a:pat) => { $crate::typefu::hlist::hlist_pat![$a,] };
    (_, $($tok:tt)*) => {
        $crate::typefu::hlist::HCons {
            tail: $crate::hlist_pat![$($tok)*],
            ..
        }
    };
    ($a:pat, $($tok:tt)*) => {
        $crate::typefu::hlist::HCons {
            head: $a,
            tail: $crate::hlist_pat![$($tok)*],
        }
    };
}

/// Returns a type signature for an HList of the provided types
///
/// This is a type macro (introduced in Rust 1.13) that makes it easier
/// to write nested type signatures.
#[macro_export]
macro_rules! HList {
    () => { $crate::typefu::hlist::HNil };
    (...$Rest:ty) => { $Rest };
    ($A:ty) => { $crate::HList![$A,] };
    ($A:ty, $($tok:tt)*) => {
        $crate::typefu::hlist::HCons<$A, $crate::HList![$($tok)*]>
    };
}
