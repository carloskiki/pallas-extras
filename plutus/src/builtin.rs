//! Builtin functions supported by the evaluator.
//!
//! Each builtin function is defined in the [specification][spec] section 4.3.
//!
//! The submodules contain implementations of built-in functions roughly grouped by their types.
//!
//! [spec]: https://plutus.cardano.intersectmbo.org/resources/plutus-core-spec.pdf

use crate::{
    constant::Constant,
    cost::{self, function as cf},
    machine,
};
use strum::{EnumString, FromRepr};
use zerocopy::{FromBytes, IntoBytes};

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

/// Builtin functions supported by the evaluator.
///
/// Expected Invariants:
/// - Quantifier arguments (`∀`) are found at the start, followed by value arguments.
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
    // Misc monomorphized constructors.
    MkPairData,
    MkNilData,
    MkNilPairData,
    SerialiseData,
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
    #[strum(serialize = "bls12_381_G1_compress")]
    BlsG1Compress,
    #[strum(serialize = "bls12_381_G1_uncompress")]
    BlsG1Uncompress,
    #[strum(serialize = "bls12_381_G1_hashToGroup")]
    BlsG1HashToGroup,
    // G2
    #[strum(serialize = "bls12_381_G2_add")]
    BlsG2Add,
    #[strum(serialize = "bls12_381_G2_neg")]
    BlsG2Neg,
    #[strum(serialize = "bls12_381_G2_scalarMul")]
    BlsG2ScalarMul,
    #[strum(serialize = "bls12_381_G2_equal")]
    BlsG2Equal,
    #[strum(serialize = "bls12_381_G2_compress")]
    BlsG2Compress,
    #[strum(serialize = "bls12_381_G2_uncompress")]
    BlsG2Uncompress,
    #[strum(serialize = "bls12_381_G2_hashToGroup")]
    BlsG2HashToGroup,
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
    /// Returns the number of quantifier arguments (`∀`) of the builtin function.
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

    /// Returns the arity (number of value arguments) of the builtin function.
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

    /// Applies the builtin function to the given arguments.
    ///
    /// # Panic
    ///
    /// Panics if the number of arguments does not match the arity of the builtin function.
    pub fn apply(
        self,
        args: Vec<machine::Value>,
        constants: &mut Vec<Constant>,
        context: &mut cost::Context,
    ) -> Option<machine::Value> {
        const fn offset(builtin: Builtin) -> usize {
            let mut offset = 0;
            let mut i = 0;
            while i < OFFSETS.len() {
                if OFFSETS[i].0 as u8 == builtin as u8 {
                    if offset >= cost::machine::BASE_INDEX {
                        offset += std::mem::size_of::<cost::machine::Base>() / 8;
                    }
                    if offset >= cost::machine::DATATYPES_INDEX {
                        offset += std::mem::size_of::<cost::machine::Datatypes>() / 8;
                    }

                    return offset;
                }
                offset += OFFSETS[i].1;
                i += 1;
            }
            panic!("all builtins are in the list");
        }

        // IMPORTANT: The order matters here! The builtins are listed in order of cost model
        // appearance.
        builtins! {
            [self, args, constants, context]
            AddInteger<cf::Affine<cf::Max<cf::First, cf::Second>>, cf::Affine<cf::Max<cf::First, cf::Second>>> => integer::add,
            AppendByteString<cf::Affine<cf::Add<cf::First, cf::Second>>, cf::Affine<cf::Add<cf::First, cf::Second>>> => bytestring::append,
            AppendString<cf::Affine<cf::Add<cf::First, cf::Second>>, cf::Affine<cf::Add<cf::First, cf::Second>>> => string::append,
            BData<cf::Constant, cf::Constant> => data::bytes,
            Blake2b256<cf::Affine<cf::First>, cf::Constant> => digest::blake2b256,
            ChooseData<cf::Constant, cf::Constant> => data::choose,
            ChooseList<cf::Constant, cf::Constant> => list::choose,
            ChooseUnit<cf::Constant, cf::Constant> => choose_unit,
            ConsByteString<cf::Affine<cf::Second>, cf::Affine<cf::Add<cf::First, cf::Second>>> => bytestring::cons_v2,
            ConstrData<cf::Constant, cf::Constant> => data::construct,
            DecodeUtf8<cf::Affine<cf::First>, cf::Affine<cf::First>> => string::decode_utf8,
            DivideInteger<cf::Divide, cf::Add<cf::Constant, cf::Mul<cf::Max<cf::Sub<cf::First, cf::Second>, cf::Constant>, cf::Constant>>> => integer::divide,
            EncodeUtf8<cf::Affine<cf::First>, cf::Affine<cf::First>> => string::encode_utf8,
            EqualsByteString<cf::StringEqualsExecution, cf::Constant> => bytestring::equals,
            EqualsData<cf::Affine<cf::Min<cf::First, cf::Second>>, cf::Constant> => data::equals,
            EqualsInteger<cf::Affine<cf::Max<cf::First, cf::Second>>, cf::Constant> => integer::equals,
            EqualsString<cf::StringEqualsExecution, cf::Constant> => string::equals,
            FstPair<cf::Constant, cf::Constant> => first_pair,
            HeadList<cf::Constant, cf::Constant> => list::head,
            IData<cf::Constant, cf::Constant> => data::integer,
            IfThenElse<cf::Constant, cf::Constant> => if_then_else,
            IndexByteString<cf::Constant, cf::Constant> => bytestring::index,
            LengthOfByteString<cf::Constant, cf::Constant> => bytestring::length,
            LessThanByteString<cf::Affine<cf::Min<cf::First, cf::Second>>, cf::Constant> => bytestring::less_than,
            LessThanEqualsByteString<cf::Affine<cf::Min<cf::First, cf::Second>>, cf::Constant> => bytestring::less_than_or_equal,
            LessThanEqualsInteger<cf::Affine<cf::Min<cf::First, cf::Second>>, cf::Constant> => integer::less_than_or_equal,
            LessThanInteger<cf::Affine<cf::Min<cf::First, cf::Second>>, cf::Constant> => integer::less_than,
            ListData<cf::Constant, cf::Constant> => data::list,
            MapData<cf::Constant, cf::Constant> => data::map,
            MkCons<cf::Constant, cf::Constant> => list::mk_cons,
            MkNilData<cf::Constant, cf::Constant> => data::mk_nil,
            MkNilPairData<cf::Constant, cf::Constant> => data::mk_nil_pair,
            MkPairData<cf::Constant, cf::Constant> => data::mk_pair,
            ModInteger<cf::Divide, cf::Affine<cf::Second>> => integer::modulo,
            MultiplyInteger<cf::Affine<cf::Mul<cf::First, cf::Second>>, cf::Affine<cf::Add<cf::First, cf::Second>>> => integer::multiply,
            NullList<cf::Constant, cf::Constant> => list::null,
            QuotientInteger<cf::Divide, cf::Add<cf::Constant, cf::Mul<cf::Max<cf::Sub<cf::First, cf::Second>, cf::Constant>, cf::Constant>>> => integer::quotient,
            RemainderInteger<cf::Divide, cf::Affine<cf::Second>> => integer::remainder,
            SerialiseData<cf::Affine<cf::First>, cf::Affine<cf::First>> => data::serialize,
            Sha2_256<cf::Affine<cf::First>, cf::Constant> => digest::sha2_256,
            Sha3_256<cf::Affine<cf::First>, cf::Constant> => digest::sha3_256,
            SliceByteString<cf::Affine<cf::Third>, cf::Affine<cf::Third>> => bytestring::slice,
            SndPair<cf::Constant, cf::Constant> => second_pair,
            SubtractInteger<cf::Affine<cf::Max<cf::First, cf::Second>>, cf::Affine<cf::Max<cf::First, cf::Second>>> => integer::subtract,
            TailList<cf::Constant, cf::Constant> => list::tail,
            Trace<cf::Constant, cf::Constant> => trace,
            UnBData<cf::Constant, cf::Constant> => data::un_bytes,
            UnConstrData<cf::Constant, cf::Constant> => data::un_construct,
            UnIData<cf::Constant, cf::Constant> => data::un_integer,
            UnListData<cf::Constant, cf::Constant> => data::un_list,
            UnMapData<cf::Constant, cf::Constant> => data::un_map,
            VerifyEcdsaSecp256k1Signature<cf::Constant, cf::Constant> => k256::verify_ecdsa,
            VerifyEd25519Signature<cf::Affine<cf::Second>, cf::Constant> => ed25519::verify,
            VerifySchnorrSecp256k1Signature<cf::Affine<cf::Second>, cf::Constant> => k256::verify_schnorr,
            BlsG1Add<cf::Constant, cf::Constant> => bls12_381::g1_add,
            BlsG1Compress<cf::Constant, cf::Constant> => bls12_381::g1_compress,
            BlsG1Equal<cf::Constant, cf::Constant> => bls12_381::g1_equals,
            BlsG1HashToGroup<cf::Affine<cf::First>, cf::Constant> => bls12_381::g1_hash_to_group,
            BlsG1Neg<cf::Constant, cf::Constant> => bls12_381::g1_neg,
            BlsG1ScalarMul<cf::Affine<cf::First>, cf::Constant> => bls12_381::g1_scalar_mul,
            BlsG1Uncompress<cf::Constant, cf::Constant> => bls12_381::g1_uncompress,
            BlsG2Add<cf::Constant, cf::Constant> => bls12_381::g2_add,
            BlsG2Compress<cf::Constant, cf::Constant> => bls12_381::g2_compress,
            BlsG2Equal<cf::Constant, cf::Constant> => bls12_381::g2_equals,
            BlsG2HashToGroup<cf::Affine<cf::First>, cf::Constant> => bls12_381::g2_hash_to_group,
            BlsG2Neg<cf::Constant, cf::Constant> => bls12_381::g2_neg,
            BlsG2ScalarMul<cf::Affine<cf::First>, cf::Constant> => bls12_381::g2_scalar_mul,
            BlsG2Uncompress<cf::Constant, cf::Constant> => bls12_381::g2_uncompress,
            BlsFinalVerify<cf::Constant, cf::Constant> => bls12_381::final_verify,
            BlsMillerLoop<cf::Constant, cf::Constant> => bls12_381::miller_loop,
            BlsMulMlResult<cf::Constant, cf::Constant> => bls12_381::mul_ml_result,
            Keccak256<cf::Affine<cf::First>, cf::Constant> => digest::keccak256,
            Blake2b224<cf::Affine<cf::First>, cf::Constant> => digest::blake2b224,
            IntegerToByteString<cf::Quadratic<cf::Third>, cf::IntegerToByteStringMemory> => integer::to_bytes,
            ByteStringToInteger<cf::Quadratic<cf::Second>, cf::Affine<cf::Second>> => bytestring::to_integer,
            AndByteString<cf::Affine2<cf::Second, cf::Third>, cf::Affine<cf::Max<cf::Second, cf::Third>>> => bytestring::and,
            OrByteString<cf::Affine2<cf::Second, cf::Third>, cf::Affine<cf::Max<cf::Second, cf::Third>>> => bytestring::or,
            XorByteString<cf::Affine2<cf::Second, cf::Third>, cf::Affine<cf::Max<cf::Second, cf::Third>>> => bytestring::xor,
            ComplementByteString<cf::Affine<cf::First>, cf::Affine<cf::First>> => bytestring::complement,
            ReadBit<cf::Constant, cf::Constant> => bytestring::read_bit,
            WriteBits<cf::Affine<cf::Second>, cf::Affine<cf::First>> => bytestring::write_bits,
            ReplicateByte<cf::Affine<cf::FirstIntegerAsBytes>, cf::Affine<cf::FirstIntegerAsBytes>> => bytestring::replicate_byte,
            ShiftByteString<cf::Affine<cf::First>, cf::Affine<cf::First>> => bytestring::shift,
            RotateByteString<cf::Affine<cf::First>, cf::Affine<cf::First>> => bytestring::rotate,
            CountSetBits<cf::Affine<cf::First>, cf::Constant> => bytestring::count_set_bits,
            FindFirstSetBit<cf::Affine<cf::First>, cf::Constant> => bytestring::first_set_bit,
            Ripemd160<cf::Affine<cf::First>, cf::Constant> => digest::ripemd160,

            ExpModInteger<cf::ExpModIntegerExecution, cf::Affine<cf::Third>> => integer::exp_mod,
            DropList<cf::Affine<cf::FirstInteger>, cf::Constant> => list::drop,
            LengthOfArray<cf::Constant, cf::Constant> => array::length,
            ListToArray<cf::Affine<cf::First>, cf::Affine<cf::First>> => list::to_array,
            IndexArray<cf::Constant, cf::Constant> => array::index,
            BlsG1MultiScalarMul<cf::Affine<cf::First>, cf::Constant> => bls12_381::g1_multi_scalar_mul,
            BlsG2MultiScalarMul<cf::Affine<cf::First>, cf::Constant> => bls12_381::g2_multi_scalar_mul,
        }
    }
}

