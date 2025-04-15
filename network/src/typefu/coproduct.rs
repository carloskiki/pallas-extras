//! Module that holds Coproduct data structures, traits, and implementations
//!
//! Think of "Coproduct" as ad-hoc enums; allowing you to do something like this
//!
//! ```
//! # fn main() {
//! # use create::typefu::Coprod;
//! // For simplicity, assign our Coproduct type to a type alias
//! // This is purely optional.
//! type I32Bool = Coprod!(i32, bool);
//! // Inject things into our Coproduct type
//! let co1 = I32Bool::inject(3);
//! let co2 = I32Bool::inject(true);
//!
//! // Getting stuff
//! let get_from_1a: Option<&i32> = co1.get();
//! let get_from_1b: Option<&bool> = co1.get();
//! assert_eq!(get_from_1a, Some(&3));
//! assert_eq!(get_from_1b, None);
//!
//! let get_from_2a: Option<&i32> = co2.get();
//! let get_from_2b: Option<&bool> = co2.get();
//! assert_eq!(get_from_2a, None);
//! assert_eq!(get_from_2b, Some(&true));
//!
//! // *Taking* stuff (by value)
//! let take_from_1a: Option<i32> = co1.take();
//! assert_eq!(take_from_1a, Some(3));
//!
//! // Or with a Result
//! let uninject_from_1a: Result<i32, _> = co1.uninject();
//! let uninject_from_1b: Result<bool, _> = co1.uninject();
//! assert_eq!(uninject_from_1a, Ok(3));
//! assert!(uninject_from_1b.is_err());
//! # }
//! ```
//!
//! Or, if you want to "fold" over all possible values of a coproduct
//!
//! ```
//! # use create::typefu::{hlist, poly_fn, Coprod};
//! # fn main() {
//! # type I32Bool = Coprod!(i32, bool);
//! # let co1 = I32Bool::inject(3);
//! # let co2 = I32Bool::inject(true);
//! // In the below, we use unreachable!() to make it obvious hat we know what type of
//! // item is inside our coproducts co1 and co2 but in real life, you should be writing
//! // complete functions for all the cases when folding coproducts
//! //
//! // to_ref borrows every item so that we can fold without consuming the coproduct.
//! assert_eq!(
//!     co1.to_ref().fold(hlist![|&i| format!("i32 {}", i),
//!                              |&b| unreachable!() /* we know this won't happen for co1 */ ]),
//!     "i32 3".to_string());
//! assert_eq!(
//!     co2.to_ref().fold(hlist![|&i| unreachable!() /* we know this won't happen for co2 */,
//!                              |&b| String::from(if b { "t" } else { "f" })]),
//!     "t".to_string());
//!
//! // Here, we use the poly_fn! macro to declare a polymorphic function to avoid caring
//! // about the order in which declare handlers for the types in our coproduct
//! let folded = co1.fold(
//!       poly_fn![
//!         |_b: bool| -> String { unreachable!() }, /* we know this won't happen for co1 */
//!         |i:  i32 | -> String { format!("i32 {}", i) },
//!       ]
//!      );
//! assert_eq!(folded, "i32 3".to_string());
//! # }
//! ```

use std::{ops::DerefMut, pin::{pin, Pin}};

use super::{
    Func, FuncOnce, Poly, PolyOnce, ToMut, ToRef,
    hlist::HCons,
    index::{Here, There},
    map::TypeMap,
};

