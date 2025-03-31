//! Functional programming utilities that should likely be upstreamed to `frunk`.

use frunk::{
    coproduct::{CNil, CoproductFoldable}, Coproduct, Func, HCons, HNil
};

macro_rules! comatch {
    ($value:expr; $($tail:tt)*) => {
        comatch_recurse!($value; $($tail)*)
    };
}

macro_rules! comatch_recurse {
    ($value:expr; _: $t:ty => $e:expr, $($tail:tt)*) => {
        match $value.uninject::<$t, _>() {
            Ok(_) => $e,
            Err(co) => comatch_recurse!(co; $($tail)*)
        }
    };
    ($value:expr; $var:ident: $t:ty => $e:expr, $($tail:tt)*) => {
        match $value.uninject::<$t, _>() {
            Ok($var) => $e,
            Err(co) => comatch_recurse!(co; $($tail)*)
        }
    };
    ($value:expr; $p:pat => $e:expr, $($tail:tt)*) => {
        match $value.uninject() {
            Ok($p) => $e,
            Err(co) => comatch_recurse!(co; $($tail)*)
        }
    };
    ($value:expr;) => {
        match $value {}
    }
}

pub trait FuncOnce<Input> {
    type Output;

    fn call(self, input: Input) -> Self::Output;
}

pub struct PolyOnce<T>(pub T);

impl<F, CH, Tail> CoproductFoldable<PolyOnce<F>, F::Output> for Coproduct<CH, Tail>
where
    F: FuncOnce<CH>,
    Tail: CoproductFoldable<PolyOnce<F>, F::Output>,
{
    fn fold(self, PolyOnce(f): PolyOnce<F>) -> F::Output {
        match self {
            Coproduct::Inl(val) => f.call(val),
            Coproduct::Inr(tail) => tail.fold(PolyOnce(f)),
        }
    }
}

pub struct HMap<T>(pub T);
pub struct CMap<T>(pub T);

pub trait TypeMap<Mapper> {
    type Output;
}

impl<F> TypeMap<HMap<F>> for HNil {
    type Output = HNil;
}

impl<F> TypeMap<CMap<F>> for HNil {
    type Output = CNil;
}

impl<Head, Tail, F> TypeMap<HMap<F>> for HCons<Head, Tail>
where
    F: TypeMap<Head>,
    Tail: TypeMap<HMap<F>>,
{
    type Output = HCons<F::Output, Tail::Output>;
}

impl<Head, Tail, F> TypeMap<CMap<F>> for HCons<Head, Tail>
where
    F: TypeMap<Head>,
    Tail: TypeMap<CMap<F>>,
{
    type Output = Coproduct<F::Output, Tail::Output>;
}

impl<F> TypeMap<HMap<F>> for CNil {
    type Output = HNil;
}

impl<F> TypeMap<CMap<F>> for CNil {
    type Output = CNil;
}

impl<Head, Tail, F> TypeMap<CMap<F>> for Coproduct<Head, Tail>
where
    F: TypeMap<Head>,
    Tail: TypeMap<CMap<F>>,
{
    type Output = Coproduct<F::Output, Tail::Output>;
}

impl<Head, Tail, F> TypeMap<HMap<F>> for Coproduct<Head, Tail>
where
    F: TypeMap<Head>,
    Tail: TypeMap<HMap<F>>,
{
    type Output = HCons<F::Output, Tail::Output>;
}

pub type ProtocolMessage<P> = <P as TypeMap<type_maps::MiniProtocolMessage>>::Output;
pub type MiniProtocolMessage<MP> = <MP as TypeMap<type_maps::StateMessage>>::Output;

pub mod type_maps {
    use crate::protocol::{MiniProtocol, State};

    use super::TypeMap;

    pub enum MiniProtocolMessage {}
    impl<MP> TypeMap<MP> for MiniProtocolMessage
    where
        MP: MiniProtocol,
    {
        type Output = super::MiniProtocolMessage<MP>;
    }

    pub enum StateMessage {}
    impl<S> TypeMap<S> for StateMessage
    where
        S: State,
    {
        type Output = S::Message;
    }
}

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

pub struct CoprodZipHList<H, C>(pub H, pub C);

impl<'a, CHead, CTail, Tail, Head, C> Func<&'a Coproduct<CHead, CTail>> for CoprodZipHList<HCons<Head, Tail>, C>
where
    C: Constructor<Head>,
    CoprodZipHList<Tail, C>: Func<&'a CTail, Output = C::Output>,
{
    type Output = C::Output;

    fn call(i: &'a Coproduct<CHead, CTail>) -> Self::Output {
        match i {
            Coproduct::Inl(_) => C::construct(),
            Coproduct::Inr(r) => CoprodZipHList::<Tail, C>::call(r),
        }
    }
}
