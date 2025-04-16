use crate::typefu::{coproduct::{CNil, Coproduct}, hlist::{HCons, HNil}, FuncOnce};

use super::TypeMap;

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

