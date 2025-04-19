use crate::typefu::{coproduct::{CNil, Coproduct}, hlist::HCons, FuncOnce};

use super::TypeMap;

pub struct Overwrite<T>(pub T);

impl<T> TypeMap<&CNil> for Overwrite<T> {
    type Output = CNil;
}

impl<T> FuncOnce<&CNil> for Overwrite<T> {
    #[inline]
    fn call_once(self, input: &CNil) -> Self::Output {
        match *input {}
    }
}

impl<T> TypeMap<CNil> for Overwrite<T> {
    type Output = CNil;
}

impl<T> FuncOnce<CNil> for Overwrite<T> {
    #[inline]
    fn call_once(self, input: CNil) -> Self::Output {
        match input {}
    }
}

impl<'a, T, Tail, C, CTail> TypeMap<&'a Coproduct<C, CTail>> for Overwrite<HCons<T, Tail>>
where
    Overwrite<Tail>: TypeMap<&'a CTail>,
{
    type Output = Coproduct<T, <Overwrite<Tail> as TypeMap<&'a CTail>>::Output>;
}

impl<'a, T, Tail, C, CTail> FuncOnce<&'a Coproduct<C, CTail>> for Overwrite<HCons<T, Tail>>
where
    Overwrite<Tail>: FuncOnce<&'a CTail>,
{
    #[inline]
    fn call_once(self, co: &'a Coproduct<C, CTail>) -> Self::Output {
        match co {
            Coproduct::Inl(_) => Coproduct::Inl(self.0.head),
            Coproduct::Inr(ctail) => Coproduct::Inr(Overwrite(self.0.tail).call_once(ctail)),
        }
    }
}

impl<T, Tail, C, CTail> TypeMap<Coproduct<C, CTail>> for Overwrite<HCons<T, Tail>>
where
    Overwrite<Tail>: TypeMap<CTail>,
{
    type Output = Coproduct<T, <Overwrite<Tail> as TypeMap<CTail>>::Output>;
}

impl<T, Tail, C, CTail> FuncOnce<Coproduct<C, CTail>> for Overwrite<HCons<T, Tail>>
where
    Overwrite<Tail>: FuncOnce<CTail>,
{
    #[inline]
    fn call_once(self, co: Coproduct<C, CTail>) -> Self::Output {
        match co {
            Coproduct::Inl(_) => Coproduct::Inl(self.0.head),
            Coproduct::Inr(ctail) => Coproduct::Inr(Overwrite(self.0.tail).call_once(ctail)),
        }
    }
}
