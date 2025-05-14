use minicbor::{CborLen, Decode, Encode};

pub mod kind;

use crate::{
    Credential,
    crypto::{Blake2b224Digest, Blake2b256Digest},
    governance,
};

use super::{address::shelley::StakeAddress, pool, protocol::RealNumber};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Certificate {
    /// Certificate for Delegation and/or Registration.
    ///
    /// TODO: link conway to enum.
    /// Since the `Conway` era, registration & delegation can be done at the same time, so this
    /// variant supports these options separately and at the same time.
    Delegation {
        credential: Credential,
        pool_keyhash: Option<Blake2b224Digest>,
        delegate_representative: Option<governance::DelegateRepresentative>,
        deposit: Option<u64>,
    },
    Unregistration {
        credential: Credential,
        deposit: Option<u64>,
    },
    PoolRegistration {
        operator: Blake2b224Digest,
        vrf_keyhash: Blake2b256Digest,
        pledge: u64,
        cost: u64,
        margin: RealNumber,
        reward_account: StakeAddress,
        owners: Box<[Blake2b224Digest]>,
        relays: Box<[pool::Relay]>,
        metadata: Option<pool::Metadata>,
    },
    PoolRetirement {
        pool_keyhash: Blake2b224Digest,
        epoch: u64,
    },
    GenesisKeyDelegation {
        genesis_hash: Blake2b224Digest,
        genesis_delegate_hash: Blake2b224Digest,
        vrf_keyhash: Blake2b256Digest,
    },
    MoveRewards {
        /// If `true`, take the funds from the treasury, otherwise take them from the reserve.
        from_treasury: bool,
        to: RewardTarget,
    },
    ConstitutionalCommittee {
        /// The cold credential, used to authorize a hot credential or resign a position.
        credential: Credential,
        kind: kind::ConstitutionalCommittee,
    },
    DelegateRepresentative {
        credential: Credential,
        kind: kind::DelegateRepresentative,
    },
}

const ARRAY_LENGTHS: [u64; 19] = [2, 2, 3, 10, 3, 4, 3, 3, 3, 3, 4, 4, 4, 5, 3, 3, 4, 3, 3];

impl Certificate {
    fn tag(&self) -> u8 {
        match self {
            Certificate::Delegation {
                deposit,
                pool_keyhash,
                delegate_representative,
                ..
            } => match (deposit, pool_keyhash, delegate_representative) {
                (None, None, None) => 0,
                (None, None, Some(_)) => 9,
                (None, Some(_), None) => 2,
                (None, Some(_), Some(_)) => 10,
                (Some(_), None, None) => 7,
                (Some(_), None, Some(_)) => 12,
                (Some(_), Some(_), None) => 11,
                (Some(_), Some(_), Some(_)) => 13,
            },
            Certificate::Unregistration { deposit, .. } => match deposit {
                Some(_) => 8,
                None => 1,
            },
            Certificate::PoolRegistration { .. } => 3,
            Certificate::PoolRetirement { .. } => 4,
            Certificate::GenesisKeyDelegation { .. } => 5,
            Certificate::MoveRewards { .. } => 6,
            Certificate::ConstitutionalCommittee { kind, .. } => match kind {
                kind::ConstitutionalCommittee::Authorize(_) => 14,
                kind::ConstitutionalCommittee::Resign(_) => 15,
            },
            Certificate::DelegateRepresentative { kind, .. } => match kind {
                kind::DelegateRepresentative::Register { .. } => 16,
                kind::DelegateRepresentative::Unregister { .. } => 17,
                kind::DelegateRepresentative::Update { .. } => 18,
            },
        }
    }
}

impl<C> Encode<C> for Certificate {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::Encoder<W>,
        ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        let tag = self.tag();
        e.array(ARRAY_LENGTHS[tag as usize])?.u8(tag)?;