pub fn if_then_else(cond: bool, then: machine::Value, else_: machine::Value) -> machine::Value {
    if cond { then } else { else_ }
}

pub fn choose_unit(_: (), then: machine::Value) -> machine::Value {
    then
}

pub fn trace(message: String, value: machine::Value) -> machine::Value {
    log::info!("{message}");
    println!("{message}");
    value
}

pub fn first_pair(pair: (Constant, Constant)) -> Constant {
    pair.0
}

pub fn second_pair(pair: (Constant, Constant)) -> Constant {
    pair.1
}

/// Convert a machine value into a builtin argument.
pub trait Input: Sized {
    fn from(value: machine::Value, constants: &[Constant]) -> Option<Self>;
}

/// Convert a builtin return value into a machine value.
pub trait Output {
    fn into(value: Self, constants: &mut Vec<Constant>) -> Option<machine::Value>;
}

/// Any constant can be used as a value.
impl<C: TryFrom<Constant>> Input for C {
    fn from(value: machine::Value, constants: &[Constant]) -> Option<Self> {
        let machine::Value::Constant(constant_index) = value else {
            return None;
        };
        (constants[constant_index.0 as usize])
            .clone()
            .try_into()
            .ok()
    }
}

impl<C: Into<Constant>> Output for C {
    fn into(value: Self, constants: &mut Vec<Constant>) -> Option<machine::Value> {
        let constant = value.into();
        let index = constants.len();
        constants.push(constant);
        Some(machine::Value::Constant(crate::ConstantIndex(index as u32)))
    }
}

