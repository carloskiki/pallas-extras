use super::{hlist::{HCons, HNil}, map::{HMap, TypeMap}};

pub trait Constructor<C>: TypeMap<C> {
    fn construct() -> Self::Output;
}

impl Constructor<HMap<HNil>> for HNil {
    fn construct() -> Self {
        HNil
    }
}

impl<Head, Tail, C> Constructor<HMap<C>> for HCons<Head, Tail>
where
    C: Constructor<Head>,
    Tail: Constructor<HMap<C>>
{
    fn construct() -> Self::Output {
        HCons {
            head: <C as Constructor<Head>>::construct(),
            tail: Tail::construct(),
        }
    }
}

