use std::fmt::Display;

use crate::{byron::transaction, epoch};
use cbor_util::ArrayOption;
use sparse_struct::SparseStruct;
use tinycbor::{
    CborLen, Decode, Encode,
    collections::{self, fixed},
    num, primitive,
};

pub mod version;
pub use version::Version;

pub mod soft_fork;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SparseStruct)]
#[struct_name = "Parameters"]
#[struct_derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Parameter {
    ScriptVersion(u16),
    SlotDuration(u64), // Try u64, and fallback to bigint if needed (also try u32 if it works..).
    MaxBlockSize(u64),
    MaxHeaderSize(u64),
    MaxTransactionSize(u64),
    MaxProposalSize(u64),
    MultiPartyComputationThreshold(u64), // TODO: LovelacePortion newtype...
    HeavyDelegationThreshold(u64),
    UpdateVoteThreshold(u64),
    //  ^ Time to live for a protocol update proposal. This used to be the number
    //  of slots after which the system made a decision regarding an update
    //  proposal confirmation, when a majority of votes was not reached in the
    //  given number of slots. If there were more positive than negative votes the
    //  proposal became confirmed, otherwise it was rejected. Since in the
    //  Byron-Shelley bridge we do not have negative votes, and we aim at
    //  simplifying the update mechanism, 'ppUpdateProposalTTL' is re-interpreted as
    //  the number of slots a proposal has to gather a majority of votes. If a
    //  majority of votes has not been reached before this period, then the
    //  proposal is rejected.
    //
    //  -- TODO: it seems this should be a slot count.
    UpdateProposalThreshold(u64),
    UpdateProposalTTL(u64),
    SoftForkRule(soft_fork::Rule),
    TransactionFeePolicy(transaction::FeePolicy),
    UnlockStakeEpoch(epoch::Number),
}

const PARAMETER_COUNT: usize = 14;

impl Encode for Parameters {
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        e.array(PARAMETER_COUNT)?;
        ArrayOption(self.script_version()).encode(e)?;
        ArrayOption(self.slot_duration()).encode(e)?;
        ArrayOption(self.max_block_size()).encode(e)?;
        ArrayOption(self.max_header_size()).encode(e)?;
        ArrayOption(self.max_transaction_size()).encode(e)?;
        ArrayOption(self.max_proposal_size()).encode(e)?;
        ArrayOption(self.multi_party_computation_threshold()).encode(e)?;
        ArrayOption(self.heavy_delegation_threshold()).encode(e)?;
        ArrayOption(self.update_vote_threshold()).encode(e)?;
        ArrayOption(self.update_proposal_threshold()).encode(e)?;
        ArrayOption(self.update_proposal_ttl()).encode(e)?;
        ArrayOption(self.soft_fork_rule()).encode(e)?;
        ArrayOption(self.transaction_fee_policy()).encode(e)?;
        ArrayOption(self.unlock_stake_epoch()).encode(e)?;
        Ok(())
    }
}

impl CborLen for Parameters {
    fn cbor_len(&self) -> usize {
        let mut len = PARAMETER_COUNT.cbor_len();
        len += ArrayOption(self.script_version()).cbor_len();
        len += ArrayOption(self.slot_duration()).cbor_len();
        len += ArrayOption(self.max_block_size()).cbor_len();
        len += ArrayOption(self.max_header_size()).cbor_len();
        len += ArrayOption(self.max_transaction_size()).cbor_len();
        len += ArrayOption(self.max_proposal_size()).cbor_len();
        len += ArrayOption(self.multi_party_computation_threshold()).cbor_len();
        len += ArrayOption(self.heavy_delegation_threshold()).cbor_len();
        len += ArrayOption(self.update_vote_threshold()).cbor_len();
        len += ArrayOption(self.update_proposal_threshold()).cbor_len();
        len += ArrayOption(self.update_proposal_ttl()).cbor_len();
        len += ArrayOption(self.soft_fork_rule()).cbor_len();
        len += ArrayOption(self.transaction_fee_policy()).cbor_len();
        len += ArrayOption(self.unlock_stake_epoch()).cbor_len();
        len
    }
}

