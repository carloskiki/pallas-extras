use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

pub mod function;
/// cost parameters for the cek machine
pub mod machine;

const BASE_INDEX: usize = 17;
const DATATYPES_INDEX: usize = 193;

/// Context for cost calculation.
pub struct Context<'a> {
    /// The cost model in use.
    pub model: &'a [i64],
    /// Allowed budget for script execution.
    pub budget: ledger::alonzo::script::execution::Units,
}

impl<'a> Context<'a> {
    /// Get the base cost model.
    ///
    /// This returns none if not enough data is available in the cost model.
    pub(crate) fn base(&self) -> Option<&'a machine::Base> {
        let bytes_prefixed = self.model.get(BASE_INDEX..)?.as_bytes();
        machine::Base::ref_from_bytes(bytes_prefixed).ok()
    }

    /// Get the cost model for datatype instructions (introduced in `1.1.0`).
    ///
    /// This returns none if not enough data is available in the cost model.
    pub(crate) fn datatypes(&self) -> Option<&'a machine::Datatypes> {
        let bytes_prefixed = self.model.get(DATATYPES_INDEX..)?.as_bytes();
        machine::Datatypes::ref_from_bytes(bytes_prefixed).ok()
    }
}

/// A pair of execution and memory costs.
#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct Pair<E, M> {
    pub execution: E,
    pub memory: M,
}

pub trait Argument {
    fn size(&self) -> u64 {
        unreachable!("The argument does not have a size");
    }
    fn value(&self) -> u64 {
        unreachable!("The argument does not have a value");
    }
}

/// A cost function that can be applied to arguments of a builtin.
pub trait Function {
    fn cost<X: Argument, Y: Argument, Z: Argument>(&self, x: X, y: Y, z: Y) -> i64;
}
