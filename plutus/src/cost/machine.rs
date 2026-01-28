use zerocopy::{FromBytes, Immutable, KnownLayout};

use crate::cost::function;

pub const BASE_INDEX: usize = 17;
pub const DATATYPES_INDEX: usize = 193;

/// Cost parameters for the base machine (version `1.0.0`).
#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct Base {
    pub application: Cost,
    pub builtin: Cost,
    pub constant: Cost,
    pub delay: Cost,
    pub force: Cost,
    pub lambda: Cost,
    pub startup: Cost,
    pub variable: Cost,
}

/// Cost parameters for version `1.1.0`, with `constr` datatypes and `case`.
#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct Datatypes {
    pub construct: Cost,
    pub case: Cost,
}

type Cost = super::Pair<function::Constant, function::Constant>;
