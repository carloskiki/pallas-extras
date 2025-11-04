use crate::{constant::Constant, program::evaluate::Value};
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
    pub fn quantifiers(&self) -> u8 {
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

    pub fn arity(&self) -> u8 {
        match self {
            // Integers
            Builtin::AddInteger => 2,
            Builtin::SubtractInteger => 2,
            Builtin::MultiplyInteger => 2,
            Builtin::DivideInteger => 2,
            Builtin::QuotientInteger => 2,
            Builtin::RemainderInteger => 2,
            Builtin::ModInteger => 2,
            Builtin::EqualsInteger => 2,
            Builtin::LessThanInteger => 2,
            Builtin::LessThanEqualsInteger => 2,
            Builtin::ExpModInteger => 3,
            
            // Bytestrings
            Builtin::AppendByteString => 2,
            Builtin::ConsByteString => 2,
            Builtin::SliceByteString => 3,
            Builtin::LengthOfByteString => 1,
            Builtin::IndexByteString => 2,
            Builtin::EqualsByteString => 2,
            Builtin::LessThanByteString => 2,
            Builtin::LessThanEqualsByteString => 2,
            Builtin::AndByteString => 3,
            Builtin::OrByteString => 3,
            Builtin::XorByteString => 3,
            Builtin::ComplementByteString => 1,
            Builtin::ReadBit => 2,
            Builtin::WriteBits => 3,
            Builtin::ReplicateByte => 2,
            Builtin::ShiftByteString => 2,
            Builtin::RotateByteString => 2,
            Builtin::CountSetBits => 1,
            Builtin::FindFirstSetBit => 1,
            Builtin::IntegerToByteString => 3,
            Builtin::ByteStringToInteger => 2,
            
            // Cryptography and hashes
            Builtin::Sha2_256 => 1,
            Builtin::Sha3_256 => 1,
            Builtin::Blake2b256 => 1,
            Builtin::Blake2b224 => 1,
            Builtin::Keccak256 => 1,
            Builtin::Ripemd160 => 1,
            Builtin::VerifyEd25519Signature => 3,
            Builtin::VerifyEcdsaSecp256k1Signature => 3,
            Builtin::VerifySchnorrSecp256k1Signature => 3,
            
            // Strings
            Builtin::AppendString => 2,
            Builtin::EqualsString => 2,
            Builtin::EncodeUtf8 => 1,
            Builtin::DecodeUtf8 => 1,
            
            // Bool
            Builtin::IfThenElse => 3,
            Builtin::ChooseUnit => 2,
            Builtin::Trace => 2,
            Builtin::FstPair => 1,
            Builtin::SndPair => 1,
            
            // Lists
            Builtin::ChooseList => 3,
            Builtin::MkCons => 2,
            Builtin::HeadList => 1,
            Builtin::TailList => 1,
            Builtin::NullList => 1,
            Builtin::DropList => 2,
            
            // Data
            Builtin::ChooseData => 6,
            Builtin::ConstrData => 2,
            Builtin::MapData => 1,
            Builtin::ListData => 1,
            Builtin::IData => 1,
            Builtin::BData => 1,
            Builtin::UnConstrData => 1,
            Builtin::UnMapData => 1,
            Builtin::UnListData => 1,
            Builtin::UnIData => 1,
            Builtin::UnBData => 1,
            Builtin::EqualsData => 2,
            Builtin::SerialiseData => 1,
            Builtin::MkPairData => 2,
            Builtin::MkNilData => 1,
            Builtin::MkNilPairData => 1,
            
            // BLS12_381 operations
            Builtin::BlsG1Add => 2,
            Builtin::BlsG1Neg => 1,
            Builtin::BlsG1ScalarMul => 2,
            Builtin::BlsG1Equal => 2,
            Builtin::BlsG1HashToGroup => 2,
            Builtin::BlsG1Compress => 1,
            Builtin::BlsG1Uncompress => 1,
            Builtin::BlsG2Add => 2,
            Builtin::BlsG2Neg => 1,
            Builtin::BlsG2ScalarMul => 2,
            Builtin::BlsG2Equal => 2,
            Builtin::BlsG2HashToGroup => 2,
            Builtin::BlsG2Compress => 1,
            Builtin::BlsG2Uncompress => 1,
            Builtin::BlsMillerLoop => 2,
            Builtin::BlsMulMlResult => 2,
            Builtin::BlsFinalVerify => 2,
            Builtin::BlsG1MultiScalarMul => 2,
            Builtin::BlsG2MultiScalarMul => 2,
            
            // Arrays
            Builtin::LengthOfArray => 1,
            Builtin::ListToArray => 1,
            Builtin::IndexArray => 2,
        }
    }

    pub fn apply(
        self,
        args: Vec<Value>,
        constants: &mut Vec<Constant>,
    ) -> Option<Value> {
        let function = match self {
            // Integers
            Builtin::AddInteger => integer::add,
            Builtin::SubtractInteger => integer::subtract,
            Builtin::MultiplyInteger => integer::multiply,
            Builtin::DivideInteger => integer::divide,
            Builtin::QuotientInteger => integer::quotient,
            Builtin::RemainderInteger => integer::remainder,
            Builtin::ModInteger => integer::modulo,
            Builtin::EqualsInteger => integer::equals,
            Builtin::LessThanInteger => integer::less_than,
            Builtin::LessThanEqualsInteger => integer::less_than_or_equal,
            Builtin::ExpModInteger => integer::exp_mod,
            
            // Bytestrings
            Builtin::AppendByteString => bytestring::append,
            Builtin::ConsByteString => bytestring::cons_v2,
            Builtin::SliceByteString => bytestring::slice,
            Builtin::LengthOfByteString => bytestring::length,
            Builtin::IndexByteString => bytestring::index,
            Builtin::EqualsByteString => bytestring::equals,
            Builtin::LessThanByteString => bytestring::less_than,
            Builtin::LessThanEqualsByteString => bytestring::less_than_or_equal,
            Builtin::AndByteString => bytestring::and,
            Builtin::OrByteString => bytestring::or,
            Builtin::XorByteString => bytestring::xor,
            Builtin::ComplementByteString => bytestring::complement,
            Builtin::ReadBit => bytestring::read_bit,
            Builtin::WriteBits => bytestring::write_bits,
            Builtin::ReplicateByte => bytestring::replicate_byte,
            Builtin::ShiftByteString => bytestring::shift,
            Builtin::RotateByteString => bytestring::rotate,
            Builtin::CountSetBits => bytestring::count_set_bits,
            Builtin::FindFirstSetBit => bytestring::first_set_bit,
            Builtin::IntegerToByteString => integer::to_bytes,
            Builtin::ByteStringToInteger => bytestring::to_integer,
            
            // Cryptography and hashes
            Builtin::Sha2_256 => digest::sha2_256,
            Builtin::Sha3_256 => digest::sha3_256,
            Builtin::Blake2b256 => digest::blake2b256,
            Builtin::Blake2b224 => digest::blake2b224,
            Builtin::Keccak256 => digest::keccak256,
            Builtin::Ripemd160 => digest::ripemd160,
            Builtin::VerifyEd25519Signature => ed25519::verify,
            Builtin::VerifyEcdsaSecp256k1Signature => k256::verify_ecdsa,
            Builtin::VerifySchnorrSecp256k1Signature => k256::verify_schnorr,
            
            // Strings
            Builtin::AppendString => string::append,
            Builtin::EqualsString => string::equals,
            Builtin::EncodeUtf8 => string::encode_utf8,
            Builtin::DecodeUtf8 => string::decode_utf8,
            
            // Bool
            Builtin::IfThenElse => if_then_else,
            
            // Unit
            Builtin::ChooseUnit => choose_unit,
            
            // Tracing
            Builtin::Trace => trace,
            
            // Pairs
            Builtin::FstPair => first_pair,
            Builtin::SndPair => second_pair,
            
            // Lists
            Builtin::ChooseList => list::choose,
            Builtin::MkCons => list::mk_cons,
            Builtin::HeadList => list::head,
            Builtin::TailList => list::tail,
            Builtin::NullList => list::null,
            Builtin::DropList => list::drop,
            
            // Data
            Builtin::ChooseData => data::choose,
            Builtin::ConstrData => data::construct,
            Builtin::MapData => data::map,
            Builtin::ListData => data::list,
            Builtin::IData => data::integer,
            Builtin::BData => data::bytes,
            Builtin::UnConstrData => data::un_construct,
            Builtin::UnMapData => data::un_map,
            Builtin::UnListData => data::un_list,
            Builtin::UnIData => data::un_integer,
            Builtin::UnBData => data::un_bytes,
            Builtin::EqualsData => data::equals,
            Builtin::SerialiseData => data::serialize,
            Builtin::MkPairData => data::mk_pair,
            Builtin::MkNilData => data::mk_nil,
            Builtin::MkNilPairData => data::mk_nil_pair,
            
            // BLS12_381 operations
            Builtin::BlsG1Add => bls12_381::g1_add,
            Builtin::BlsG1Neg => bls12_381::g1_neg,
            Builtin::BlsG1ScalarMul => bls12_381::g1_scalar_mul,
            Builtin::BlsG1Equal => bls12_381::g1_equals,
            Builtin::BlsG1HashToGroup => bls12_381::g1_hash_to_group,
            Builtin::BlsG1Compress => bls12_381::g1_compress,
            Builtin::BlsG1Uncompress => bls12_381::g1_uncompress,
            Builtin::BlsG2Add => bls12_381::g2_add,
            Builtin::BlsG2Neg => bls12_381::g2_neg,
            Builtin::BlsG2ScalarMul => bls12_381::g2_scalar_mul,
            Builtin::BlsG2Equal => bls12_381::g2_equals,
            Builtin::BlsG2HashToGroup => bls12_381::g2_hash_to_group,
            Builtin::BlsG2Compress => bls12_381::g2_compress,
            Builtin::BlsG2Uncompress => bls12_381::g2_uncompress,
            Builtin::BlsMillerLoop => bls12_381::miller_loop,
            Builtin::BlsMulMlResult => bls12_381::mul_ml_result,
            Builtin::BlsFinalVerify => bls12_381::final_verify,
            Builtin::BlsG1MultiScalarMul => bls12_381::g1_multi_scalar_mul,
            Builtin::BlsG2MultiScalarMul => bls12_381::g2_multi_scalar_mul,
            
            // Arrays
            Builtin::LengthOfArray => array::length,
            Builtin::ListToArray => list::to_array,
            Builtin::IndexArray => array::index,
        };
        

        function(args, constants)
    }
}

