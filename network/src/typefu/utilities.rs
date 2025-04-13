use super::{
    hlist::{HCons, HNil},
    map::TypeMap, Func,
};

/// Unzip a [`hlist`](super::hlist) of tuples into two separate `hlist`s.
///
/// ## Example
/// ```
/// use crate::typefu::{hlist::{hlist, hlist_pat}, Func, utilities::Unzip};
/// 
/// let list = hlist![(32, "test"), ("hello", 0.5) , ((2..5), vec![1, 2, 3])];
/// let (hlist_pat![num, string, range], right_list) = Unzip::call(list);
/// 
/// assert_eq!(num, 32);
/// assert_eq!(string, "hello");
/// assert_eq!(range, 2..5);
/// assert_eq!(right_list, hlist!["test", 0.5, vec![1, 2, 3]]);
/// ````
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