impl Decode<'_> for Parameters {
    type Error = fixed::Error<Error>;

    fn decode(d: &mut tinycbor::Decoder<'_>) -> Result<Self, Self::Error> {
        fn decode_opt<'a, T: Decode<'a>>(
            v: &mut tinycbor::ArrayVisitor<'_, 'a>,
            f: impl Fn(<ArrayOption<T> as Decode<'a>>::Error) -> Error,
        ) -> Result<Option<T>, fixed::Error<Error>> {
            v.visit::<ArrayOpt<_>>()
                .ok_or(fixed::Error::Missing)?
                .map_err(|e| fixed::Error::Collection(collections::Error::Element(f(e))))
                .map(|a| a.0)
        }

        let mut parameters: Self = Self::default();
        let mut visitor = d.array_visitor().map_err(collections::Error::Malformed)?;

        if let Some(script_version) = decode_opt(&mut visitor, Error::ScriptVersion)? {
            parameters.insert(Parameter::ScriptVersion(script_version));
        }
        if let Some(slot_duration) = decode_opt(&mut visitor, Error::SlotDuration)? {
            parameters.insert(Parameter::SlotDuration(slot_duration));
        }
        if let Some(max_block_size) = decode_opt(&mut visitor, Error::MaxBlockSize)? {
            parameters.insert(Parameter::MaxBlockSize(max_block_size));
        }
        if let Some(max_header_size) = decode_opt(&mut visitor, Error::MaxHeaderSize)? {
            parameters.insert(Parameter::MaxHeaderSize(max_header_size));
        }
        if let Some(max_transaction_size) = decode_opt(&mut visitor, Error::MaxTransactionSize)? {
            parameters.insert(Parameter::MaxTransactionSize(max_transaction_size));
        }
        if let Some(max_proposal_size) = decode_opt(&mut visitor, Error::MaxProposalSize)? {
            parameters.insert(Parameter::MaxProposalSize(max_proposal_size));
        }
        if let Some(multi_party_computation_threshold) =
            decode_opt(&mut visitor, Error::MultiPartyComputationThreshold)?
        {
            parameters.insert(Parameter::MultiPartyComputationThreshold(
                multi_party_computation_threshold,
            ));
        }
        if let Some(heavy_delegation_threshold) =
            decode_opt(&mut visitor, Error::HeavyDelegationThreshold)?
        {
            parameters.insert(Parameter::HeavyDelegationThreshold(
                heavy_delegation_threshold,
            ));
        }
        if let Some(update_vote_threshold) = decode_opt(&mut visitor, Error::UpdateVoteThreshold)? {
            parameters.insert(Parameter::UpdateVoteThreshold(update_vote_threshold));
        }
        if let Some(update_proposal_threshold) =
            decode_opt(&mut visitor, Error::UpdateProposalThreshold)?
        {
            parameters.insert(Parameter::UpdateProposalThreshold(
                update_proposal_threshold,
            ));
        }
        if let Some(update_proposal_ttl) = decode_opt(&mut visitor, Error::UpdateProposalTTL)? {
            parameters.insert(Parameter::UpdateProposalTTL(update_proposal_ttl));
        }
        if let Some(soft_fork_rule) = decode_opt(&mut visitor, Error::SoftForkRule)? {
            parameters.insert(Parameter::SoftForkRule(soft_fork_rule));
        }
        if let Some(transaction_fee_policy) = decode_opt(&mut visitor, Error::TransactionFeePolicy)?
        {
            parameters.insert(Parameter::TransactionFeePolicy(transaction_fee_policy));
        }
        if let Some(unlock_stake_epoch) = decode_opt(&mut visitor, Error::UnlockStakeEpoch)? {
            parameters.insert(Parameter::UnlockStakeEpoch(unlock_stake_epoch));
        }

        Ok(parameters)
    }
}

