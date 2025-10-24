use crate::{ValueIndex, constant::Constant};
use macro_rules_attribute::apply;
use strum::{EnumString, FromRepr};

// Builtin Implementations
//
// Take stuff by value, or by shared reference.
//
// INVARIANTS:
//
// - The first argument is never a `*` value (always a constant).
// - Quantifier arguments (`∀`) are found at the start, followed by value arguments.

mod array;
mod bls12_381;
mod bytestring;
mod data;
mod digest;
mod ed25519;
mod integer;
mod k256;
mod list;
mod string;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromRepr, EnumString)]
#[strum(serialize_all = "camelCase")]
pub enum Builtin {
    // Integers
    AddInteger,
    SubtractInteger,
    MultiplyInteger,
    DivideInteger,
    QuotientInteger,
    RemainderInteger,
    ModInteger,
    EqualsInteger,
    LessThanInteger,
    LessThanEqualsInteger,
    // Bytestrings
    AppendByteString,
    ConsByteString,
    SliceByteString,
    LengthOfByteString,
    IndexByteString,
    EqualsByteString,
    LessThanByteString,
    LessThanEqualsByteString,
    // Cryptography and hashes
    #[strum(serialize = "sha2_256")]
    Sha2_256,
    #[strum(serialize = "sha3_256")]
    Sha3_256,
    #[strum(serialize = "blake2b_256")]
    Blake2b256,
    VerifyEd25519Signature, // formerly verifySignature
    VerifyEcdsaSecp256k1Signature = 52,
    VerifySchnorrSecp256k1Signature,
    // Strings
    AppendString = 22,
    EqualsString,
    EncodeUtf8,
    DecodeUtf8,
    // Bool
    IfThenElse,
    // Unit
    ChooseUnit,
    // Tracing
    Trace,
    // Pairs
    FstPair,
    SndPair,
    // Lists
    ChooseList,
    MkCons,
    HeadList,
    TailList,
    NullList,
    // Data
    // See Note [Legacy pattern matching on built-in types].
    // It is convenient to have a "choosing" function for a data type that has more than two
    // constructors to get pattern matching over it and we may end up having multiple such data
    // types, hence we include the name of the data type as a suffix.
    ChooseData,
    ConstrData,
    MapData,
    ListData,
    IData,
    BData,
    UnConstrData,
    UnMapData,
    UnListData,
    UnIData,
    UnBData,
    EqualsData,
    SerialiseData,
    // Misc monomorphized constructors.
    // We could simply replace those with constants, but we use built-in functions for consistency
    // with monomorphic built-in types. Polymorphic built-in constructors are generally problematic,
    // See Note [Representable built-in functions over polymorphic built-in types].
    MkPairData,
    MkNilData,
    MkNilPairData,
    // BLS12_381 operations
    // G1
    #[strum(serialize = "bls12_381_G1_add")]
    BlsG1Add = 54,
    #[strum(serialize = "bls12_381_G1_neg")]
    BlsG1Neg,
    #[strum(serialize = "bls12_381_G1_scalarMul")]
    BlsG1ScalarMul,
    #[strum(serialize = "bls12_381_G1_equal")]
    BlsG1Equal,
    #[strum(serialize = "bls12_381_G1_hashToGroup")]
    BlsG1HashToGroup,
    #[strum(serialize = "bls12_381_G1_compress")]
    BlsG1Compress,
    #[strum(serialize = "bls12_381_G1_uncompress")]
    BlsG1Uncompress,
    // G2
    #[strum(serialize = "bls12_381_G2_add")]
    BlsG2Add,
    #[strum(serialize = "bls12_381_G2_neg")]
    BlsG2Neg,
    #[strum(serialize = "bls12_381_G2_scalarMul")]
    BlsG2ScalarMul,
    #[strum(serialize = "bls12_381_G2_equal")]
    BlsG2Equal,
    #[strum(serialize = "bls12_381_G2_hashToGroup")]
    BlsG2HashToGroup,
    #[strum(serialize = "bls12_381_G2_compress")]
    BlsG2Compress,
    #[strum(serialize = "bls12_381_G2_uncompress")]
    BlsG2Uncompress,
    // Pairing
    #[strum(serialize = "bls12_381_millerLoop")]
    BlsMillerLoop,
    #[strum(serialize = "bls12_381_mulMlResult")]
    BlsMulMlResult,
    #[strum(serialize = "bls12_381_finalVerify")]
    BlsFinalVerify,
    // Keccak_256, Blake2b_224
    #[strum(serialize = "keccak_256")]
    Keccak256,
    #[strum(serialize = "blake2b_224")]
    Blake2b224,
    // Conversions
    IntegerToByteString,
    ByteStringToInteger,
    // Logical
    AndByteString,
    OrByteString,
    XorByteString,
    ComplementByteString,
    ReadBit,
    WriteBits,
    ReplicateByte,
    // Bitwise
    ShiftByteString,
    RotateByteString,
    CountSetBits,
    FindFirstSetBit,
    // Ripemd_160
    #[strum(serialize = "ripemd_160")]
    Ripemd160,
    // Batch 6
    ExpModInteger,
    DropList,
    // Arrays
    LengthOfArray,
    ListToArray,
    IndexArray,
    // BLS12_381 multi scalar multiplication
    #[strum(serialize = "bls12_381_G1_multiScalarMul")]
    BlsG1MultiScalarMul,
    #[strum(serialize = "bls12_381_G2_multiScalarMul")]
    BlsG2MultiScalarMul,
    // // Values
    // InsertCoin,
    // LookupCoin,
    // UnionValue,
    // ValueContains,
    // ValueData,
    // UnValueData,
}

