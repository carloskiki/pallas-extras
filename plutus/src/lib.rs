pub struct Program {
    version: Version,
}

pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

pub enum Term {
    Variable,
    Delay,
    Lambda,
    Application,
    Constant,
    Force,
    Error,
    Builtin,
    // Version 1.1.0
    Constructor,
    Case,
}

pub enum Builtin {
    Integer,
    Bytes,
    String,
    Unit,
    Boolean,
    List,
    Pair = 0b0110,
    // TypeApplication = 0b0111, Probably only for decoding
    Data = 0b1000,
    BLSG1Element,
    BLSG2Element,
    BLSMlResult,
    Array,
}


pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
