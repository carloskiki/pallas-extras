use minicbor::{CborLen, Decode, Encode};

use crate::crypto::{Blake2b224Digest, Blake2b256Digest};

use super::{address::shelley::StakeAddress, credential, pool, protocol::RealNumber};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(flat)]
pub enum Certificate {
    #[n(0)]
    StakeRegistration {
        #[n(0)]
        stake_credential: credential::Payment,
    },
    #[n(1)]
    StakeDeregistration {
        #[n(0)]
        stake_credential: credential::Payment,
    },
    #[n(2)]
    StakeDelegation {
        #[n(0)]
        stake_credential: credential::Payment,
        #[cbor(n(1), with = "minicbor::bytes")]
        pool_keyhash: Blake2b224Digest,
    },
    #[n(3)]
    PoolRegistration {
        #[cbor(n(0), with = "minicbor::bytes")]
        operator: Blake2b224Digest,
        #[cbor(n(1), with = "minicbor::bytes")]
        vrf_keyhash: Blake2b256Digest,
        #[n(2)]
        pledge: u64,
        #[n(3)]
        cost: u64,
        #[n(4)]
        margin: RealNumber,
        #[n(5)]
        reward_account: StakeAddress,
        #[n(6)]
        #[cbor(with = "cbor_util::boxed_slice::bytes")]
        owners: Box<[Blake2b224Digest]>,
        #[n(7)]
        #[cbor(with = "cbor_util::boxed_slice")]
        relays: Box<[pool::Relay]>,
        #[n(8)]
        metadata: Option<pool::Metadata>,
    },
    #[n(4)]
    PoolRetirement {
        #[cbor(n(0), with = "minicbor::bytes")]
        pool_keyhash: Blake2b224Digest,
        #[n(1)]
        epoch: u64,
    },
    #[n(5)]
    GenesisKeyDelegation {
        #[cbor(n(0), with = "minicbor::bytes")]
        genesis_hash: Blake2b224Digest,
        #[cbor(n(1), with = "minicbor::bytes")]
        genesis_delegate_hash: Blake2b224Digest,
        #[cbor(n(2), with = "minicbor::bytes")]
        vrf_keyhash: Blake2b256Digest,
    },
    #[n(6)]
    MoveRewards {
        /// If `true`, take the funds from the treasury, otherwise take them from the reserve.
        #[n(0)]
        #[cbor(with = "cbor_util::bool_as_u8")]
        from_treasury: bool,
        #[n(1)]
        to: RewardTarget,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RewardTarget {
    OtherAccountingPot(u64),
    StakeAddresses(Box<[(StakeAddress, u64)]>)
}

impl<C> Encode<C> for RewardTarget {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            RewardTarget::StakeAddresses(v) => {
                e.map(v.len() as u64)?;
                for (address, amount) in v.iter() {
                    e.encode_with(address, ctx)?;
                    e.u64(*amount)?;
                }
            }
            RewardTarget::OtherAccountingPot(amount) => {
                e.u64(*amount)?;
            }
        }
        Ok(())
    }
}

impl<C> Decode<'_, C> for RewardTarget {
    fn decode(d: &mut minicbor::Decoder<'_>, ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        if d.probe().u64().is_err_and(|e| e.is_type_mismatch()) {
            let value: Box<[(StakeAddress, u64)]> = cbor_util::list_as_map::decode(d, ctx)?;
            return Ok(RewardTarget::StakeAddresses(value));
        } else {
            let value = d.u64()?;
            return Ok(RewardTarget::OtherAccountingPot(value));
        }
    }
}

impl<C> CborLen<C> for RewardTarget {
    fn cbor_len(&self, ctx: &mut C) -> usize {
        match self {
            RewardTarget::OtherAccountingPot(v) => v.cbor_len(ctx),
            RewardTarget::StakeAddresses(items) => cbor_util::list_as_map::cbor_len(items, ctx),
        }
    }
}
