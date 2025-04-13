use super::{coproduct::Coproduct, hlist::{HCons, HNil}, map::{HMap, TypeMap}, Func};

/// Given an input type, generate a value of some output type.
///
/// This is useful to extract the associated constant of a trait, for example.
pub trait Constructor<C>: TypeMap<C> {
    fn construct() -> Self::Output;
}

impl<C> Constructor<HNil> for HMap<C> {
    #[inline]
    fn construct() -> Self::Output {
        HNil
    }
}

/// Given a `HList` of constructors, construct an `HList` of the type they construct.
impl<Head, Tail, C> Constructor<HCons<Head, Tail>> for HMap<C>
where
    C: Constructor<Head>,
    HMap<C>: Constructor<Tail>,
{
    #[inline]
    fn construct() -> Self::Output {
        HCons {
            head: <C as Constructor<Head>>::construct(),
            tail: <HMap<C> as Constructor<Tail>>::construct(),
        }
    }
}

/// Given a `Coproduct` of constructors, construct an `HList` of the type they construct.
impl<Head, Tail, C> Constructor<Coproduct<Head, Tail>> for HMap<C>
where
    C: Constructor<Head>,
    HMap<C>: Constructor<Tail>,
{
    #[inline]
    fn construct() -> Self::Output {
        HCons {
            head: <C as Constructor<Head>>::construct(),
            tail: <HMap<C> as Constructor<Tail>>::construct(),
        }
    }
}
