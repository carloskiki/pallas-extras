use crate::program::{DeBruijn, Program};

pub enum Frame {
    Force,
    LeftApplicationToTerm,
    LeftApplicationToValue,
    RightApplicationToValue,
    ConstructorArgument,
    CaseScrutinee,
}

pub struct Term {
    index: u32,
}

impl Program<DeBruijn> {
    pub fn value(&self, term: Term) -> bool {
        match self.program.get(term.index as usize) {
            Some(term) => term.is_value(),
            None => false,
        }
        
    }
}


// Construct:
// - Only need to know how many remaining you have to evaluate
