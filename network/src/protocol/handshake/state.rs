use std::{fmt::Debug, time::Duration};

use minicbor::Encode;

use crate::traits::state::{Client, State};

use super::{
    Coprod,
    message::{AcceptVersion, ProposeVersions, QueryReply, Refuse},
};

#[derive(PartialOrd, Ord, Hash)]
pub struct Propose<VD>(pub(crate) std::marker::PhantomData<VD>);

impl<T> Debug for Propose<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Propose").field(&self.0).finish()
    }
}

impl<VD> Default for Propose<VD> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> Clone for Propose<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Propose<T> {}

impl<T> PartialEq for Propose<T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<T> Eq for Propose<T> {}

impl<VD> State for Propose<VD>
where
    VD: Encode<()> + for<'a> minicbor::Decode<'a, ()> + 'static,
{
    const TIMEOUT: Duration = Duration::from_secs(10);
    type Agency = Client;

    type Message = Coprod![ProposeVersions<VD>];
}

#[derive(PartialOrd, Ord, Hash)]
pub struct Confirm<VD>(std::marker::PhantomData<VD>);

impl<T> Debug for Confirm<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Confirm").field(&self.0).finish()
    }
}

impl<VD> Default for Confirm<VD> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> Clone for Confirm<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Confirm<T> {}

impl<T> PartialEq for Confirm<T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<T> Eq for Confirm<T> {}

impl<VD> State for Confirm<VD>
where
    VD: Encode<()> + for<'a> minicbor::Decode<'a, ()> + 'static,
{
    const TIMEOUT: std::time::Duration = Duration::from_secs(10);
    type Agency = Client;

    type Message = Coprod![AcceptVersion<VD>, Refuse<'static>, QueryReply<VD>];
}
