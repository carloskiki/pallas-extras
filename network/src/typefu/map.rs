use std::marker::PhantomData;

use super::{
    Func, FuncOnce,
    coproduct::{CNil, Coproduct},
    hlist::{HCons, HNil},
};

/// A wrapper that allows a mapper to work over a Coproduct or HList, and generate a Coproduct.
pub struct CMap<T>(pub T);
/// A wrapper that allow a mapper to work over a HList or Coproduct, and generate a HList
pub struct HMap<T>(pub T);

/// A trait that works as a function signature at the type level
pub trait TypeMap<Input> {
    type Output;
}

impl<F> TypeMap<HNil> for HMap<F> {
    type Output = HNil;
}

impl<F> TypeMap<HNil> for CMap<F> {
    type Output = CNil;
}

impl<Head, Tail, F> TypeMap<HCons<Head, Tail>> for HMap<F>
where
    F: TypeMap<Head>,
    HMap<F>: TypeMap<Tail>,
{
    type Output = HCons<F::Output, <HMap<F> as TypeMap<Tail>>::Output>;
}

impl<Head, Tail, F> TypeMap<HCons<Head, Tail>> for CMap<F>
where
    F: TypeMap<Head>,
    CMap<F>: TypeMap<Tail>,
{
    type Output = Coproduct<F::Output, <CMap<F> as TypeMap<Tail>>::Output>;
}

impl<F> TypeMap<CNil> for HMap<F> {
    type Output = HNil;
}

impl<F> TypeMap<CNil> for CMap<F> {
    type Output = CNil;
}

impl<Head, Tail, F> TypeMap<Coproduct<Head, Tail>> for CMap<F>
where
    F: TypeMap<Head>,
    CMap<F>: TypeMap<Tail>,
{
    type Output = Coproduct<F::Output, <CMap<F> as TypeMap<Tail>>::Output>;
}

impl<Head, Tail, F> TypeMap<Coproduct<Head, Tail>> for HMap<F>
where
    F: TypeMap<Head>,
    HMap<F>: TypeMap<Tail>,
{
    type Output = HCons<F::Output, <HMap<F> as TypeMap<Tail>>::Output>;
}

impl<Head, Tail, F> FuncOnce<Coproduct<Head, Tail>> for CMap<F>
where
    F: FuncOnce<Head>,
    CMap<F>: FuncOnce<Tail>,
{
    #[inline]
    fn call_once(self, input: Coproduct<Head, Tail>) -> Self::Output {
        match input {
            Coproduct::Inl(head) => Coproduct::Inl(self.0.call_once(head)),
            Coproduct::Inr(tail) => Coproduct::Inr(self.call_once(tail)),
        }
    }
}

/// The Identity TypeMap; a signature that maps a type to its.
pub enum Identity {}
impl<T> TypeMap<T> for Identity {
    type Output = T;
}

pub struct Zip<T>(pub T);

impl<T> TypeMap<HNil> for Zip<T> {
    type Output = HNil;
}
impl<T> FuncOnce<HNil> for Zip<T> {
    #[inline]
    fn call_once(self, i: HNil) -> Self::Output {
        i
    }
}

impl<H1, H2, T1, T2> TypeMap<HCons<H1, T1>> for Zip<HCons<H2, T2>>
where
    Zip<T2>: TypeMap<T1>,
{
    type Output = HCons<(H1, H2), <Zip<T2> as TypeMap<T1>>::Output>;
}
impl<H1, H2, T1, T2> FuncOnce<HCons<H1, T1>> for Zip<HCons<H2, T2>>
where
    Zip<T2>: FuncOnce<T1>,
{
    #[inline]
    fn call_once(self, HCons { head, tail }: HCons<H1, T1>) -> Self::Output {
        HCons {
            head: (head, self.0.head),
            tail: Zip(self.0.tail).call_once(tail),
        }
    }
}

impl<T> TypeMap<CNil> for Zip<T> {
    type Output = CNil;
}
impl<T> FuncOnce<CNil> for Zip<T> {
    #[inline]
    fn call_once(self, i: CNil) -> Self::Output {
        i
    }
}

impl<H1, T1, H2, T2> TypeMap<Coproduct<H1, T1>> for Zip<HCons<H2, T2>>
where
    Zip<T2>: TypeMap<T1>,
{
    type Output = Coproduct<(H1, H2), <Zip<T2> as TypeMap<T1>>::Output>;
}
impl<H1, T1, H2, T2> FuncOnce<Coproduct<H1, T1>> for Zip<HCons<H2, T2>>
where
    Zip<T2>: FuncOnce<T1>,
{
    #[inline]
    fn call_once(self, input: Coproduct<H1, T1>) -> Self::Output {
        match input {
            Coproduct::Inl(head) => Coproduct::Inl((head, self.0.head)),
            Coproduct::Inr(tail) => Coproduct::Inr(Zip(self.0.tail).call_once(tail)),
        }
    }
}