impl<C: Into<Constant>> Output for Option<C> {
    fn into(value: Self, constants: &mut Vec<Constant>) -> Option<machine::Value> {
        match value {
            Some(v) => Output::into(v, constants),
            None => None,
        }
    }
}

/// Any machine value can be used as a builtin input.
impl Input for machine::Value {
    fn from(value: machine::Value, _constants: &[Constant]) -> Option<Self> {
        Some(value)
    }
}

/// Any machine value can be used as a builtin return value.
impl Output for machine::Value {
    fn into(value: Self, _constants: &mut Vec<Constant>) -> Option<machine::Value> {
        Some(value)
    }
}

/// A builtin function that can be applied to arguments.
pub trait Function<I, CE, CM> {
    fn apply(
        self,
        args: Vec<machine::Value>,
        constants: &mut Vec<Constant>,
        context: &mut cost::Context,
    ) -> Option<machine::Value>;
}

impl_function!(A);
impl_function!(A, B);
impl_function!(A, B, C);
impl_function!(A, B, C, D);
impl_function!(A, B, C, D, E);
impl_function!(A, B, C, D, E, F);

macro_rules! impl_function {
    ($($ty:ident),*) => {
        #[allow(unused_parens, non_snake_case)]
        impl<O: Output, FN, CE, CM, $($ty: Input),*> Function<($($ty,)*), CE, CM> for FN
        where
            FN: Fn($($ty),*) -> O,
            CE: cost::Function<($($ty),*)>,
            CM: cost::Function<($($ty),*)>,
        {
            fn apply(
                self,
                args: Vec<machine::Value>,
                constants: &mut Vec<Constant>,
                context: &mut cost::Context,
            ) -> Option<machine::Value> {
                let mut args = args.into_iter();
                let tuple = (
                    $(
                        $ty::from(
                            args.next().expect("correct number of arguments passed"),
                            constants
                        )?
                    ),*
                );

                let cost::Pair { execution, memory } = cost::Pair::<CE, CM>::ref_from_prefix(
                    context.model.as_bytes(),
                ).ok()?.0;
                let execution_cost = execution.cost(&tuple);
                context.budget.execution = context
                    .budget
                    .execution
                    .checked_sub_signed(execution_cost)?;
                let memory_cost = memory.cost(&tuple);
                context.budget.memory = context
                    .budget
                    .memory
                    .checked_sub_signed(memory_cost)?;

                let ($($ty),*) = tuple;
                let output = (self)($($ty),*);
                O::into(output, constants)
            }
        }
    };
}
use impl_function;