impl Builtin {
    pub fn polymorphic_count(&self) -> u8 {
        match self {
            Builtin::IfThenElse
            | Builtin::ChooseUnit
            | Builtin::Trace
            | Builtin::MkCons
            | Builtin::HeadList
            | Builtin::TailList
            | Builtin::NullList
            | Builtin::ChooseData
            | Builtin::DropList
            | Builtin::LengthOfArray
            | Builtin::ListToArray
            | Builtin::IndexArray => 1,
            Builtin::FstPair | Builtin::SndPair | Builtin::ChooseList => 2,
            _ => 0,
        }
    }
}

#[apply(builtin)]
pub fn if_then_else(cond: bool, then: ValueIndex, else_: ValueIndex) -> ValueIndex {
    if cond { then } else { else_ }
}

#[apply(builtin)]
pub fn choose_unit(_u: (), then: ValueIndex) -> ValueIndex {
    then
}

// TODO: do something with the trace.
#[apply(builtin)]
pub fn trace(_message: String, value: ValueIndex) -> ValueIndex {
    value
}

#[apply(builtin)]
pub fn first_pair(pair: (Constant, Constant)) -> Constant {
    pair.0
}

#[apply(builtin)]
pub fn second_pair(pair: (Constant, Constant)) -> Constant {
    pair.1
}

macro_rules! builtin {
    (pub fn $name:ident ( $($args:tt)+ ) -> $($rest:tt)+) => {
        #[allow(unused_mut)]
        pub fn $name (mut args: Vec<$crate::cek::Value>, constants: &mut [$crate::constant::Constant]) -> Option<$crate::cek::Value> {
            let mut __index = 0u32;
            builtin!(@unwrap ( $($args)+ ) __index, constants, args);

            builtin!(@result $($rest)+; constants, args);
        }
    };

    (@unwrap ($arg_name:ident: ValueIndex $(, $($rest:tt)*)? ) $index:ident, $constants:ident, $args:ident) => {
        let $arg_name = ValueIndex($index);
        $index += 1;
        builtin!(@unwrap ($($($rest)*)?) $index, $constants, $args)
    };
    
    (@unwrap (mut $arg_name:ident: $arg_ty:ty $(, $($rest:tt)*)? ) $index:ident, $constants:ident, $args:ident) => {
        builtin!(@unwrap ($arg_name: $arg_ty $(, $($rest)*)?) $index, $constants, $args);
    };

    (@unwrap ($arg_name:ident: $arg_ty:ty $(, $($rest:tt)*)? ) $index:ident, $constants:ident, $args:ident) => {
        let mut $arg_name: $arg_ty = {
            let $crate::cek::Value::Constant(constant_index) = &$args[$index as usize] else {
                return None;
            };
            std::mem::take(&mut $constants[constant_index.0 as usize]).try_into().ok()?
        };
        $index += 1;
        builtin!(@unwrap ($($($rest)*)?) $index, $constants, $args);
    };

    (@unwrap () $index:ident, $constants:ident, $args:ident) => {};

    (@result ValueIndex $block:block; $constants:ident, $args:ident) => {
        #[allow(clippy::redundant_closure_call)]
        let result: $crate::ValueIndex = (|| $block)();
        return Some($args.swap_remove(result.0 as usize));
    };

    (@result Option<$ret:ty> $block:block; $constants:ident, $args:ident) => {{
        #[allow(clippy::redundant_closure_call)]
        let result: $ret = (|| $block)()?;
        builtin!(@wrap $ret; $constants, $args, result);
    }};

    (@result $ret:ty $block:block; $constants:ident, $args:ident) => {{
        #[allow(clippy::redundant_closure_call)]
        let result: $ret = (|| $block)();
        builtin!(@wrap $ret; $constants, $args, result);
    }};

    (@wrap $ret:ty; $constants:ident, $args:ident, $result:ident) => {
        let $crate::cek::Value::Constant(const_index) = $args[0] else {
            panic!("Invariant violation: expected the first argument to builtin to be a constant");
        };

        $constants[const_index.0 as usize] = $result.into();
        return Some($crate::cek::Value::Constant(const_index));
    }
}
use builtin;
