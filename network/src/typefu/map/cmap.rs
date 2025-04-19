use crate::typefu::{coproduct::{CNil, Coproduct}, hlist::{HCons, HNil}, FuncOnce};

use super::TypeMap;

/// A wrapper that allows a mapper to work over a Coproduct or HList, and generate a Coproduct.
pub struct CMap<T>(pub T);

impl<F> TypeMap<HNil> for CMap<F> {
    type Output = CNil;
}

impl<Head, Tail, F> TypeMap<HCons<Head, Tail>> for CMap<F>
where
    F: TypeMap<Head>,
    CMap<F>: TypeMap<Tail>,
{
    type Output = Coproduct<F::Output, <CMap<F> as TypeMap<Tail>>::Output>;
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

impl<F> FuncOnce<CNil> for CMap<F> {
    #[inline]
    fn call_once(self, i: CNil) -> Self::Output {
        match i {}
    }
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