/// Provide the builtins in order of cost model entry.
///
/// This calls the `Function::apply` implementation for each builtin with the specified cost_model
/// and correct offset based on the function's index.
macro_rules! builtins {
    ([$var:ident, $args:ident, $constants:ident, $context:ident] $($builtin:ident<$execution:ty, $memory:ty> => $fn:path),* $(,)?) => {
        const OFFSETS: &[(Builtin, usize)] = &[
            $(
                (
                    Builtin::$builtin,
                    std::mem::size_of::<cost::Pair<$execution, $memory>>() / 8,
                ),
            )*
        ];

        let full_model = $context.model;
        let ret = match $var {
            $(
                b @ Builtin::$builtin => <_ as Function<
                    _,
                    $execution,
                    $memory,
                >>::apply($fn, $args, $constants, {
                    $context.model = &$context.model[offset(b)..];
                    $context
                }),
            )*
        };
        $context.model = full_model;
        ret
    };
}
use builtins;

// #[test]
// fn bisect_list() {
//     let builtin = Builtin::ByteStringToInteger;
//
//     builtin.apply(
//         vec![],
//         &mut vec![],
//         &mut cost::Context {
//             model: &[],
//             budget: crate::Budget {
//                 memory: u64::MAX,
//                 execution: u64::MAX,
//             },
//         }
//     );
// }
