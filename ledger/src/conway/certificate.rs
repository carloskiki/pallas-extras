use tinycbor::{
    CborLen, Decode, Decoder, Encode,
    container::{self, bounded},
    tag,
};

use crate::{
    conway::{
        governance::{self, Anchor},
        pool,
    },
    crypto::{Blake2b224Digest, Blake2b256Digest},
    epoch, interval,
    shelley::{self, Credential, address::Account, transaction::Coin},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Certificate<'a> {
    /// Certificate for Delegation and/or Registration.
    ///
    /// Since the [`Era::Conway`] era, registration & delegation can be done at the same time, so this
    /// variant supports these options separately and at the same time.
    AccountAction {
        credential: Credential<'a>,
        pool: Option<&'a shelley::pool::Id>,
        delegate_representative: Option<governance::DelegateRepresentative<'a>>,
        deposit: Option<Coin>,
    },
    AccountUnregistration {
        credential: Credential<'a>,
        deposit: Option<Coin>,
    },
    PoolRegistration {
        operator: &'a Blake2b224Digest,
        vrf_keyhash: &'a Blake2b256Digest,
        pledge: Coin,
        cost: Coin,
        margin: interval::Unit,
        account: Account<'a>,
        owners: Vec<&'a Blake2b224Digest>,
        relays: Vec<pool::Relay<'a>>,
        metadata: Option<pool::Metadata<'a>>,
    },
    PoolRetirement {
        pool: &'a shelley::pool::Id,
        epoch: epoch::Number,
    },
    ConstitutionalCommitteeAuthorization {
        issuer: Credential<'a>,
        hot_credential: Credential<'a>,
    },
    ConstitutionalCommitteeResignation {
        credential: Credential<'a>,
        anchor: Option<Anchor<'a>>,
    },
    DelegateRepresentativeRegistration {
        credential: Credential<'a>,
        deposit: Coin,
        anchor: Option<Anchor<'a>>,
    },
    DelegateRepresentativeUnregistration {
        credential: Credential<'a>,
        deposit: Coin,
    },
    DelegateRepresentativeUpdate {
        credential: Credential<'a>,
        anchor: Option<Anchor<'a>>,
    },
}