#[derive(Debug)]
pub enum Error {
    ScriptVersion(fixed::Error<num::Error>),
    SlotDuration(fixed::Error<primitive::Error>),
    MaxBlockSize(fixed::Error<primitive::Error>),
    MaxHeaderSize(fixed::Error<primitive::Error>),
    MaxTransactionSize(fixed::Error<primitive::Error>),
    MaxProposalSize(fixed::Error<primitive::Error>),
    MultiPartyComputationThreshold(fixed::Error<primitive::Error>),
    HeavyDelegationThreshold(fixed::Error<primitive::Error>),
    UpdateVoteThreshold(fixed::Error<primitive::Error>),
    UpdateProposalThreshold(fixed::Error<primitive::Error>),
    UpdateProposalTTL(fixed::Error<primitive::Error>),
    SoftForkRule(fixed::Error<<soft_fork::Rule as Decode<'static>>::Error>),
    TransactionFeePolicy(fixed::Error<<transaction::FeePolicy as Decode<'static>>::Error>),
    UnlockStakeEpoch(fixed::Error<<epoch::Number as Decode<'static>>::Error>),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ScriptVersion(e) => write!(f, "in ScriptVersion: {}", e),
            Error::SlotDuration(e) => write!(f, "in SlotDuration: {}", e),
            Error::MaxBlockSize(e) => write!(f, "in MaxBlockSize: {}", e),
            Error::MaxHeaderSize(e) => write!(f, "in MaxHeaderSize: {}", e),
            Error::MaxTransactionSize(e) => write!(f, "in MaxTransactionSize: {}", e),
            Error::MaxProposalSize(e) => write!(f, "in MaxProposalSize: {}", e),
            Error::MultiPartyComputationThreshold(e) => {
                write!(f, "in MultiPartyComputationThreshold: {}", e)
            }
            Error::HeavyDelegationThreshold(e) => write!(f, "in HeavyDelegationThreshold: {}", e),
            Error::UpdateVoteThreshold(e) => write!(f, "in UpdateVoteThreshold: {}", e),
            Error::UpdateProposalThreshold(e) => write!(f, "in UpdateProposalThreshold: {}", e),
            Error::UpdateProposalTTL(e) => write!(f, "in UpdateProposalTTL: {}", e),
            Error::SoftForkRule(e) => write!(f, "in SoftForkRule: {}", e),
            Error::TransactionFeePolicy(e) => write!(f, "in TransactionFeePolicy: {}", e),
            Error::UnlockStakeEpoch(e) => write!(f, "in UnlockStakeEpoch: {}", e),
        }
    }
}

impl core::error::Error for Error {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(match self {
            Error::ScriptVersion(e) => e,
            Error::SlotDuration(e) => e,
            Error::MaxBlockSize(e) => e,
            Error::MaxHeaderSize(e) => e,
            Error::MaxTransactionSize(e) => e,
            Error::MaxProposalSize(e) => e,
            Error::MultiPartyComputationThreshold(e) => e,
            Error::HeavyDelegationThreshold(e) => e,
            Error::UpdateVoteThreshold(e) => e,
            Error::UpdateProposalThreshold(e) => e,
            Error::UpdateProposalTTL(e) => e,
            Error::SoftForkRule(e) => e,
            Error::TransactionFeePolicy(e) => e,
            Error::UnlockStakeEpoch(e) => e,
        })
    }
}

struct ArrayOpt<T>(Option<T>);

impl<T> Encode for ArrayOpt<T>
where
    T: Encode,
{
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        match &self.0 {
            Some(v) => {
                e.array(1)?;
                v.encode(e)
            }
            None => e.array(0),
        }
    }
}

impl<'a, T: Decode<'a>> Decode<'a> for ArrayOpt<T> {
    type Error = fixed::Error<T::Error>;

    fn decode(d: &mut tinycbor::Decoder<'a>) -> Result<Self, Self::Error> {
        let mut visitor = d.array_visitor().map_err(collections::Error::Malformed)?;
        let ret = visitor
            .visit()
            .transpose()
            .map_err(collections::Error::Element)?;
        if visitor.remaining() != Some(0) {
            return Err(fixed::Error::Surplus);
        }
        Ok(ArrayOpt(ret))
    }
}

impl<T: CborLen> CborLen for ArrayOpt<T> {
    fn cbor_len(&self) -> usize {
        match &self.0 {
            Some(v) => 1 + v.cbor_len(),
            None => 1,
        }
    }
}
