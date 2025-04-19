use crate::typefu::{
    hlist::{HCons, HNil},
    map::TypeMap, Func,
};

pub enum Unzip {}
pub enum UnzipLeft {}
pub enum UnzipRight {}

impl<T, U, Tail> TypeMap<HCons<(T, U), Tail>> for UnzipLeft
where
    UnzipLeft: TypeMap<Tail>,
{
    type Output = HCons<T, <UnzipLeft as TypeMap<Tail>>::Output>;
}

impl TypeMap<HNil> for UnzipLeft {
    type Output = HNil;
}


impl<T, U, Tail> TypeMap<HCons<(T, U), Tail>> for UnzipRight
where
    UnzipRight: TypeMap<Tail>,
{
    type Output = HCons<U, <UnzipRight as TypeMap<Tail>>::Output>;
}

impl TypeMap<HNil> for UnzipRight {
    type Output = HNil;
}

impl<T> TypeMap<T> for Unzip
where
    UnzipLeft: TypeMap<T>,
    UnzipRight: TypeMap<T>,
{
    type Output = (
        <UnzipLeft as TypeMap<T>>::Output,
        <UnzipRight as TypeMap<T>>::Output,
    );
}

impl<T, U, Tail> Func<HCons<(T, U), Tail>> for Unzip
where
    Unzip: Func<Tail, Output = (
        <UnzipLeft as TypeMap<Tail>>::Output,
        <UnzipRight as TypeMap<Tail>>::Output,
    )>,
    UnzipLeft: TypeMap<Tail>,
    UnzipRight: TypeMap<Tail>,
{
    fn call(i: HCons<(T, U), Tail>) -> Self::Output {
        let (left, right) = Unzip::call(i.tail);
        let left = HCons {
            head: i.head.0,
            tail: left,
        };
        let right = HCons {
            head: i.head.1,
            tail: right,
        };
        (left, right)
    }
}

impl Func<HNil> for Unzip {
    fn call(_: HNil) -> Self::Output {
        (HNil, HNil)
    }
}
