use super::{coproduct::{CNil, Coproduct}, hlist::{HCons, HNil}};

pub struct CMap<T>(pub T);
pub struct HMap<T>(pub T);

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


pub enum Identity {}
impl<T> TypeMap<T> for Identity {
    type Output = T;
}
