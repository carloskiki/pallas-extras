//! Cost accounting.
//!
//! This module defines the cost accounting [`Context`], as well as the cost [`Function`]s and
//! parameters used by the CEK machine and built-in functions.

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

pub mod function;
pub mod machine;

/// Cost accounting context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Context<'a> {
    /// The cost model.
    pub model: &'a [i64],
    /// Allowed budget for script execution.
    pub budget: super::Budget,
}

impl<'a> Context<'a> {
    /// Get the base cost model.
    ///
    /// This returns none if not enough data is available in the cost model.
    pub(crate) fn base(&self) -> Option<&'a machine::Base> {
        let bytes_prefixed = self.model.get(machine::BASE_INDEX..)?.as_bytes();
        machine::Base::ref_from_prefix(bytes_prefixed)
            .map(|(b, _)| b)
            .ok()
    }

    /// Get the cost model for datatype instructions (introduced in `1.1.0`).
    ///
    /// This returns none if not enough data is available in the cost model.
    pub(crate) fn datatypes(&self) -> Option<&'a machine::Datatypes> {
        let bytes_prefixed = self.model.get(machine::DATATYPES_INDEX..)?.as_bytes();
        machine::Datatypes::ref_from_prefix(bytes_prefixed)
            .map(|(d, _)| d)
            .ok()
    }

    /// Apply a cost function with no arguments to the budget.
    ///
    /// Returns `Some(())` if the cost could be applied, `None` otherwise.
    pub(crate) fn apply_no_args<E: Function<()>, M: Function<()>>(
        &mut self,
        cost: &function::Pair<E, M>,
    ) -> Option<()> {
        let exec_cost = cost.execution.cost(&());
        let mem_cost = cost.memory.cost(&());
        self.budget.execution = self.budget.execution.checked_sub_signed(exec_cost)?;
        self.budget.memory = self.budget.memory.checked_sub_signed(mem_cost)?;
        Some(())
    }
}

/// A cost function for a [`builtin`](crate::builtin).
/// 
/// A simple example is [`function::Constant`], which ignores its inputs and returns the cost given
/// by the cost model. Other functions are combinators, such as [`function::ops::Add`], which sums
/// the cost of its two sub-functions. Concretely, an affine function of the first input (i.e., `a
/// + b*x`) is simply `Add<Constant, Mul<Constant, First>>`. This is a common pattern, so there is
/// a type alias [`function::Affine`] for it.
pub trait Function<I>: FromBytes + Immutable + KnownLayout {
    /// Compute the cost for the given inputs.
    fn cost(&self, inputs: &I) -> i64;
}
