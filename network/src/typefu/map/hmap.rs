use crate::typefu::{coproduct::{CNil, Coproduct}, hlist::{HCons, HNil}, Func, FuncMany};

use super::TypeMap;

/// A wrapper that allow a mapper to work over a HList or Coproduct, and generate a HList
pub struct HMap<T>(pub T);

impl<F> TypeMap<HNil> for HMap<F> {
    type Output = HNil;
}

impl<Head, Tail, F> TypeMap<HCons<Head, Tail>> for HMap<F>
where
    F: TypeMap<Head>,
    HMap<F>: TypeMap<Tail>,
{
    type Output = HCons<F::Output, <HMap<F> as TypeMap<Tail>>::Output>;
}

impl<F> TypeMap<CNil> for HMap<F> {
    type Output = HNil;
}

impl<Head, Tail, F> TypeMap<Coproduct<Head, Tail>> for HMap<F>
where
    F: TypeMap<Head>,
    HMap<F>: TypeMap<Tail>,
{
    type Output = HCons<F::Output, <HMap<F> as TypeMap<Tail>>::Output>;
}

impl<F> FuncMany<HNil> for HMap<F> {
    fn call_many(&self, _: HNil) -> Self::Output {
        HNil
    }
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

impl<F> Func<HNil> for HMap<F> {
    fn call(i: HNil) -> Self::Output {
        i
    }
}

impl<H, Tail, F> Func<HCons<H, Tail>> for HMap<F>
where
    F: Func<H>,
    HMap<F>: Func<Tail>,
{
    fn call(input: HCons<H, Tail>) -> Self::Output {
        HCons {
            head: F::call(input.head),
            tail: Self::call(input.tail),
        }
    }
}
