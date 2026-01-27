use crate::evaluate::Value;

pub mod function;

/// Context for cost calculation.
pub struct Context<'a> {
    pub model: &'a [i64],
    pub budget: ledger::alonzo::script::execution::Units,
}

pub trait Function {
    fn cost(&self, agruments: &[Value]) -> i64;
}