/// Enum type representing a Coproduct. Think of this as a Result, but capable
/// of supporting any arbitrary number of types instead of just 2.
///
/// To construct a Coproduct, you would typically declare a type using the `Coprod!` type
/// macro and then use the `inject` method.
///
/// # Examples
///
/// ```
/// # fn main() {
/// use crate::typfu::Coprod;
///
/// type I32Bool = Coprod!(i32, bool);
/// let co1 = I32Bool::inject(3);
/// let get_from_1a: Option<&i32> = co1.get();
/// let get_from_1b: Option<&bool> = co1.get();
/// assert_eq!(get_from_1a, Some(&3));
/// assert_eq!(get_from_1b, None);
/// # }
/// ```
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
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() {
    /// use frunk::Coproduct;
    /// use frunk_core::Coprod;
    ///
    /// type I32F32 = Coprod!(i32, f32);
    ///
    /// // Constructing coproducts using inject:
    /// let co1_nice: I32F32 = Coproduct::inject(1i32);
    /// let co2_nice: I32F32 = Coproduct::inject(42f32);
    ///
    /// // Compare this to the "hard way":
    /// let co1_ugly: I32F32 = Coproduct::Inl(1i32);
    /// let co2_ugly: I32F32 = Coproduct::Inr(Coproduct::Inl(42f32));
    ///
    /// assert_eq!(co1_nice, co1_ugly);
    /// assert_eq!(co2_nice, co2_ugly);
    ///
    /// // Feel free to use `inject` on a type alias, or even directly on the
    /// // `Coprod!` macro. (the latter requires wrapping the type in `<>`)
    /// let _ = I32F32::inject(42f32);
    /// let _ = <Coprod!(i32, f32)>::inject(42f32);
    ///
    /// // You can also use a turbofish to specify the type of the input when
    /// // it is ambiguous (e.g. an empty `vec![]`).
    /// // The Index parameter should be left as `_`.
    /// type Vi32Vf32 = Coprod!(Vec<i32>, Vec<f32>);
    /// let _: Vi32Vf32 = Coproduct::inject::<Vec<i32>, _>(vec![]);
    /// # }
    /// ```
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
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # fn main() {
    /// use frunk_core::Coprod;
    ///
    /// type I32F32 = Coprod!(i32, f32);
    /// type I32 = Coprod!(i32); // remainder after uninjecting f32
    /// type F32 = Coprod!(f32); // remainder after uninjecting i32
    ///
    /// let co1 = I32F32::inject(42f32);
    ///
    /// // You can let type inference find the desired type.
    /// let co1 = I32F32::inject(42f32);
    /// let co1_as_i32: Result<i32, F32> = co1.uninject();
    /// let co1_as_f32: Result<f32, I32> = co1.uninject();
    /// assert_eq!(co1_as_i32, Err(F32::inject(42f32)));
    /// assert_eq!(co1_as_f32, Ok(42f32));
    ///
    /// // It is not necessary to annotate the type of the remainder:
    /// let res: Result<i32, _> = co1.uninject();
    /// assert!(res.is_err());
    ///
    /// // You can also use turbofish syntax to specify the type.
    /// // The Index parameter should be left as `_`.
    /// let co2 = I32F32::inject(1i32);
    /// assert_eq!(co2.uninject::<i32, _>(), Ok(1));
    /// assert_eq!(co2.uninject::<f32, _>(), Err(I32::inject(1)));
    /// # }
    /// ```
    ///
    /// Chaining calls for an exhaustive match:
    ///
    /// ```rust
    /// # fn main() {
    /// use frunk_core::Coprod;
    ///
    /// type I32F32 = Coprod!(i32, f32);
    ///
    /// // Be aware that this particular example could be
    /// // written far more succinctly using `fold`.
    /// fn handle_i32_f32(co: I32F32) -> &'static str {
    ///     // Remove i32 from the coproduct
    ///     let co = match co.uninject::<i32, _>() {
    ///         Ok(x) => return "integer!",
    ///         Err(co) => co,
    ///     };
    ///
    ///     // Remove f32 from the coproduct
    ///     let co = match co.uninject::<f32, _>() {
    ///         Ok(x) => return "float!",
    ///         Err(co) => co,
    ///     };
    ///
    ///     // Now co is empty
    ///     match co { /* unreachable */ }
    /// }
    ///
    /// assert_eq!(handle_i32_f32(I32F32::inject(3)), "integer!");
    /// assert_eq!(handle_i32_f32(I32F32::inject(3.0)), "float!");
    /// # }
    #[inline(always)]
    pub fn uninject<T, Index>(self) -> Result<T, <Self as CoprodUninjector<T, Index>>::Remainder>
    where
        Self: CoprodUninjector<T, Index>,
    {
        CoprodUninjector::uninject(self)
    }

    /// Borrow each variant of the Coproduct.
    ///
    /// # Example
    ///
    /// Composing with `subset` to match a subset of variants without
    /// consuming the coproduct:
    ///
    /// ```
    /// # fn main() {
    /// use frunk::Coproduct;
    /// use frunk_core::Coprod;
    ///
    /// let co: Coprod!(i32, bool, String) = Coproduct::inject(true);
    ///
    /// assert!(co.to_ref().subset::<Coprod!(&bool, &String), _>().is_ok());
    /// # }
    /// ```
    #[inline(always)]
    pub fn to_ref<'a>(&'a self) -> <Self as ToRef<'a>>::Output
    where
        Self: ToRef<'a>,
    {
        ToRef::to_ref(self)
    }

    /// Borrow each variant of the `Coproduct` mutably.
    ///
    /// # Example
    ///
    /// Composing with `subset` to match a subset of variants without
    /// consuming the coproduct:
    ///
    /// ```
    /// # fn main() {
    /// use frunk::Coproduct;
    /// use frunk_core::Coprod;
    ///
    /// let mut co: Coprod!(i32, bool, String) = Coproduct::inject(true);
    ///
    /// assert!(co.to_mut().subset::<Coprod!(&mut bool, &mut String), _>().is_ok());
    /// # }
    /// ```
    #[inline(always)]
    pub fn to_mut<'a>(&'a mut self) -> <Self as ToMut<'a>>::Output
    where
        Self: ToMut<'a>,
    {
        ToMut::to_mut(self)
    }

    /// Use functions to transform a Coproduct into a single value.
    ///
    /// A variety of types are supported for the `Folder` argument:
    ///
    /// * An `hlist![]` of closures (one for each type, in order).
    /// * A single closure (for a Coproduct that is homogenous).
    /// * A single [`Poly`].
    ///
    /// [`Poly`]: ../traits/struct.Poly.html
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() {
    /// use frunk_core::{Coprod, hlist};
    ///
    /// type I32F32Bool = Coprod!(i32, f32, bool);
    ///
    /// let co1 = I32F32Bool::inject(3);
    /// let co2 = I32F32Bool::inject(true);
    /// let co3 = I32F32Bool::inject(42f32);
    ///
    /// let folder = hlist![|&i| format!("int {}", i),
    ///                     |&f| format!("float {}", f),
    ///                     |&b| (if b { "t" } else { "f" }).to_string()];
    ///
    /// assert_eq!(co1.to_ref().fold(folder), "int 3".to_string());
    /// # }
    /// ```
    ///
    /// Using a polymorphic function type has the advantage of not
    /// forcing you to care about the order in which you declare
    /// handlers for the types in your Coproduct.
    ///
    /// ```
    /// # fn main() {
    /// use frunk::{Poly, Func};
    /// use frunk_core::Coprod;
    ///
    /// type I32F32Bool = Coprod!(i32, f32, bool);
    ///
    /// impl Func<i32> for P {
    ///     type Output = bool;
    ///     fn call(args: i32) -> Self::Output {
    ///         args > 100
    ///     }
    /// }
    /// impl Func<bool> for P {
    ///     type Output = bool;
    ///     fn call(args: bool) -> Self::Output {
    ///         args
    ///     }
    /// }
    /// impl Func<f32> for P {
    ///     type Output = bool;
    ///     fn call(args: f32) -> Self::Output {
    ///         args > 9000f32
    ///     }
    /// }
    /// struct P;
    ///
    /// let co1 = I32F32Bool::inject(3);
    /// let folded = co1.fold(Poly(P));
    /// # }
    /// ```
    #[inline(always)]
    pub fn fold<Output, Folder>(self, folder: Folder) -> Output
    where
        Self: CoproductFoldable<Folder, Output>,
    {
        CoproductFoldable::fold(self, folder)
    }

    /// Apply a function to each variant of a Coproduct.
    ///
    /// The transforms some `Coprod!(A, B, C, ..., E)` into some
    /// `Coprod!(T, U, V, ..., Z)`. A variety of types are supported for the
    /// mapper argument:
    ///
    /// * An `hlist![]` of closures (one for each variant).
    /// * A single closure (for mapping a Coproduct that is homogenous).
    /// * A single [`Poly`].
    ///
    /// # Examples
    ///
    /// ```
    /// use frunk::{hlist, Coprod};
    ///
    /// type I32F32Bool = Coprod!(i32, f32, bool);
    /// type BoolStrU8 = Coprod!(bool, &'static str, u8);
    ///
    /// let co1 = I32F32Bool::inject(3);
    /// let co2 = I32F32Bool::inject(42f32);
    /// let co3 = I32F32Bool::inject(true);
    ///
    /// let mapper = hlist![
    ///     |n| n > 0,
    ///     |f| if f == 42f32 { "😀" } else { "🤨" },
    ///     |b| if b { 1u8 } else { 0u8 },
    /// ];
    ///
    /// assert_eq!(co1.map(&mapper), BoolStrU8::inject(true));
    /// assert_eq!(co2.map(&mapper), BoolStrU8::inject("😀"));
    /// assert_eq!(co3.map(&mapper), BoolStrU8::inject(1u8));
    /// ```
    ///
    /// Using a polymorphic function type has the advantage of not forcing you
    /// to care about the order in which you declare handlers for the types in
    /// your Coproduct.
    ///
    /// ```
    /// use frunk::{poly_fn, Coprod};
    ///
    /// type I32F32Bool = Coprod!(i32, f32, bool);
    ///
    /// let co1 = I32F32Bool::inject(3);
    /// let co2 = I32F32Bool::inject(42f32);
    /// let co3 = I32F32Bool::inject(true);
    ///
    /// let mapper = poly_fn![
    ///     |b: bool| -> bool { !b },
    ///     |n: i32| -> i32 { n + 3 },
    ///     |f: f32| -> f32 { -f },
    /// ];
    ///
    /// assert_eq!(co1.map(&mapper), I32F32Bool::inject(6));
    /// assert_eq!(co2.map(&mapper), I32F32Bool::inject(-42f32));
    /// assert_eq!(co3.map(&mapper), I32F32Bool::inject(false));
    /// ```
    ///
    /// You can also use a singular closure if the Coproduct variants are all
    /// the same.
    ///
    /// ```
    /// use frunk::Coprod;
    ///
    /// type IntInt = Coprod!(i32, i32);
    /// type BoolBool = Coprod!(bool, bool);
    ///
    /// let mapper = |n| n > 0;
    ///
    /// let co = IntInt::Inl(42);
    /// assert_eq!(co.map(mapper), BoolBool::Inl(true));
    /// ```
    #[inline(always)]
    pub fn map<F>(self, mapper: F) -> <Self as CoproductMappable<F>>::Output
    where
        Self: CoproductMappable<F>,
    {
        CoproductMappable::map(self, mapper)
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
            Coproduct::Inl(l) => {
                Pin::new(l).poll(cx)
            },
            Coproduct::Inr(r) => {
                Pin::new(r).poll(cx)
            },
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

/// Trait for folding a coproduct into a single value.
///
/// This trait is part of the implementation of the inherent method
/// [`Coproduct::fold`]. Please see that method for more information.
///
/// You only need to import this trait when working with generic
/// Coproducts or Folders of unknown type. If the type of everything is known,
/// then `co.fold(folder)` should "just work" even without the trait.
///
/// [`Coproduct::fold`]: enum.Coproduct.html#method.fold
pub trait CoproductFoldable<Folder, Output> {
    /// Use functions to fold a coproduct into a single value.
    ///
    /// Please see the [inherent method] for more information.
    ///
    /// The only difference between that inherent method and this
    /// trait method is the location of the type parameters.
    /// (here, they are on the trait rather than the method)
    ///
    /// [inherent method]: enum.Coproduct.html#method.fold
    fn fold(self, f: Folder) -> Output;
}

impl<P, R, CH, CTail> CoproductFoldable<Poly<P>, R> for Coproduct<CH, CTail>
where
    P: Func<CH, Output = R>,
    CTail: CoproductFoldable<Poly<P>, R>,
{
    fn fold(self, f: Poly<P>) -> R {
        use self::Coproduct::*;
        match self {
            Inl(r) => P::call(r),
            Inr(rest) => rest.fold(f),
        }
    }
}

impl<F, R, FTail, CH, CTail> CoproductFoldable<HCons<F, FTail>, R> for Coproduct<CH, CTail>
where
    F: FnOnce(CH) -> R,
    CTail: CoproductFoldable<FTail, R>,
{
    fn fold(self, f: HCons<F, FTail>) -> R {
        use self::Coproduct::*;
        let f_head = f.head;
        let f_tail = f.tail;
        match self {
            Inl(r) => (f_head)(r),
            Inr(rest) => rest.fold(f_tail),
        }
    }
}

impl<F, CH, Tail> CoproductFoldable<PolyOnce<F>, F::Output> for Coproduct<CH, Tail>
where
    F: FuncOnce<CH>,
    Tail: CoproductFoldable<PolyOnce<F>, F::Output>,
{
    fn fold(self, PolyOnce(f): PolyOnce<F>) -> F::Output {
        match self {
            Coproduct::Inl(val) => f.call_once(val),
            Coproduct::Inr(tail) => tail.fold(PolyOnce(f)),
        }
    }
}

/// This is literally impossible; CNil is not instantiable
impl<F, R> CoproductFoldable<F, R> for CNil {
    fn fold(self, _: F) -> R {
        unreachable!()
    }
}

/// Trait for mapping over a coproduct's variants.
///
/// This trait is part of the implementation of the inherent method
/// [`Coproduct::map`]. Please see that method for more information.
///
/// You only need to import this trait when working with generic Coproducts or
/// mappers of unknown type. If the type of everything is known, then
/// `co.map(mapper)` should "just work" even without the trait.
pub trait CoproductMappable<Mapper> {
    type Output;

    /// Use functions to map each variant of a coproduct.
    ///
    /// Please see the [inherent method] for more information.
    ///
    /// The only difference between that inherent method and this
    /// trait method is the location of the type parameters.
    /// (here, they are on the trait rather than the method)
    ///
    /// [inherent method]: Coproduct::map
    fn map(self, f: Mapper) -> Self::Output;
}

/// Implementation for mapping a Coproduct using an `hlist!`.
impl<F, R, MapperTail, CH, CTail> CoproductMappable<HCons<F, MapperTail>> for Coproduct<CH, CTail>
where
    F: FnOnce(CH) -> R,
    CTail: CoproductMappable<MapperTail>,
{
    type Output = Coproduct<R, <CTail as CoproductMappable<MapperTail>>::Output>;

    #[inline]
    fn map(self, mapper: HCons<F, MapperTail>) -> Self::Output {
        match self {
            Coproduct::Inl(l) => Coproduct::Inl((mapper.head)(l)),
            Coproduct::Inr(rest) => Coproduct::Inr(rest.map(mapper.tail)),
        }
    }
}

/// Implementation for mapping a Coproduct using a `&hlist!`.
impl<'a, F, R, MapperTail, CH, CTail> CoproductMappable<&'a HCons<F, MapperTail>>
    for Coproduct<CH, CTail>
where
    F: Fn(CH) -> R,
    CTail: CoproductMappable<&'a MapperTail>,
{
    type Output = Coproduct<R, <CTail as CoproductMappable<&'a MapperTail>>::Output>;

    #[inline]
    fn map(self, mapper: &'a HCons<F, MapperTail>) -> Self::Output {
        match self {
            Coproduct::Inl(l) => Coproduct::Inl((mapper.head)(l)),
            Coproduct::Inr(rest) => Coproduct::Inr(rest.map(&mapper.tail)),
        }
    }
}

/// Implementation for mapping a Coproduct using a `&mut hlist!`.
impl<'a, F, R, MapperTail, CH, CTail> CoproductMappable<&'a mut HCons<F, MapperTail>>
    for Coproduct<CH, CTail>
where
    F: FnMut(CH) -> R,
    CTail: CoproductMappable<&'a mut MapperTail>,
{
    type Output = Coproduct<R, <CTail as CoproductMappable<&'a mut MapperTail>>::Output>;

    #[inline]
    fn map(self, mapper: &'a mut HCons<F, MapperTail>) -> Self::Output {
        match self {
            Coproduct::Inl(l) => Coproduct::Inl((mapper.head)(l)),
            Coproduct::Inr(rest) => Coproduct::Inr(rest.map(&mut mapper.tail)),
        }
    }
}

/// Implementation for mapping a Coproduct using a `poly_fn!`.
impl<P, CH, CTail> CoproductMappable<Poly<P>> for Coproduct<CH, CTail>
where
    P: Func<CH>,
    CTail: CoproductMappable<Poly<P>>,
{
    type Output =
        Coproduct<<P as TypeMap<CH>>::Output, <CTail as CoproductMappable<Poly<P>>>::Output>;

    #[inline]
    fn map(self, poly: Poly<P>) -> Self::Output {
        match self {
            Coproduct::Inl(l) => Coproduct::Inl(P::call(l)),
            Coproduct::Inr(rest) => Coproduct::Inr(rest.map(poly)),
        }
    }
}

/// Implementation for mapping a Coproduct using a `&poly_fn!`.
impl<'a, P, CH, CTail> CoproductMappable<&'a Poly<P>> for Coproduct<CH, CTail>
where
    P: Func<CH>,
    CTail: CoproductMappable<&'a Poly<P>>,
{
    type Output =
        Coproduct<<P as TypeMap<CH>>::Output, <CTail as CoproductMappable<&'a Poly<P>>>::Output>;

    #[inline]
    fn map(self, poly: &'a Poly<P>) -> Self::Output {
        match self {
            Coproduct::Inl(l) => Coproduct::Inl(P::call(l)),
            Coproduct::Inr(rest) => Coproduct::Inr(rest.map(poly)),
        }
    }
}

/// Implementation for mapping a Coproduct using a `&mut poly_fn!`.
impl<'a, P, CH, CTail> CoproductMappable<&'a mut Poly<P>> for Coproduct<CH, CTail>
where
    P: Func<CH>,
    CTail: CoproductMappable<&'a mut Poly<P>>,
{
    type Output = Coproduct<
        <P as TypeMap<CH>>::Output,
        <CTail as CoproductMappable<&'a mut Poly<P>>>::Output,
    >;

    #[inline]
    fn map(self, poly: &'a mut Poly<P>) -> Self::Output {
        match self {
            Coproduct::Inl(l) => Coproduct::Inl(P::call(l)),
            Coproduct::Inr(rest) => Coproduct::Inr(rest.map(poly)),
        }
    }
}

/// Implementation for mapping a Coproduct using a single function that can
/// handle all variants.
impl<F, R, CH, CTail> CoproductMappable<F> for Coproduct<CH, CTail>
where
    F: FnMut(CH) -> R,
    CTail: CoproductMappable<F>,
{
    type Output = Coproduct<R, <CTail as CoproductMappable<F>>::Output>;

    #[inline]
    fn map(self, mut f: F) -> Self::Output {
        match self {
            Coproduct::Inl(l) => Coproduct::Inl(f(l)),
            Coproduct::Inr(rest) => Coproduct::Inr(rest.map(f)),
        }
    }
}

/// Base case map impl.
impl<F> CoproductMappable<F> for CNil {
    type Output = CNil;

    #[inline(always)]
    fn map(self, _: F) -> Self::Output {
        match self {}
    }
}

pub struct Overwrite<T>(pub T);

impl<T, Tail, C, CTail> CoproductMappable<Overwrite<HCons<T, Tail>>> for Coproduct<C, CTail>
where
    CTail: CoproductMappable<Overwrite<Tail>>,
{
    type Output = Coproduct<T, CTail::Output>;

    #[inline]
    fn map(self, Overwrite(HCons { head, tail }): Overwrite<HCons<T, Tail>>) -> Self::Output {
        match self {
            Coproduct::Inl(_) => Coproduct::Inl(head),
            Coproduct::Inr(ctail) => Coproduct::Inr(ctail.map(Overwrite(tail))),
        }
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

macro_rules! comatch {
    ($value:expr; _: $t:ty => $e:expr, $($tail:tt)*) => {
        match $value.uninject::<$t, _>() {
            Ok(_) => $e,
            Err(co) => comatch!(co; $($tail)*)
        }
    };
    ($value:expr; $var:ident: $t:ty => $e:expr, $($tail:tt)*) => {
        match $value.uninject::<$t, _>() {
            Ok($var) => $e,
            Err(co) => comatch!(co; $($tail)*)
        }
    };
    ($value:expr; $p:pat => $e:expr, $($tail:tt)*) => {
        match $value.uninject() {
            Ok($p) => $e,
            Err(co) => comacth!(co; $($tail)*)
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
///
/// # Examples
///
/// ```
/// # fn main() {
/// use frunk_core::Coprod;
///
/// type I32Bool = Coprod!(i32, bool);
/// let co1 = I32Bool::inject(3);
///
/// // Use ...Tail to append another coproduct at the end.
/// let co2 = <Coprod!(&str, String, ...I32Bool)>::inject(3);
/// # }
/// ```
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

    use crate::typefu::hlist::hlist;

    use super::Coproduct::*;
    use super::*;

    use std::format;
    use std::string::{String, ToString};

    #[test]
    fn test_coproduct_inject() {
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
    fn test_coproduct_fold_consuming() {
        type I32F32StrBool = Coprod!(i32, f32, bool);

        let co1 = I32F32StrBool::inject(3);
        let folded = co1.fold(hlist![
            |i| format!("int {}", i),
            |f| format!("float {}", f),
            |b| (if b { "t" } else { "f" }).to_string(),
        ]);

        assert_eq!(folded, "int 3".to_string());
    }

    #[test]
    fn test_coproduct_poly_fold_consuming() {
        type I32F32StrBool = Coprod!(i32, f32, bool);

        impl<T> TypeMap<T> for P {
            type Output = bool;
        }
        impl Func<i32> for P {
            fn call(args: i32) -> Self::Output {
                args > 100
            }
        }
        impl Func<bool> for P {
            fn call(args: bool) -> Self::Output {
                args
            }
        }
        impl Func<f32> for P {
            fn call(args: f32) -> Self::Output {
                args > 9000f32
            }
        }
        struct P;

        let co1 = I32F32StrBool::inject(3);
        let folded = co1.fold(Poly(P));

        assert!(!folded);
    }

    #[test]
    fn test_coproduct_fold_non_consuming() {
        type I32F32Bool = Coprod!(i32, f32, bool);

        let co1 = I32F32Bool::inject(3);
        let co2 = I32F32Bool::inject(true);
        let co3 = I32F32Bool::inject(42f32);

        assert_eq!(
            co1.to_ref().fold(hlist![
                |&i| format!("int {}", i),
                |&f| format!("float {}", f),
                |&b| (if b { "t" } else { "f" }).to_string(),
            ]),
            "int 3".to_string()
        );
        assert_eq!(
            co2.to_ref().fold(hlist![
                |&i| format!("int {}", i),
                |&f| format!("float {}", f),
                |&b| (if b { "t" } else { "f" }).to_string(),
            ]),
            "t".to_string()
        );
        assert_eq!(
            co3.to_ref().fold(hlist![
                |&i| format!("int {}", i),
                |&f| format!("float {}", f),
                |&b| (if b { "t" } else { "f" }).to_string(),
            ]),
            "float 42".to_string()
        );
    }

    #[test]
    fn test_coproduct_uninject() {
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

    #[test]
    fn test_coproduct_map_ref() {
        type I32Bool = Coprod!(i32, bool);
        type I32BoolRef<'a> = Coprod!(i32, &'a bool);

        fn map_it(co: &I32Bool) -> I32BoolRef<'_> {
            // For some reason rustc complains about lifetimes if you try to
            // inline the closure literal into the hlist 🤷.
            let map_bool: fn(&bool) -> &bool = |b| b;

            let mapper = hlist![|n: &i32| *n + 3, map_bool];

            co.to_ref().map(mapper)
        }

        let co = I32Bool::inject(3);
        let new = map_it(&co);
        assert_eq!(new, I32BoolRef::inject(6))
    }

    #[test]
    fn test_coproduct_map_with_ref_mapper() {
        type I32Bool = Coprod!(i32, bool);

        // HList mapper

        let mapper = hlist![|n| n + 3, |b: bool| !b];

        let co = I32Bool::inject(3);
        let co = co.map(&mapper);
        let co = co.map(&mapper);

        assert_eq!(co, I32Bool::inject(9));

        // Fn mapper

        type StrStr = Coprod!(String, String);

        let captured = String::from("!");
        let mapper = |s: String| format!("{}{}", s, &captured);

        let co = StrStr::Inl(String::from("hi"));
        let co = co.map(&mapper);
        let co = co.map(&mapper);

        assert_eq!(co, StrStr::Inl(String::from("hi!!")));
    }

    #[test]
    fn test_coproduct_map_with_mut_mapper() {
        type I32Bool = Coprod!(i32, bool);

        // HList mapper

        let mut number = None;
        let mut boolean = None;

        let mut mapper = hlist![
            |n: i32| {
                number = Some(n);
                n
            },
            |b: bool| {
                boolean = Some(b);
                b
            },
        ];

        let co = I32Bool::inject(3);
        let co = co.map(&mut mapper);
        assert_eq!(co, I32Bool::inject(3));
        assert_eq!(number, Some(3));
        assert_eq!(boolean, None);

        // Fn mapper

        type StrStr = Coprod!(String, String);

        let mut captured = String::new();
        let mut mapper = |s: String| {
            let s = format!("{s}!");
            captured.push_str(&s);
            s
        };

        let co = StrStr::Inl(String::from("hi"));
        let co = co.map(&mut mapper);
        let co = co.map(&mut mapper);

        assert_eq!(co, StrStr::Inl(String::from("hi!!")));
        assert_eq!(captured, String::from("hi!hi!!"));
    }
}
