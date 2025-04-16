use crate::typefu::{coproduct::{CNil, Coproduct}, hlist::{HCons, HNil}};

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