        match self {
            Certificate::Delegation {
                credential,
                deposit,
                pool_keyhash,
                delegate_representative,
            } => {
                e.encode(credential)?;
                if let Some(pool_keyhash) = pool_keyhash {
                    minicbor::bytes::encode(pool_keyhash, e, ctx)?;
                }
                if let Some(delegate_representative) = delegate_representative {
                    e.encode(delegate_representative)?;
                }
                if let Some(deposit) = deposit {
                    e.u64(*deposit)?;
                }
                Ok(())
            }
            Certificate::Unregistration {
                credential,
                deposit,
            } => {
                e.encode(credential)?;
                if let Some(deposit) = deposit {
                    e.u64(*deposit)?;
                }
                Ok(())
            }
            Certificate::PoolRegistration {
                operator,
                vrf_keyhash,
                pledge,
                cost,
                margin,
                reward_account,
                owners,
                relays,
                metadata,
            } => {
                minicbor::bytes::encode(operator, e, ctx)?;
                minicbor::bytes::encode(vrf_keyhash, e, ctx)?;
                e.u64(*pledge)?;
                e.u64(*cost)?;
                e.encode(margin)?;
                e.encode(reward_account)?;
                cbor_util::boxed_slice::bytes::encode(owners, e, ctx)?;
                e.encode(relays)?;
                e.encode(metadata)?;
                Ok(())
            }
            Certificate::PoolRetirement {
                pool_keyhash,
                epoch,
            } => {
                minicbor::bytes::encode(pool_keyhash, e, ctx)?;
                e.u64(*epoch)?;
                Ok(())
            }
            Certificate::GenesisKeyDelegation {
                genesis_hash,
                genesis_delegate_hash,
                vrf_keyhash,
            } => {
                minicbor::bytes::encode(genesis_hash, e, ctx)?;
                minicbor::bytes::encode(genesis_delegate_hash, e, ctx)?;
                minicbor::bytes::encode(vrf_keyhash, e, ctx)?;
                Ok(())
            }
            Certificate::MoveRewards { from_treasury, to } => {
                cbor_util::bool_as_u8::encode(from_treasury, e, ctx)?;
                e.encode(to)?;
                Ok(())
            }
            Certificate::ConstitutionalCommittee { credential, kind } => {
                e.encode(credential)?;
                match kind {
                    kind::ConstitutionalCommittee::Authorize(credential) => e.encode(credential),
                    kind::ConstitutionalCommittee::Resign(anchor) => e.encode(anchor),
                }?
                .ok()
            }
            Certificate::DelegateRepresentative { credential, kind } => {
                e.encode(credential)?;
                match kind {
                    kind::DelegateRepresentative::Register { deposit, anchor } => {
                        e.u64(*deposit)?.encode(anchor)
                    }
                    kind::DelegateRepresentative::Update { anchor } => e.encode(anchor),
                    kind::DelegateRepresentative::Unregister { deposit } => e.u64(*deposit),
                }?
                .ok()
            }
        }
    }
}

impl<C> Decode<'_, C> for Certificate {
    fn decode(d: &mut minicbor::Decoder<'_>, ctx: &mut C) -> Result<Self, minicbor::decode::Error> {
        let array_len = d.array()?;
        let tag = d.u8()?;
        if array_len.is_some_and(|l| Some(&l) != ARRAY_LENGTHS.get(tag as usize)) {
            return Err(minicbor::decode::Error::message("invalid array length"));
        }

        let certificate = match tag {
            0 => Certificate::Delegation {
                credential: d.decode()?,
                deposit: None,
                pool_keyhash: None,
                delegate_representative: None,
            },
            1 => Certificate::Unregistration {
                credential: d.decode()?,
                deposit: None,
            },
            2 => Certificate::Delegation {
                credential: d.decode()?,
                deposit: None,
                pool_keyhash: minicbor::bytes::decode(d, ctx)?,
                delegate_representative: None,
            },
            3 => Certificate::PoolRegistration {
                operator: minicbor::bytes::decode(d, ctx)?,
                vrf_keyhash: minicbor::bytes::decode(d, ctx)?,
                pledge: d.u64()?,
                cost: d.u64()?,
                margin: d.decode()?,
                reward_account: d.decode()?,
                owners: cbor_util::boxed_slice::bytes::decode(d, ctx)?,
                relays: cbor_util::boxed_slice::decode(d, ctx)?,
                metadata: d.decode()?,
            },
            4 => Certificate::PoolRetirement {
                pool_keyhash: minicbor::bytes::decode(d, ctx)?,
                epoch: d.u64()?,
            },
            5 => Certificate::GenesisKeyDelegation {
                genesis_hash: minicbor::bytes::decode(d, ctx)?,
                genesis_delegate_hash: minicbor::bytes::decode(d, ctx)?,
                vrf_keyhash: minicbor::bytes::decode(d, ctx)?,
            },
            6 => Certificate::MoveRewards {
                from_treasury: cbor_util::bool_as_u8::decode(d, ctx)?,
                to: d.decode()?,
            },
            7 => Certificate::Delegation {
                credential: d.decode()?,
                deposit: Some(d.u64()?),
                pool_keyhash: None,
                delegate_representative: None,
            },
            8 => Certificate::Unregistration {
                credential: d.decode()?,
                deposit: Some(d.u64()?),
            },
            9 => Certificate::Delegation {
                credential: d.decode()?,
                deposit: None,
                pool_keyhash: None,
                delegate_representative: Some(d.decode()?),
            },
            10 => Certificate::Delegation {
                credential: d.decode()?,
                deposit: None,
                pool_keyhash: Some(minicbor::bytes::decode(d, ctx)?),
                delegate_representative: Some(d.decode()?),
            },
            11 => Certificate::Delegation {
                credential: d.decode()?,
                pool_keyhash: Some(minicbor::bytes::decode(d, ctx)?),
                delegate_representative: None,
                deposit: Some(d.u64()?),
            },
            12 => Certificate::Delegation {
                credential: d.decode()?,
                pool_keyhash: None,
                delegate_representative: Some(d.decode()?),
                deposit: Some(d.u64()?),
            },
            13 => Certificate::Delegation {
                credential: d.decode()?,
                pool_keyhash: Some(minicbor::bytes::decode(d, ctx)?),
                delegate_representative: Some(d.decode()?),
                deposit: Some(d.u64()?),
            },
            14 => Certificate::ConstitutionalCommittee {
                credential: d.decode()?,
                kind: kind::ConstitutionalCommittee::Authorize(d.decode()?),
            },
            15 => Certificate::ConstitutionalCommittee {
                credential: d.decode()?,
                kind: kind::ConstitutionalCommittee::Resign(d.decode()?),
            },
            16 => Certificate::DelegateRepresentative {
                credential: d.decode()?,
                kind: kind::DelegateRepresentative::Register {
                    deposit: d.u64()?,
                    anchor: d.decode()?,
                },
            },
            17 => Certificate::DelegateRepresentative {
                credential: d.decode()?,
                kind: kind::DelegateRepresentative::Unregister { deposit: d.u64()? },
            },
            18 => Certificate::DelegateRepresentative {
                credential: d.decode()?,
                kind: kind::DelegateRepresentative::Update {
                    anchor: d.decode()?,
                },
            },
            _ => return Err(minicbor::decode::Error::message("invalid tag").at(d.position())),
        };

        if array_len.is_none() {
            if d.datatype()? != minicbor::data::Type::Break {
                return Err(minicbor::decode::Error::message("invalid array length"));
            }
            d.skip()?;
        }

        Ok(certificate)
    }
}

