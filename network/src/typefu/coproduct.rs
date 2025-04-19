//! Module that holds Coproduct data structures, traits, and implementations
//!
//! Think of "Coproduct" as ad-hoc enums.

use std::{ops::DerefMut, pin::Pin};

use super::{
    ToMut, ToRef,
    index::{Here, There},
};

/// Enum type representing a Coproduct. Think of this as a Result, but capable
/// of supporting any arbitrary number of types instead of just 2.
///
/// To construct a Coproduct, you would typically declare a type using the `Coprod!` type
/// macro and then use the `inject` method.
#[derive(PartialEq, Debug, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum Coproduct<H, T> {
    /// Coproduct is either H or T, in this case, it is H
    Inl(H),
    /// Coproduct is either H or T, in this case, it is T
    Inr(T),
}

/// Phantom type for signature purposes only (has no value)
///
/// Used by the macro to terminate the Coproduct type signature
#[derive(PartialEq, Debug, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum CNil {}

impl<Head, Tail> Coproduct<Head, Tail> {
    /// Instantiate a coproduct from an element.
    ///
    /// This is generally much nicer than nested usage of `Coproduct::{Inl, Inr}`.
    /// The method uses a trick with type inference to automatically build the correct variant
    /// according to the input type.
    ///
    /// In standard usage, the `Index` type parameter can be ignored,
    /// as it will typically be solved for using type inference.
    ///
    /// # Rules
    ///
    /// If the type does not appear in the coproduct, the conversion is forbidden.
    ///
    /// If the type appears multiple times in the coproduct, type inference will fail.
    #[inline(always)]
    pub fn inject<T, Index>(to_insert: T) -> Self
    where
        Self: CoprodInjector<T, Index>,
    {
        CoprodInjector::inject(to_insert)
    }

    /// Attempt to extract a value from a coproduct (or get the remaining possibilities).
    ///
    /// By chaining calls to this, one can exhaustively match all variants of a coproduct.
    #[inline(always)]
    pub fn uninject<T, Index>(self) -> Result<T, <Self as CoprodUninjector<T, Index>>::Remainder>
    where
        Self: CoprodUninjector<T, Index>,
    {
        CoprodUninjector::uninject(self)
    }
}

impl<Head, Tail> Default for Coproduct<Head, Tail>
where
    Head: Default,
{
    fn default() -> Self {
        Coproduct::Inl(Head::default())
    }
}

impl<Head, Tail> Future for Coproduct<Head, Tail>
where
    Head: Future + Unpin,
    Tail: Future<Output = Head::Output> + Unpin,
{
    type Output = Head::Output;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.deref_mut() {
            Coproduct::Inl(l) => Pin::new(l).poll(cx),
            Coproduct::Inr(r) => Pin::new(r).poll(cx),
        }
    }
}

/// Trait for instantiating a coproduct from an element
///
/// This trait is part of the implementation of the inherent static method
/// [`Coproduct::inject`]. Please see that method for more information.
///
/// You only need to import this trait when working with generic
/// Coproducts of unknown type. In most code, `Coproduct::inject` will
/// "just work," with or without this trait.
///
/// [`Coproduct::inject`]: enum.Coproduct.html#method.inject
pub trait CoprodInjector<InjectType, Index> {
    /// Instantiate a coproduct from an element.
    ///
    /// Please see the [inherent static method] for more information.
    ///
    /// The only difference between that inherent method and this
    /// trait method is the location of the type parameters.
    /// (here, they are on the trait rather than the method)
    ///
    /// [inherent static method]: enum.Coproduct.html#method.inject
    fn inject(to_insert: InjectType) -> Self;
}

impl<I, Tail> CoprodInjector<I, Here> for Coproduct<I, Tail> {
    fn inject(to_insert: I) -> Self {
        Coproduct::Inl(to_insert)
    }
}

impl<Head, I, Tail, TailIndex> CoprodInjector<I, There<TailIndex>> for Coproduct<Head, Tail>
where
    Tail: CoprodInjector<I, TailIndex>,
{
    fn inject(to_insert: I) -> Self {
        let tail_inserted = <Tail as CoprodInjector<I, TailIndex>>::inject(to_insert);
        Coproduct::Inr(tail_inserted)
    }
}

impl<'a, CH: 'a, CTail> ToRef<'a> for Coproduct<CH, CTail>
where
    CTail: ToRef<'a>,
{
    type Output = Coproduct<&'a CH, <CTail as ToRef<'a>>::Output>;

    #[inline(always)]
    fn to_ref(&'a self) -> Self::Output {
        match *self {
            Coproduct::Inl(ref r) => Coproduct::Inl(r),
            Coproduct::Inr(ref rest) => Coproduct::Inr(rest.to_ref()),
        }
    }
}

impl<'a> ToRef<'a> for CNil {
    type Output = CNil;

    fn to_ref(&'a self) -> CNil {
        match *self {}
    }
}

impl<'a, CH: 'a, CTail> ToMut<'a> for Coproduct<CH, CTail>
where
    CTail: ToMut<'a>,
{
    type Output = Coproduct<&'a mut CH, <CTail as ToMut<'a>>::Output>;

    #[inline(always)]
    fn to_mut(&'a mut self) -> Self::Output {
        match *self {
            Coproduct::Inl(ref mut r) => Coproduct::Inl(r),
            Coproduct::Inr(ref mut rest) => Coproduct::Inr(rest.to_mut()),
        }
    }
}

impl<'a> ToMut<'a> for CNil {
    type Output = CNil;

    fn to_mut(&'a mut self) -> CNil {
        match *self {}
    }
}