#[apply(builtin)]
pub fn if_then_else(cond: bool, then: Value, else_: Value) -> Value {
    if cond { then } else { else_ }
}

#[apply(builtin)]
pub fn choose_unit(_u: (), then: Value) -> Value {
    then
}

// TODO: do something with the trace.
#[apply(builtin)]
pub fn trace(_message: String, value: Value) -> Value {
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
        #[allow(unused_mut, clippy::ptr_arg)]
        pub fn $name (args: Vec<$crate::program::evaluate::Value>, constants: &mut Vec<$crate::constant::Constant>) -> Option<$crate::program::evaluate::Value> {
            // #[allow(unused_variables)]
            // let $crate::program::evaluate::Value::Constant(const_index) = args[0] else {
            //     unreachable!("Invariant violation: expected the first argument to builtin to be a constant");
            // };
            
            let mut __iter = args.into_iter();
            builtin!(@unwrap ( $($args)+ ) __iter, constants, args);

            builtin!(@result $($rest)+; constants);
        }
    };

    (@unwrap ($arg_name:ident: Value $(, $($rest:tt)*)? ) $iter:ident, $constants:ident, $args:ident) => {
        let $arg_name: Value = $iter.next().expect("builtin has the enough arguments");
        builtin!(@unwrap ($($($rest)*)?) $iter, $constants, $args)
    };
    
    (@unwrap (mut $arg_name:ident: $arg_ty:ty $(, $($rest:tt)*)? ) $iter:ident, $constants:ident, $args:ident) => {
        builtin!(@unwrap ($arg_name: $arg_ty $(, $($rest)*)?) $iter, $constants, $args);
    };

    (@unwrap ($arg_name:ident: $arg_ty:ty $(, $($rest:tt)*)? ) $iter:ident, $constants:ident, $args:ident) => {
        let mut $arg_name: $arg_ty = {
            let $crate::program::evaluate::Value::Constant(constant_index) = $iter.next().expect("builtin has the enough arguments") else {
                return None;
            };
            (&$constants[constant_index.0 as usize]).clone().try_into().ok()?
        };
        builtin!(@unwrap ($($($rest)*)?) $iter, $constants, $args);
    };

    (@unwrap () $iter:ident, $constants:ident, $args:ident) => {};

    (@result Value $block:block; $constants:ident) => {
        #[allow(clippy::redundant_closure_call)]
        return Some((|| $block)());
    };

    (@result Option<$ret:ty> $block:block; $constants:ident) => {{
        #[allow(clippy::redundant_closure_call)]
        let result: $ret = (|| $block)()?;
        builtin!(@wrap $ret; $constants, result);
    }};

    (@result $ret:ty $block:block; $constants:ident) => {{
        #[allow(clippy::redundant_closure_call)]
        let result: $ret = (|| $block)();
        builtin!(@wrap $ret; $constants, result);
    }};

    (@wrap $ret:ty; $constants:ident, $result:ident) => {
        let index = $constants.len();
        $constants.push($result.into());
        return Some($crate::program::evaluate::Value::Constant($crate::ConstantIndex(index as u32)));
    }
}
use builtin;
