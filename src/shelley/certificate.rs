use either::Either;
use minicbor::{Decode, Encode};

use crate::{Blake2b224Digest, Blake2b256Digest};

use super::{RealNumber, address::StakeAddress, credential, pool};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
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
        #[n(1)]
        pool_keyhash: Blake2b224Digest,
    },
    #[n(3)]
    PoolRegistration {
        #[n(0)]
        operator: Blake2b224Digest,
        #[n(1)]
        vrf_keyhash: Blake2b256Digest,
        #[n(2)]
        pledge: u64,
        #[n(3)]
        cost: u64,
        #[n(4)]
        margin: RealNumber,
        #[n(5)]
        reward_account: StakeAddress<false>,
        #[n(6)]
        #[cbor(with = "crate::cbor::boxed_slice")]
        owners: Box<[Blake2b224Digest]>,
        #[n(7)]
        #[cbor(with = "crate::cbor::boxed_slice")]
        relays: Box<[pool::Relay]>,
        #[n(8)]
        metadata: Option<pool::Metadata>,
    },
    #[n(4)]
    PoolRetirement {
        #[n(0)]
        pool_keyhash: Blake2b224Digest,
        #[n(1)]
        epoch: u64,
    },
    #[n(5)]
    GenesisKeyDelegation {
        #[n(0)]
        genesis_hash: Blake2b224Digest,
        #[n(1)]
        genesis_delegate_hash: Blake2b224Digest,
        #[n(2)]
        vrf_keyhash: Blake2b256Digest,
    },
    #[n(6)]
    MoveRewards {
        /// If `true`, take the funds from the treasury, otherwise take them from the reserve.
        #[n(0)]
        #[cbor(with = "crate::cbor::bool_as_u8")]
        from_treasury: bool,
        #[n(1)]
        to: RewardTarget,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RewardTarget(Either<Box<[(StakeAddress<false>, u64)]>, u64>);

impl<C> Encode<C> for RewardTarget {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            RewardTarget(Either::Left(v)) => {
                e.map(v.len() as u64)?;
                for (address, amount) in v.iter() {
                    e.encode_with(address, ctx)?;
                    e.u64(*amount)?;
                }
            }
            RewardTarget(Either::Right(amount)) => {
                e.u64(*amount)?;
            }
        }
        Ok(())
    }
}

impl<C> Decode<'_, C> for RewardTarget {
    fn decode(d: &mut minicbor::Decoder<'_>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
        if d.probe().u64().is_err_and(|e| e.is_type_mismatch()) {
            let value: Result<Box<[(_, _)]>, minicbor::decode::Error> =
                d.map_iter::<StakeAddress<false>, u64>()?.collect();
            return Ok(RewardTarget(Either::Left(value?)));
        } else {
            let value = d.u64()?;
            return Ok(RewardTarget(Either::Right(value)));
        }
    }
}
