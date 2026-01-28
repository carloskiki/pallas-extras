use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

pub mod function;
/// cost parameters for the cek machine
pub mod machine;

const BASE_INDEX: usize = 17;
const DATATYPES_INDEX: usize = 193;

/// Context for cost calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Context<'a> {
    /// The cost model in use.
    pub model: &'a [i64],
    /// Allowed budget for script execution.
    pub budget: super::Budget,
}

impl<'a> Context<'a> {
    /// Get the base cost model.
    ///
    /// This returns none if not enough data is available in the cost model.
    pub(crate) fn base(&self) -> Option<&'a machine::Base> {
        let bytes_prefixed = self.model.get(BASE_INDEX..)?.as_bytes();
        machine::Base::ref_from_prefix(bytes_prefixed)
            .map(|(b, _)| b)
            .ok()
    }

    /// Get the cost model for datatype instructions (introduced in `1.1.0`).
    ///
    /// This returns none if not enough data is available in the cost model.
    pub(crate) fn datatypes(&self) -> Option<&'a machine::Datatypes> {
        let bytes_prefixed = self.model.get(DATATYPES_INDEX..)?.as_bytes();
        machine::Datatypes::ref_from_prefix(bytes_prefixed)
            .map(|(d, _)| d)
            .ok()
    }

    /// Apply a cost function with no arguments to the budget.
    ///
    /// Returns `Some(())` if the cost could be applied, `None` otherwise.
    pub(crate) fn apply_no_args<E: Function, M: Function>(
        &mut self,
        cost: &Pair<E, M>,
    ) -> Option<()> {
        let exec_cost = cost.execution.cost((), (), ());
        let mem_cost = cost.memory.cost((), (), ());
        self.budget.execution = self.budget.execution.checked_sub_signed(exec_cost)?;
        self.budget.memory = self.budget.memory.checked_sub_signed(mem_cost)?;
        Some(())
    }
}

/// A pair of execution and memory costs.
#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct Pair<E, M> {
    pub execution: E,
    pub memory: M,
}

pub trait Argument: Copy {
    fn size(&self) -> u64 {
        unreachable!("The argument does not have a size");
    }
    fn value(&self) -> u64 {
        unreachable!("The argument does not have a value");
    }
}

impl Argument for () {}

/// A cost function that can be applied to arguments of a builtin.
pub trait Function {
    fn cost<X: Argument, Y: Argument, Z: Argument>(&self, x: X, y: Y, z: Z) -> i64;
}
