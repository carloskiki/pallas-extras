use super::state::State;

pub trait Message {
    const SIZE_LIMIT: usize;
    const TAG: u64;
    const ELEMENT_COUNT: u64;

    type ToState: State;
}
