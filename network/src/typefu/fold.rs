use std::marker::PhantomData;

use super::{coproduct::{CNil, Coproduct}, map::TypeMap, FuncOnce};

pub struct Fold<F, O>(pub F, pub PhantomData<O>);

impl<F, O> TypeMap<CNil> for Fold<F, O> {
    type Output = O;
}

impl<F, O, Head, Tail> TypeMap<Coproduct<Head, Tail>> for Fold<F, O>
where
    F: TypeMap<Head, Output = O>,
    Fold<F, O>: TypeMap<Tail>,
{
    type Output = O;
}

impl<F, O> FuncOnce<CNil> for Fold<F, O> {
    fn call_once(self, input: CNil) -> Self::Output {
        match input {}
    }
}

impl<F, O, Head, Tail> FuncOnce<Coproduct<Head, Tail>> for Fold<F, O>
where
    F: FuncOnce<Head, Output = O>,
    Fold<F, O>: FuncOnce<Tail, Output = O>,
{
    fn call_once(self, input: Coproduct<Head, Tail>) -> Self::Output {
        match input {
            Coproduct::Inl(head) => self.0.call_once(head),
            Coproduct::Inr(tail) => self.call_once(tail),
        }
    }
}