/// Trait for extracting a value from a coproduct in an exhaustive way.
///
/// This trait is part of the implementation of the inherent method
/// [`Coproduct::uninject`]. Please see that method for more information.
///
/// You only need to import this trait when working with generic
/// Coproducts of unknown type. If you have a Coproduct of known type,
/// then `co.uninject()` should "just work" even without the trait.
///
/// [`Coproduct::uninject`]: enum.Coproduct.html#method.uninject
pub trait CoprodUninjector<T, Idx>: CoprodInjector<T, Idx> {
    type Remainder;

    /// Attempt to extract a value from a coproduct (or get the remaining possibilities).
    ///
    /// Please see the [inherent method] for more information.
    ///
    /// The only difference between that inherent method and this
    /// trait method is the location of the type parameters.
    /// (here, they are on the trait rather than the method)
    ///
    /// [inherent method]: enum.Coproduct.html#method.uninject
    fn uninject(self) -> Result<T, Self::Remainder>;
}

impl<Hd, Tl> CoprodUninjector<Hd, Here> for Coproduct<Hd, Tl> {
    type Remainder = Tl;

    fn uninject(self) -> Result<Hd, Tl> {
        match self {
            Coproduct::Inl(h) => Ok(h),
            Coproduct::Inr(t) => Err(t),
        }
    }
}

impl<Hd, Tl, T, N> CoprodUninjector<T, There<N>> for Coproduct<Hd, Tl>
where
    Tl: CoprodUninjector<T, N>,
{
    type Remainder = Coproduct<Hd, Tl::Remainder>;

    fn uninject(self) -> Result<T, Self::Remainder> {
        match self {
            Coproduct::Inl(h) => Err(Coproduct::Inl(h)),
            Coproduct::Inr(t) => t.uninject().map_err(Coproduct::Inr),
        }
    }
}

#[macro_export]
macro_rules! comatch {
    ($value:expr; _: $t:ty => $e:expr, $($tail:tt)*) => {
        match $value.uninject::<$t, _>() {
            Ok(_) => $e,
            Err(co) => $crate::comatch!(co; $($tail)*)
        }
    };
    ($value:expr; $var:ident: $t:ty => $e:expr, $($tail:tt)*) => {
        match $value.uninject::<$t, _>() {
            Ok($var) => $e,
            Err(co) => $crate::comatch!(co; $($tail)*)
        }
    };
    ($value:expr; _ => $e:expr $(,)?) => {
        $e
    };
    ($value:expr; $p:pat => $e:expr, $($tail:tt)*) => {
        match $value.uninject() {
            Ok($p) => $e,
            Err(co) => $crate::comatch!(co; $($tail)*)
        }
    };
    ($value:expr;) => {
        match $value {}
    }
}
#[allow(unused)]
pub(crate) use comatch;

/// Returns a type signature for a Coproduct of the provided types
///
/// This is a type macro (introduced in Rust 1.13) that makes it easier
/// to write nested type signatures.
macro_rules! Coprod {
    () => { $crate::typefu::coproduct::CNil };
    (...$Rest:ty) => { $Rest };
    ($A:ty) => { $crate::typefu::coproduct::Coprod![$A,] };
    ($A:ty, $($tok:tt)*) => {
        $crate::typefu::coproduct::Coproduct<$A, $crate::typefu::coproduct::Coprod![$($tok)*]>
    };
}
pub(crate) use Coprod;

#[cfg(test)]
mod tests {

    use super::Coproduct::*;
    use super::*;

    #[test]
    fn coproduct_inject() {
        type I32StrBool = Coprod!(i32, &'static str, bool);

        let co1 = I32StrBool::inject(3);
        assert_eq!(co1, Inl(3));
        match co1.uninject::<bool, _>() {
            Ok(_) => panic!("Expected an error"),
            Err(co) => assert_eq!(co, Inl(3)),
        }
        match co1.uninject::<i32, _>() {
            Ok(v) => assert_eq!(v, 3),
            Err(_) => panic!("Expected a value"),
        }

        let co2 = I32StrBool::inject(false);
        assert_eq!(co2, Inr(Inr(Inl(false))));
        match co2.uninject::<i32, _>() {
            Ok(_) => panic!("Expected an error"),
            Err(_) => assert_eq!(co2.uninject::<bool, _>(), Ok(false)),
        }
    }

    #[test]
    fn coproduct_uninject() {
        type I32StrBool = Coprod!(i32, &'static str, bool);

        let co1 = I32StrBool::inject(3);
        let co2 = I32StrBool::inject("hello");
        let co3 = I32StrBool::inject(false);

        let uninject_i32_co1: Result<i32, _> = co1.uninject();
        let uninject_str_co1: Result<&'static str, _> = co1.uninject();
        let uninject_bool_co1: Result<bool, _> = co1.uninject();
        assert_eq!(uninject_i32_co1, Ok(3));
        assert!(uninject_str_co1.is_err());
        assert!(uninject_bool_co1.is_err());

        let uninject_i32_co2: Result<i32, _> = co2.uninject();
        let uninject_str_co2: Result<&'static str, _> = co2.uninject();
        let uninject_bool_co2: Result<bool, _> = co2.uninject();
        assert!(uninject_i32_co2.is_err());
        assert_eq!(uninject_str_co2, Ok("hello"));
        assert!(uninject_bool_co2.is_err());

        let uninject_i32_co3: Result<i32, _> = co3.uninject();
        let uninject_str_co3: Result<&'static str, _> = co3.uninject();
        let uninject_bool_co3: Result<bool, _> = co3.uninject();
        assert!(uninject_i32_co3.is_err());
        assert!(uninject_str_co3.is_err());
        assert_eq!(uninject_bool_co3, Ok(false));
    }
}
