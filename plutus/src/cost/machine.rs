use zerocopy::{FromBytes, Immutable, KnownLayout};

use crate::cost::function;

/// Cost parameters for the base machine (version `1.0.0`).
#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct Base {
    pub apply: Cost,
    pub builtin: Cost,
    pub konst: Cost,
    pub delay: Cost,
    pub force: Cost,
    pub lambda: Cost,
    pub startup: Cost,
    pub var: Cost,
}

/// Cost parameters for version `1.1.0`, with `constr` datatypes and `case`.
#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct Datatypes {
    pub constr: Cost,
    pub case: Cost,
}

type Cost = super::Pair<function::Constant, function::Constant>;