#[derive(Debug, thiserror::Error, displaydoc::Display)]
/// while decoding `Certificate`
pub enum Error {
    /// while decoding field `credential` in variant `AccountAction`
    AccountActionCredential(#[source] <Credential<'static> as Decode<'static>>::Error),
    /// while decoding field `pool` in variant `AccountAction`
    AccountActionPool(#[source] <&'static Blake2b224Digest as Decode<'static>>::Error),
    /// while decoding field `delegate_representative` in variant `AccountAction`
    AccountActionDelegateRepresentative(
        #[source] <governance::DelegateRepresentative<'static> as Decode<'static>>::Error,
    ),
    /// while decoding field `deposit` in variant `AccountAction`
    AccountActionDeposit(#[source] <Coin as Decode<'static>>::Error),
    /// while decoding field `credential` in variant `AccountUnregistration`
    AccountUnregistrationCredential(#[source] <Credential<'static> as Decode<'static>>::Error),
    /// while decoding field `deposit` in variant `AccountUnregistration`
    AccountUnregistrationDeposit(#[source] <Coin as Decode<'static>>::Error),
    /// while decoding field `operator` in variant `PoolRegistration`
    PoolRegistrationOperator(#[source] <&'static Blake2b224Digest as Decode<'static>>::Error),
    /// while decoding field `vrf_keyhash` in variant `PoolRegistration`
    PoolRegistrationVrfKeyHash(#[source] <&'static Blake2b256Digest as Decode<'static>>::Error),
    /// while decoding field `pledge` in variant `PoolRegistration`
    PoolRegistrationPledge(#[source] <Coin as Decode<'static>>::Error),
    /// while decoding field `cost` in variant `PoolRegistration`
    PoolRegistrationCost(#[source] <Coin as Decode<'static>>::Error),
    /// while decoding field `margin` in variant `PoolRegistration`
    PoolRegistrationMargin(#[source] <interval::Unit as Decode<'static>>::Error),
    /// while decoding field `account` in variant `PoolRegistration`
    PoolRegistrationAccount(#[source] <Account<'static> as Decode<'static>>::Error),
    /// while decoding field `owners` in variant `PoolRegistration`
    PoolRegistrationOwners(#[source] <Vec<&'static Blake2b224Digest> as Decode<'static>>::Error),
    /// while decoding field `relays` in variant `PoolRegistration`
    PoolRegistrationRelays(#[source] <Vec<pool::Relay<'static>> as Decode<'static>>::Error),
    /// while decoding field `metadata` in variant `PoolRegistration`
    PoolRegistrationMetadata(#[source] <Option<pool::Metadata<'static>> as Decode<'static>>::Error),
    /// while decoding field `pool` in variant `PoolRetirement`
    PoolRetirementPool(#[source] <&'static shelley::pool::Id as Decode<'static>>::Error),
    /// while decoding field `epoch` in variant `PoolRetirement`
    PoolRetirementEpoch(#[source] <epoch::Number as Decode<'static>>::Error),
    /// while decoding field `issuer` in variant `ConstitutionalCommitteeAuthorization`
    ConstitutionalCommitteeAuthorizationIssuer(
        #[source] <Credential<'static> as Decode<'static>>::Error,
    ),
    /// while decoding field `hot_credential` in variant `ConstitutionalCommitteeAuthorization`
    ConstitutionalCommitteeAuthorizationHotCredential(
        #[source] <Credential<'static> as Decode<'static>>::Error,
    ),
    /// while decoding field `credential` in variant `ConstitutionalCommitteeResignation`
    ConstitutionalCommitteeResignationCredential(
        #[source] <Credential<'static> as Decode<'static>>::Error,
    ),
    /// while decoding field `anchor` in variant `ConstitutionalCommitteeResignation`
    ConstitutionalCommitteeResignationAnchor(
        #[source] <Option<Anchor<'static>> as Decode<'static>>::Error,
    ),
    /// while decoding field `credential` in variant `DelegateRepresentativeRegistration`
    DelegateRepresentativeRegistrationCredential(
        #[source] <Credential<'static> as Decode<'static>>::Error,
    ),
    /// while decoding field `deposit` in variant `DelegateRepresentativeRegistration`
    DelegateRepresentativeRegistrationDeposit(#[source] <Coin as Decode<'static>>::Error),
    /// while decoding field `anchor` in variant `DelegateRepresentativeRegistration`
    DelegateRepresentativeRegistrationAnchor(
        #[source] <Option<Anchor<'static>> as Decode<'static>>::Error,
    ),
    /// while decoding field `credential` in variant `DelegateRepresentativeUnregistration`
    DelegateRepresentativeUnregistrationCredential(
        #[source] <Credential<'static> as Decode<'static>>::Error,
    ),
    /// while decoding field `deposit` in variant `DelegateRepresentativeUnregistration`
    DelegateRepresentativeUnregistrationDeposit(#[source] <Coin as Decode<'static>>::Error),
    /// while decoding field `credential` in variant `DelegateRepresentativeUpdate`
    DelegateRepresentativeUpdateCredential(
        #[source] <Credential<'static> as Decode<'static>>::Error,
    ),
    /// while decoding field `anchor` in variant `DelegateRepresentativeUpdate`
    DelegateRepresentativeUpdateAnchor(
        #[source] <Option<Anchor<'static>> as Decode<'static>>::Error,
    ),
}

const ARRAY_LENGTHS: [usize; 19] = [2, 2, 3, 10, 3, 4, 2, 3, 3, 3, 4, 4, 4, 5, 3, 3, 4, 3, 3];

impl Certificate<'_> {
    fn tag_len(&self) -> (usize, usize) {
        match self {
            Certificate::AccountAction {
                deposit,
                pool: pool_keyhash,
                delegate_representative,
                ..
            } => match (deposit, pool_keyhash, delegate_representative) {
                (None, None, None) => (0, 2),
                (None, None, Some(_)) => (9, 3),
                (None, Some(_), None) => (2, 3),
                (None, Some(_), Some(_)) => (10, 4),
                (Some(_), None, None) => (7, 3),
                (Some(_), None, Some(_)) => (12, 4),
                (Some(_), Some(_), None) => (11, 4),
                (Some(_), Some(_), Some(_)) => (13, 5),
            },
            Certificate::AccountUnregistration { deposit, .. } => match deposit {
                Some(_) => (8, 3),
                None => (1, 2),
            },
            Certificate::PoolRegistration { .. } => (3, 10),
            Certificate::PoolRetirement { .. } => (4, 3),
            Certificate::ConstitutionalCommitteeAuthorization { .. } => (14, 3),
            Certificate::ConstitutionalCommitteeResignation { .. } => (15, 3),
            Certificate::DelegateRepresentativeRegistration { .. } => (16, 4),
            Certificate::DelegateRepresentativeUnregistration { .. } => (17, 3),
            Certificate::DelegateRepresentativeUpdate { .. } => (18, 3),
        }
    }
}

impl Encode for Certificate<'_> {
    fn encode<W: tinycbor::Write>(&self, e: &mut tinycbor::Encoder<W>) -> Result<(), W::Error> {
        let (tag, len) = self.tag_len();
        e.array(len)?;
        tag.encode(e)?;

        match self {
            Certificate::AccountAction {
                credential,
                deposit,
                pool: pool_keyhash,
                delegate_representative,
            } => {
                credential.encode(e)?;
                if let Some(pool_keyhash) = pool_keyhash {
                    pool_keyhash.encode(e)?;
                }
                if let Some(delegate_representative) = delegate_representative {
                    delegate_representative.encode(e)?;
                }
                if let Some(deposit) = deposit {
                    deposit.encode(e)?;
                }
                Ok(())
            }
            Certificate::AccountUnregistration {
                credential,
                deposit,
            } => {
                credential.encode(e)?;
                if let Some(deposit) = deposit {
                    deposit.encode(e)?;
                }
                Ok(())
            }
            Certificate::PoolRegistration {
                operator,
                vrf_keyhash,
                pledge,
                cost,
                margin,
                account,
                owners,
                relays,
                metadata,
            } => {
                operator.encode(e)?;
                vrf_keyhash.encode(e)?;
                pledge.encode(e)?;
                cost.encode(e)?;
                margin.encode(e)?;
                account.encode(e)?;
                owners.encode(e)?;
                relays.encode(e)?;
                metadata.encode(e)
            }
            Certificate::PoolRetirement { pool, epoch } => {
                pool.encode(e)?;
                epoch.encode(e)
            }
            Certificate::ConstitutionalCommitteeAuthorization {
                issuer,
                hot_credential,
            } => {
                issuer.encode(e)?;
                hot_credential.encode(e)
            }
            Certificate::ConstitutionalCommitteeResignation { credential, anchor } => {
                credential.encode(e)?;
                anchor.encode(e)
            }
            Certificate::DelegateRepresentativeRegistration {
                credential,
                deposit,
                anchor,
            } => {
                credential.encode(e)?;
                deposit.encode(e)?;
                anchor.encode(e)
            }
            Certificate::DelegateRepresentativeUnregistration {
                credential,
                deposit,
            } => {
                credential.encode(e)?;
                deposit.encode(e)
            }
            Certificate::DelegateRepresentativeUpdate { credential, anchor } => {
                credential.encode(e)?;
                anchor.encode(e)
            }
        }
    }
}

impl<'a, 'b: 'a> Decode<'b> for Certificate<'a> {
    type Error = container::Error<bounded::Error<tag::Error<Error>>>;

    fn decode(d: &mut Decoder<'b>) -> Result<Self, Self::Error> {
        macro_rules! visit {
            ($visitor:ident, $error_variant:ident) => {
                $visitor
                    .visit()
                    .ok_or(bounded::Error::Missing)?
                    .map_err(|e| {
                        bounded::Error::Content(tag::Error::Content(Error::$error_variant(e)))
                    })
            };
        }
        let mut v = d.array_visitor()?;
        let tag: u64 = v
            .visit()
            .ok_or(bounded::Error::Missing)?
            .map_err(|e| bounded::Error::Content(tag::Error::Malformed(e)))?;

        let certificate = match tag {
            0 => Certificate::AccountAction {
                credential: visit!(v, AccountActionCredential)?,
                deposit: None,
                pool: None,
                delegate_representative: None,
            },
            1 => Certificate::AccountUnregistration {
                credential: visit!(v, AccountUnregistrationCredential)?,
                deposit: None,
            },
            2 => Certificate::AccountAction {
                credential: visit!(v, AccountActionCredential)?,
                deposit: None,
                pool: Some(visit!(v, AccountActionPool)?),
                delegate_representative: None,
            },
            3 => Certificate::PoolRegistration {
                operator: visit!(v, PoolRegistrationOperator)?,
                vrf_keyhash: visit!(v, PoolRegistrationVrfKeyHash)?,
                pledge: visit!(v, PoolRegistrationPledge)?,
                cost: visit!(v, PoolRegistrationCost)?,
                margin: visit!(v, PoolRegistrationMargin)?,
                account: visit!(v, PoolRegistrationAccount)?,
                owners: visit!(v, PoolRegistrationOwners)?,
                relays: visit!(v, PoolRegistrationRelays)?,
                metadata: visit!(v, PoolRegistrationMetadata)?,
            },
            4 => Certificate::PoolRetirement {
                pool: visit!(v, PoolRetirementPool)?,
                epoch: visit!(v, PoolRetirementEpoch)?,
            },
            7 => Certificate::AccountAction {
                credential: visit!(v, AccountActionCredential)?,
                deposit: Some(visit!(v, AccountActionDeposit)?),
                pool: None,
                delegate_representative: None,
            },
            8 => Certificate::AccountUnregistration {
                credential: visit!(v, AccountUnregistrationCredential)?,
                deposit: Some(visit!(v, AccountUnregistrationDeposit)?),
            },
            9 => Certificate::AccountAction {
                credential: visit!(v, AccountActionCredential)?,
                deposit: None,
                pool: None,
                delegate_representative: Some(visit!(v, AccountActionDelegateRepresentative)?),
            },
            10 => Certificate::AccountAction {
                credential: visit!(v, AccountActionCredential)?,
                deposit: None,
                pool: Some(visit!(v, AccountActionPool)?),
                delegate_representative: Some(visit!(v, AccountActionDelegateRepresentative)?),
            },
            11 => Certificate::AccountAction {
                credential: visit!(v, AccountActionCredential)?,
                pool: Some(visit!(v, AccountActionPool)?),
                delegate_representative: None,
                deposit: Some(visit!(v, AccountActionDeposit)?),
            },
            12 => Certificate::AccountAction {
                credential: visit!(v, AccountActionCredential)?,
                pool: None,
                delegate_representative: Some(visit!(v, AccountActionDelegateRepresentative)?),
                deposit: Some(visit!(v, AccountActionDeposit)?),
            },
            13 => Certificate::AccountAction {
                credential: visit!(v, AccountActionCredential)?,
                pool: Some(visit!(v, AccountActionPool)?),
                delegate_representative: Some(visit!(v, AccountActionDelegateRepresentative)?),
                deposit: Some(visit!(v, AccountActionDeposit)?),
            },
            14 => Certificate::ConstitutionalCommitteeAuthorization {
                issuer: visit!(v, ConstitutionalCommitteeAuthorizationIssuer)?,
                hot_credential: visit!(v, ConstitutionalCommitteeAuthorizationHotCredential)?,
            },
            15 => Certificate::ConstitutionalCommitteeResignation {
                credential: visit!(v, ConstitutionalCommitteeResignationCredential)?,
                anchor: visit!(v, ConstitutionalCommitteeResignationAnchor)?,
            },
            16 => Certificate::DelegateRepresentativeRegistration {
                credential: visit!(v, DelegateRepresentativeRegistrationCredential)?,
                deposit: visit!(v, DelegateRepresentativeRegistrationDeposit)?,
                anchor: visit!(v, DelegateRepresentativeRegistrationAnchor)?,
            },
            17 => Certificate::DelegateRepresentativeUnregistration {
                credential: visit!(v, DelegateRepresentativeUnregistrationCredential)?,
                deposit: visit!(v, DelegateRepresentativeUnregistrationDeposit)?,
            },
            18 => Certificate::DelegateRepresentativeUpdate {
                credential: visit!(v, DelegateRepresentativeUpdateCredential)?,
                anchor: visit!(v, DelegateRepresentativeUpdateAnchor)?,
            },
            _ => return Err(bounded::Error::Content(tag::Error::InvalidTag).into()),
        };
        if v.remaining() != Some(0) {
            return Err(bounded::Error::Surplus.into());
        }

        Ok(certificate)
    }
}

impl CborLen for Certificate<'_> {
    fn cbor_len(&self) -> usize {
        let (_, len) = self.tag_len();
        1 + ARRAY_LENGTHS[len]
            + match self {
                Certificate::AccountAction {
                    credential,
                    deposit,
                    pool: pool_keyhash,
                    delegate_representative,
                } => {
                    let mut size = credential.cbor_len();
                    if let Some(pool_keyhash) = pool_keyhash {
                        size += pool_keyhash.cbor_len();
                    }
                    if let Some(delegate_representative) = delegate_representative {
                        size += delegate_representative.cbor_len();
                    }
                    if let Some(deposit) = deposit {
                        size += deposit.cbor_len();
                    }
                    size
                }
                Certificate::AccountUnregistration {
                    credential,
                    deposit,
                } => {
                    let mut size = credential.cbor_len();
                    if let Some(deposit) = deposit {
                        size += deposit.cbor_len();
                    }
                    size
                }
                Certificate::PoolRegistration {
                    operator,
                    vrf_keyhash,
                    pledge,
                    cost,
                    margin,
                    account,
                    owners,
                    relays,
                    metadata,
                } => {
                    operator.cbor_len()
                        + vrf_keyhash.cbor_len()
                        + pledge.cbor_len()
                        + cost.cbor_len()
                        + margin.cbor_len()
                        + account.cbor_len()
                        + owners.cbor_len()
                        + relays.cbor_len()
                        + metadata.cbor_len()
                }
                Certificate::PoolRetirement { pool, epoch } => pool.cbor_len() + epoch.cbor_len(),
                Certificate::ConstitutionalCommitteeAuthorization {
                    issuer,
                    hot_credential,
                } => issuer.cbor_len() + hot_credential.cbor_len(),
                Certificate::ConstitutionalCommitteeResignation { credential, anchor } => {
                    credential.cbor_len() + anchor.cbor_len()
                }
                Certificate::DelegateRepresentativeRegistration {
                    credential,
                    deposit,
                    anchor,
                } => credential.cbor_len() + deposit.cbor_len() + anchor.cbor_len(),
                Certificate::DelegateRepresentativeUnregistration {
                    credential,
                    deposit,
                } => credential.cbor_len() + deposit.cbor_len(),
                Certificate::DelegateRepresentativeUpdate { credential, anchor } => {
                    credential.cbor_len() + anchor.cbor_len()
                }
            }
    }
}