impl<C> CborLen<C> for Certificate {
    fn cbor_len(&self, ctx: &mut C) -> usize {
        let tag = self.tag();
        let array_len = ARRAY_LENGTHS[tag as usize];
        array_len.cbor_len(ctx)
            + tag.cbor_len(ctx)
            + (match self {
                Certificate::Delegation {
                    credential,
                    deposit,
                    pool_keyhash,
                    delegate_representative,
                } => {
                    credential.cbor_len(ctx)
                        + pool_keyhash
                            .map(|x| minicbor::bytes::cbor_len(x, ctx))
                            .unwrap_or_default()
                        + delegate_representative
                            .map(|x| x.cbor_len(ctx))
                            .unwrap_or_default()
                        + deposit.map(|x| x.cbor_len(ctx)).unwrap_or_default()
                }
                Certificate::Unregistration {
                    credential,
                    deposit,
                } => {
                    credential.cbor_len(ctx) + deposit.map(|x| x.cbor_len(ctx)).unwrap_or_default()
                }
                Certificate::PoolRegistration {
                    operator,
                    vrf_keyhash,
                    pledge,
                    cost,
                    margin,
                    reward_account,
                    owners,
                    relays,
                    metadata,
                } => {
                    minicbor::bytes::cbor_len(operator, ctx)
                        + minicbor::bytes::cbor_len(vrf_keyhash, ctx)
                        + pledge.cbor_len(ctx)
                        + cost.cbor_len(ctx)
                        + margin.cbor_len(ctx)
                        + reward_account.cbor_len(ctx)
                        + cbor_util::boxed_slice::bytes::cbor_len(owners, ctx)
                        + relays.cbor_len(ctx)
                        + metadata.cbor_len(ctx)
                }
                Certificate::PoolRetirement {
                    pool_keyhash,
                    epoch,
                } => minicbor::bytes::cbor_len(pool_keyhash, ctx) + epoch.cbor_len(ctx),
                Certificate::GenesisKeyDelegation {
                    genesis_hash,
                    genesis_delegate_hash,
                    vrf_keyhash,
                } => {
                    minicbor::bytes::cbor_len(genesis_hash, ctx)
                        + minicbor::bytes::cbor_len(genesis_delegate_hash, ctx)
                        + minicbor::bytes::cbor_len(vrf_keyhash, ctx)
                }
                Certificate::MoveRewards { from_treasury, to } => {
                    cbor_util::bool_as_u8::cbor_len(from_treasury, ctx) + to.cbor_len(ctx)
                }
                Certificate::ConstitutionalCommittee { credential, kind } => {
                    credential.cbor_len(ctx)
                        + match kind {
                            kind::ConstitutionalCommittee::Authorize(credential) => {
                                credential.cbor_len(ctx)
                            }
                            kind::ConstitutionalCommittee::Resign(anchor) => anchor.cbor_len(ctx),
                        }
                }
                Certificate::DelegateRepresentative { credential, kind } => {
                    credential.cbor_len(ctx)
                        + match kind {
                            kind::DelegateRepresentative::Register { deposit, anchor } => {
                                deposit.cbor_len(ctx) + anchor.cbor_len(ctx)
                            }
                            kind::DelegateRepresentative::Update { anchor } => anchor.cbor_len(ctx),
                            kind::DelegateRepresentative::Unregister { deposit } => {
                                deposit.cbor_len(ctx)
                            }
                        }
                }
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RewardTarget {
    OtherAccountingPot(u64),
    StakeAddresses(Box<[(StakeAddress, u64)]>),
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
            Ok(RewardTarget::StakeAddresses(value))
        } else {
            let value = d.u64()?;
            Ok(RewardTarget::OtherAccountingPot(value))
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
