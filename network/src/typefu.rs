// The MIT License (MIT)
// 
// Copyright (c) 2016 by Lloyd Chan
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

//! Functional programming utilities mostly coming from `frunk`.

use hlist::HCons;
use map::{HMap, TypeMap};

pub mod coproduct;
pub mod hlist;
pub mod index;
pub mod map;
pub mod constructor;
pub mod utilities;

/// An alternative to AsRef that does not force the reference type to be a pointer itself.
///
/// This lets us create implementations for our recursive traits that take the resulting
/// Output reference type, without having to deal with strange, spurious overflows
/// that sometimes occur when trying to implement a trait for &'a T (see this comment:
/// <https://github.com/lloydmeta/frunk/pull/106#issuecomment-377927198>)
///
/// This functionality is also provided as an inherent method [on HLists] and [on
/// Coproducts](coproduct::Coproduct::to_ref).
/// However, you may find this trait useful in generic contexts.
///
/// [on HLists]: ../hlist/struct.HCons.html#method.to_ref
pub trait ToRef<'a> {
    type Output;

    fn to_ref(&'a self) -> Self::Output;
}

/// An alternative to `AsMut` that does not force the reference type to be a pointer itself.
///
/// This parallels [`ToRef`]; see it for more information.
///
/// [`ToRef`]: trait.ToRef.html
pub trait ToMut<'a> {
    type Output;

    fn to_mut(&'a mut self) -> Self::Output;
}

pub trait FuncOnce<Input>: TypeMap<Input> {
    fn call_once(self, input: Input) -> Self::Output;
}

// Same as [`Poly`], but for [`FuncOnce`].
pub struct PolyOnce<T>(pub T);

/// Wrapper type around a function for polymorphic maps and folds.
///
/// This is a thin generic wrapper type that is used to differentiate
/// between single-typed generic closures `F` that implements, say, `Fn(i8) -> bool`,
/// and a Poly-typed `F` that implements multiple Function types
/// via the [`Func`] trait. (say, `Func<i8, Output=bool>` and `Func<bool, Output=f32>`)
///
/// This is needed because there are completely generic impls for many of the
/// HList traits that take a simple unwrapped closure.
///
/// [`Func`]: trait.Func.html
#[derive(Debug, Copy, Clone, Default)]
pub struct Poly<T>(pub T);

/// This is a simple, user-implementable alternative to `Fn`.
///
/// Might not be necessary if/when Fn(Once, Mut) traits are implementable
/// in stable Rust
pub trait Func<Input>: TypeMap<Input> {
    /// Call the `Func`.
    ///
    /// Notice that this does not take a self argument, which in turn means `Func`
    /// cannot effectively close over a context. This decision trades power for convenience;
    /// a three-trait `Fn` heirarchy like that in std provides a great deal of power in a
    /// small fraction of use-cases, but it also comes at great expanse to the other 95% of
    /// use cases.
    fn call(i: Input) -> Self::Output;
}

pub trait FuncMany<Input>: TypeMap<Input> {
    fn call_many(&self, input: Input) -> Self::Output;
}

impl<H, Tail, F> FuncMany<HCons<H, Tail>> for HMap<F>
where
    F: FuncMany<H>,
    HMap<F>: FuncMany<Tail>,
{
    fn call_many(&self, input: HCons<H, Tail>) -> Self::Output {
        HCons {
            head: self.0.call_many(input.head),
            tail: self.call_many(input.tail),
        }
    }
}
