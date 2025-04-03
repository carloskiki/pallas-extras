use super::{hlist::{HCons, HNil}, map::{HMap, TypeMap}};

pub trait Constructor<C>: TypeMap<C> {
    fn construct() -> Self::Output;
}

impl Constructor<HNil> for HMap<HNil> {
    #[inline]
    fn construct() -> Self::Output {
        HNil
    }
}

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

