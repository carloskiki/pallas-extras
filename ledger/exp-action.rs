pub mod action {
    use minicbor::{CborLen, Decode, Encode};
    use crate::{
        Credential, address::shelley::StakeAddress, crypto::Blake2b224Digest, epoch,
        protocol::{self, RealNumber},
        transaction::{self, Coin},
    };
    use super::Constitution;
    #[cbor(flat)]
    pub enum Action {
        #[n(0)]
        ParameterChange {
            #[n(0)]
            id: Option<Id>,
            #[n(1)]
            update: protocol::ParameterUpdate,
            #[cbor(n(2), with = "minicbor::bytes")]
            policy_hash: Option<Blake2b224Digest>,
        },
        #[n(1)]
        HardForkInitialization {
            #[n(0)]
            id: Option<Id>,
            #[n(1)]
            version: protocol::Version,
        },
        #[n(2)]
        TreasuryWithdrawals {
            #[cbor(n(0), with = "cbor_util::list_as_map", has_nil)]
            withdrawals: Box<[(StakeAddress, Coin)]>,
            #[cbor(n(1), with = "minicbor::bytes")]
            policy_hash: Option<Blake2b224Digest>,
        },
        #[n(3)]
        NoConfidence { #[n(0)] id: Option<Id> },
        #[n(4)]
        UpdateCommittee {
            #[n(0)]
            id: Option<Id>,
            #[cbor(n(1), with = "cbor_util::set")]
            remove: Box<[Credential]>,
            #[cbor(n(2), with = "cbor_util::list_as_map")]
            add: Box<[(Credential, epoch::Number)]>,
            #[n(3)]
            signature_threshold: RealNumber,
        },
        #[n(5)]
        NewConstitution { #[n(0)] id: Option<Id>, #[n(1)] constitution: Constitution },
        #[n(6)]
        Info,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Action {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                Action::ParameterChange {
                    id: __self_0,
                    update: __self_1,
                    policy_hash: __self_2,
                } => {
                    ::core::fmt::Formatter::debug_struct_field3_finish(
                        f,
                        "ParameterChange",
                        "id",
                        __self_0,
                        "update",
                        __self_1,
                        "policy_hash",
                        &__self_2,
                    )
                }
                Action::HardForkInitialization { id: __self_0, version: __self_1 } => {
                    ::core::fmt::Formatter::debug_struct_field2_finish(
                        f,
                        "HardForkInitialization",
                        "id",
                        __self_0,
                        "version",
                        &__self_1,
                    )
                }
                Action::TreasuryWithdrawals {
                    withdrawals: __self_0,
                    policy_hash: __self_1,
                } => {
                    ::core::fmt::Formatter::debug_struct_field2_finish(
                        f,
                        "TreasuryWithdrawals",
                        "withdrawals",
                        __self_0,
                        "policy_hash",
                        &__self_1,
                    )
                }
                Action::NoConfidence { id: __self_0 } => {
                    ::core::fmt::Formatter::debug_struct_field1_finish(
                        f,
                        "NoConfidence",
                        "id",
                        &__self_0,
                    )
                }
                Action::UpdateCommittee {
                    id: __self_0,
                    remove: __self_1,
                    add: __self_2,
                    signature_threshold: __self_3,
                } => {
                    ::core::fmt::Formatter::debug_struct_field4_finish(
                        f,
                        "UpdateCommittee",
                        "id",
                        __self_0,
                        "remove",
                        __self_1,
                        "add",
                        __self_2,
                        "signature_threshold",
                        &__self_3,
                    )
                }
                Action::NewConstitution { id: __self_0, constitution: __self_1 } => {
                    ::core::fmt::Formatter::debug_struct_field2_finish(
                        f,
                        "NewConstitution",
                        "id",
                        __self_0,
                        "constitution",
                        &__self_1,
                    )
                }
                Action::Info => ::core::fmt::Formatter::write_str(f, "Info"),
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Action {
        #[inline]
        fn clone(&self) -> Action {
            match self {
                Action::ParameterChange {
                    id: __self_0,
                    update: __self_1,
                    policy_hash: __self_2,
                } => {
                    Action::ParameterChange {
                        id: ::core::clone::Clone::clone(__self_0),
                        update: ::core::clone::Clone::clone(__self_1),
                        policy_hash: ::core::clone::Clone::clone(__self_2),
                    }
                }
                Action::HardForkInitialization { id: __self_0, version: __self_1 } => {
                    Action::HardForkInitialization {
                        id: ::core::clone::Clone::clone(__self_0),
                        version: ::core::clone::Clone::clone(__self_1),
                    }
                }
                Action::TreasuryWithdrawals {
                    withdrawals: __self_0,
                    policy_hash: __self_1,
                } => {
                    Action::TreasuryWithdrawals {
                        withdrawals: ::core::clone::Clone::clone(__self_0),
                        policy_hash: ::core::clone::Clone::clone(__self_1),
                    }
                }
                Action::NoConfidence { id: __self_0 } => {
                    Action::NoConfidence {
                        id: ::core::clone::Clone::clone(__self_0),
                    }
                }
                Action::UpdateCommittee {
                    id: __self_0,
                    remove: __self_1,
                    add: __self_2,
                    signature_threshold: __self_3,
                } => {
                    Action::UpdateCommittee {
                        id: ::core::clone::Clone::clone(__self_0),
                        remove: ::core::clone::Clone::clone(__self_1),
                        add: ::core::clone::Clone::clone(__self_2),
                        signature_threshold: ::core::clone::Clone::clone(__self_3),
                    }
                }
                Action::NewConstitution { id: __self_0, constitution: __self_1 } => {
                    Action::NewConstitution {
                        id: ::core::clone::Clone::clone(__self_0),
                        constitution: ::core::clone::Clone::clone(__self_1),
                    }
                }
                Action::Info => Action::Info,
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Action {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Action {
        #[inline]
        fn eq(&self, other: &Action) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
                && match (self, other) {
                    (
                        Action::ParameterChange {
                            id: __self_0,
                            update: __self_1,
                            policy_hash: __self_2,
                        },
                        Action::ParameterChange {
                            id: __arg1_0,
                            update: __arg1_1,
                            policy_hash: __arg1_2,
                        },
                    ) => {
                        __self_0 == __arg1_0 && __self_1 == __arg1_1
                            && __self_2 == __arg1_2
                    }
                    (
                        Action::HardForkInitialization {
                            id: __self_0,
                            version: __self_1,
                        },
                        Action::HardForkInitialization {
                            id: __arg1_0,
                            version: __arg1_1,
                        },
                    ) => __self_0 == __arg1_0 && __self_1 == __arg1_1,
                    (
                        Action::TreasuryWithdrawals {
                            withdrawals: __self_0,
                            policy_hash: __self_1,
                        },
                        Action::TreasuryWithdrawals {
                            withdrawals: __arg1_0,
                            policy_hash: __arg1_1,
                        },
                    ) => __self_0 == __arg1_0 && __self_1 == __arg1_1,
                    (
                        Action::NoConfidence { id: __self_0 },
                        Action::NoConfidence { id: __arg1_0 },
                    ) => __self_0 == __arg1_0,
                    (
                        Action::UpdateCommittee {
                            id: __self_0,
                            remove: __self_1,
                            add: __self_2,
                            signature_threshold: __self_3,
                        },
                        Action::UpdateCommittee {
                            id: __arg1_0,
                            remove: __arg1_1,
                            add: __arg1_2,
                            signature_threshold: __arg1_3,
                        },
                    ) => {
                        __self_0 == __arg1_0 && __self_1 == __arg1_1
                            && __self_2 == __arg1_2 && __self_3 == __arg1_3
                    }
                    (
                        Action::NewConstitution { id: __self_0, constitution: __self_1 },
                        Action::NewConstitution { id: __arg1_0, constitution: __arg1_1 },
                    ) => __self_0 == __arg1_0 && __self_1 == __arg1_1,
                    _ => true,
                }
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for Action {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<Option<Id>>;
            let _: ::core::cmp::AssertParamIsEq<protocol::ParameterUpdate>;
            let _: ::core::cmp::AssertParamIsEq<Option<Blake2b224Digest>>;
            let _: ::core::cmp::AssertParamIsEq<Option<Id>>;
            let _: ::core::cmp::AssertParamIsEq<protocol::Version>;
            let _: ::core::cmp::AssertParamIsEq<Box<[(StakeAddress, Coin)]>>;
            let _: ::core::cmp::AssertParamIsEq<Option<Blake2b224Digest>>;
            let _: ::core::cmp::AssertParamIsEq<Option<Id>>;
            let _: ::core::cmp::AssertParamIsEq<Option<Id>>;
            let _: ::core::cmp::AssertParamIsEq<Box<[Credential]>>;
            let _: ::core::cmp::AssertParamIsEq<Box<[(Credential, epoch::Number)]>>;
            let _: ::core::cmp::AssertParamIsEq<RealNumber>;
            let _: ::core::cmp::AssertParamIsEq<Option<Id>>;
            let _: ::core::cmp::AssertParamIsEq<Constitution>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for Action {
        #[inline]
        fn partial_cmp(
            &self,
            other: &Action,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            match (self, other) {
                (
                    Action::ParameterChange {
                        id: __self_0,
                        update: __self_1,
                        policy_hash: __self_2,
                    },
                    Action::ParameterChange {
                        id: __arg1_0,
                        update: __arg1_1,
                        policy_hash: __arg1_2,
                    },
                ) => {
                    match ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0) {
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                            match ::core::cmp::PartialOrd::partial_cmp(
                                __self_1,
                                __arg1_1,
                            ) {
                                ::core::option::Option::Some(
                                    ::core::cmp::Ordering::Equal,
                                ) => {
                                    ::core::cmp::PartialOrd::partial_cmp(__self_2, __arg1_2)
                                }
                                cmp => cmp,
                            }
                        }
                        cmp => cmp,
                    }
                }
                (
                    Action::HardForkInitialization { id: __self_0, version: __self_1 },
                    Action::HardForkInitialization { id: __arg1_0, version: __arg1_1 },
                ) => {
                    match ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0) {
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                            ::core::cmp::PartialOrd::partial_cmp(__self_1, __arg1_1)
                        }
                        cmp => cmp,
                    }
                }
                (
                    Action::TreasuryWithdrawals {
                        withdrawals: __self_0,
                        policy_hash: __self_1,
                    },
                    Action::TreasuryWithdrawals {
                        withdrawals: __arg1_0,
                        policy_hash: __arg1_1,
                    },
                ) => {
                    match ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0) {
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                            ::core::cmp::PartialOrd::partial_cmp(__self_1, __arg1_1)
                        }
                        cmp => cmp,
                    }
                }
                (
                    Action::NoConfidence { id: __self_0 },
                    Action::NoConfidence { id: __arg1_0 },
                ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
                (
                    Action::UpdateCommittee {
                        id: __self_0,
                        remove: __self_1,
                        add: __self_2,
                        signature_threshold: __self_3,
                    },
                    Action::UpdateCommittee {
                        id: __arg1_0,
                        remove: __arg1_1,
                        add: __arg1_2,
                        signature_threshold: __arg1_3,
                    },
                ) => {
                    match ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0) {
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                            match ::core::cmp::PartialOrd::partial_cmp(
                                __self_1,
                                __arg1_1,
                            ) {
                                ::core::option::Option::Some(
                                    ::core::cmp::Ordering::Equal,
                                ) => {
                                    match ::core::cmp::PartialOrd::partial_cmp(
                                        __self_2,
                                        __arg1_2,
                                    ) {
                                        ::core::option::Option::Some(
                                            ::core::cmp::Ordering::Equal,
                                        ) => {
                                            ::core::cmp::PartialOrd::partial_cmp(__self_3, __arg1_3)
                                        }
                                        cmp => cmp,
                                    }
                                }
                                cmp => cmp,
                            }
                        }
                        cmp => cmp,
                    }
                }
                (
                    Action::NewConstitution { id: __self_0, constitution: __self_1 },
                    Action::NewConstitution { id: __arg1_0, constitution: __arg1_1 },
                ) => {
                    match ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0) {
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                            ::core::cmp::PartialOrd::partial_cmp(__self_1, __arg1_1)
                        }
                        cmp => cmp,
                    }
                }
                _ => ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr),
            }
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for Action {
        #[inline]
        fn cmp(&self, other: &Action) -> ::core::cmp::Ordering {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
                ::core::cmp::Ordering::Equal => {
                    match (self, other) {
                        (
                            Action::ParameterChange {
                                id: __self_0,
                                update: __self_1,
                                policy_hash: __self_2,
                            },
                            Action::ParameterChange {
                                id: __arg1_0,
                                update: __arg1_1,
                                policy_hash: __arg1_2,
                            },
                        ) => {
                            match ::core::cmp::Ord::cmp(__self_0, __arg1_0) {
                                ::core::cmp::Ordering::Equal => {
                                    match ::core::cmp::Ord::cmp(__self_1, __arg1_1) {
                                        ::core::cmp::Ordering::Equal => {
                                            ::core::cmp::Ord::cmp(__self_2, __arg1_2)
                                        }
                                        cmp => cmp,
                                    }
                                }
                                cmp => cmp,
                            }
                        }
                        (
                            Action::HardForkInitialization {
                                id: __self_0,
                                version: __self_1,
                            },
                            Action::HardForkInitialization {
                                id: __arg1_0,
                                version: __arg1_1,
                            },
                        ) => {
                            match ::core::cmp::Ord::cmp(__self_0, __arg1_0) {
                                ::core::cmp::Ordering::Equal => {
                                    ::core::cmp::Ord::cmp(__self_1, __arg1_1)
                                }
                                cmp => cmp,
                            }
                        }
                        (
                            Action::TreasuryWithdrawals {
                                withdrawals: __self_0,
                                policy_hash: __self_1,
                            },
                            Action::TreasuryWithdrawals {
                                withdrawals: __arg1_0,
                                policy_hash: __arg1_1,
                            },
                        ) => {
                            match ::core::cmp::Ord::cmp(__self_0, __arg1_0) {
                                ::core::cmp::Ordering::Equal => {
                                    ::core::cmp::Ord::cmp(__self_1, __arg1_1)
                                }
                                cmp => cmp,
                            }
                        }
                        (
                            Action::NoConfidence { id: __self_0 },
                            Action::NoConfidence { id: __arg1_0 },
                        ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                        (
                            Action::UpdateCommittee {
                                id: __self_0,
                                remove: __self_1,
                                add: __self_2,
                                signature_threshold: __self_3,
                            },
                            Action::UpdateCommittee {
                                id: __arg1_0,
                                remove: __arg1_1,
                                add: __arg1_2,
                                signature_threshold: __arg1_3,
                            },
                        ) => {
                            match ::core::cmp::Ord::cmp(__self_0, __arg1_0) {
                                ::core::cmp::Ordering::Equal => {
                                    match ::core::cmp::Ord::cmp(__self_1, __arg1_1) {
                                        ::core::cmp::Ordering::Equal => {
                                            match ::core::cmp::Ord::cmp(__self_2, __arg1_2) {
                                                ::core::cmp::Ordering::Equal => {
                                                    ::core::cmp::Ord::cmp(__self_3, __arg1_3)
                                                }
                                                cmp => cmp,
                                            }
                                        }
                                        cmp => cmp,
                                    }
                                }
                                cmp => cmp,
                            }
                        }
                        (
                            Action::NewConstitution {
                                id: __self_0,
                                constitution: __self_1,
                            },
                            Action::NewConstitution {
                                id: __arg1_0,
                                constitution: __arg1_1,
                            },
                        ) => {
                            match ::core::cmp::Ord::cmp(__self_0, __arg1_0) {
                                ::core::cmp::Ordering::Equal => {
                                    ::core::cmp::Ord::cmp(__self_1, __arg1_1)
                                }
                                cmp => cmp,
                            }
                        }
                        _ => ::core::cmp::Ordering::Equal,
                    }
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for Action {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_discr, state);
            match self {
                Action::ParameterChange {
                    id: __self_0,
                    update: __self_1,
                    policy_hash: __self_2,
                } => {
                    ::core::hash::Hash::hash(__self_0, state);
                    ::core::hash::Hash::hash(__self_1, state);
                    ::core::hash::Hash::hash(__self_2, state)
                }
                Action::HardForkInitialization { id: __self_0, version: __self_1 } => {
                    ::core::hash::Hash::hash(__self_0, state);
                    ::core::hash::Hash::hash(__self_1, state)
                }
                Action::TreasuryWithdrawals {
                    withdrawals: __self_0,
                    policy_hash: __self_1,
                } => {
                    ::core::hash::Hash::hash(__self_0, state);
                    ::core::hash::Hash::hash(__self_1, state)
                }
                Action::NoConfidence { id: __self_0 } => {
                    ::core::hash::Hash::hash(__self_0, state)
                }
                Action::UpdateCommittee {
                    id: __self_0,
                    remove: __self_1,
                    add: __self_2,
                    signature_threshold: __self_3,
                } => {
                    ::core::hash::Hash::hash(__self_0, state);
                    ::core::hash::Hash::hash(__self_1, state);
                    ::core::hash::Hash::hash(__self_2, state);
                    ::core::hash::Hash::hash(__self_3, state)
                }
                Action::NewConstitution { id: __self_0, constitution: __self_1 } => {
                    ::core::hash::Hash::hash(__self_0, state);
                    ::core::hash::Hash::hash(__self_1, state)
                }
                _ => {}
            }
        }
    }
    impl<Ctx> minicbor::Encode<Ctx> for Action {
        fn encode<__W777>(
            &self,
            __e777: &mut minicbor::Encoder<__W777>,
            __ctx777: &mut Ctx,
        ) -> core::result::Result<(), minicbor::encode::Error<__W777::Error>>
        where
            __W777: minicbor::encode::Write,
        {
            match self {
                Action::ParameterChange { id, update, policy_hash, .. } => {
                    let mut __max_index777: core::option::Option<u64> = None;
                    if !minicbor::Encode::<Ctx>::is_nil(&id) {
                        __max_index777 = Some(0u64);
                    }
                    if !minicbor::Encode::<Ctx>::is_nil(&update) {
                        __max_index777 = Some(1u64);
                    }
                    if !core::option::Option::is_none(&policy_hash) {
                        __max_index777 = Some(2u64);
                    }
                    if let Some(__i777) = __max_index777 {
                        __e777.array(__i777 + 2)?;
                    } else {
                        __e777.array(1)?;
                    }
                    __e777.i64(0)?;
                    if let Some(__i777) = __max_index777 {
                        if 0 <= __i777 {
                            minicbor::Encode::encode(id, __e777, __ctx777)?
                        }
                        if 1 <= __i777 {
                            minicbor::Encode::encode(update, __e777, __ctx777)?
                        }
                        if 2 <= __i777 {
                            minicbor::bytes::encode(policy_hash, __e777, __ctx777)?
                        }
                    }
                    Ok(())
                }
                Action::HardForkInitialization { id, version, .. } => {
                    let mut __max_index777: core::option::Option<u64> = None;
                    if !minicbor::Encode::<Ctx>::is_nil(&id) {
                        __max_index777 = Some(0u64);
                    }
                    if !minicbor::Encode::<Ctx>::is_nil(&version) {
                        __max_index777 = Some(1u64);
                    }
                    if let Some(__i777) = __max_index777 {
                        __e777.array(__i777 + 2)?;
                    } else {
                        __e777.array(1)?;
                    }
                    __e777.i64(1)?;
                    if let Some(__i777) = __max_index777 {
                        if 0 <= __i777 {
                            minicbor::Encode::encode(id, __e777, __ctx777)?
                        }
                        if 1 <= __i777 {
                            minicbor::Encode::encode(version, __e777, __ctx777)?
                        }
                    }
                    Ok(())
                }
                Action::TreasuryWithdrawals { withdrawals, policy_hash, .. } => {
                    let mut __max_index777: core::option::Option<u64> = None;
                    if !cbor_util::list_as_map::is_nil(&withdrawals) {
                        __max_index777 = Some(0u64);
                    }
                    if !core::option::Option::is_none(&policy_hash) {
                        __max_index777 = Some(1u64);
                    }
                    if let Some(__i777) = __max_index777 {
                        __e777.array(__i777 + 2)?;
                    } else {
                        __e777.array(1)?;
                    }
                    __e777.i64(2)?;
                    if let Some(__i777) = __max_index777 {
                        if 0 <= __i777 {
                            cbor_util::list_as_map::encode(
                                withdrawals,
                                __e777,
                                __ctx777,
                            )?
                        }
                        if 1 <= __i777 {
                            minicbor::bytes::encode(policy_hash, __e777, __ctx777)?
                        }
                    }
                    Ok(())
                }
                Action::NoConfidence { id, .. } => {
                    let mut __max_index777: core::option::Option<u64> = None;
                    if !minicbor::Encode::<Ctx>::is_nil(&id) {
                        __max_index777 = Some(0u64);
                    }
                    if let Some(__i777) = __max_index777 {
                        __e777.array(__i777 + 2)?;
                    } else {
                        __e777.array(1)?;
                    }
                    __e777.i64(3)?;
                    if let Some(__i777) = __max_index777 {
                        if 0 <= __i777 {
                            minicbor::Encode::encode(id, __e777, __ctx777)?
                        }
                    }
                    Ok(())
                }
                Action::UpdateCommittee { id, remove, add, signature_threshold, .. } => {
                    let mut __max_index777: core::option::Option<u64> = None;
                    if !minicbor::Encode::<Ctx>::is_nil(&id) {
                        __max_index777 = Some(0u64);
                    }
                    if !(|_| false)(&remove) {
                        __max_index777 = Some(1u64);
                    }
                    if !(|_| false)(&add) {
                        __max_index777 = Some(2u64);
                    }
                    if !minicbor::Encode::<Ctx>::is_nil(&signature_threshold) {
                        __max_index777 = Some(3u64);
                    }
                    if let Some(__i777) = __max_index777 {
                        __e777.array(__i777 + 2)?;
                    } else {
                        __e777.array(1)?;
                    }
                    __e777.i64(4)?;
                    if let Some(__i777) = __max_index777 {
                        if 0 <= __i777 {
                            minicbor::Encode::encode(id, __e777, __ctx777)?
                        }
                        if 1 <= __i777 {
                            cbor_util::set::encode(remove, __e777, __ctx777)?
                        }
                        if 2 <= __i777 {
                            cbor_util::list_as_map::encode(add, __e777, __ctx777)?
                        }
                        if 3 <= __i777 {
                            minicbor::Encode::encode(
                                signature_threshold,
                                __e777,
                                __ctx777,
                            )?
                        }
                    }
                    Ok(())
                }
                Action::NewConstitution { id, constitution, .. } => {
                    let mut __max_index777: core::option::Option<u64> = None;
                    if !minicbor::Encode::<Ctx>::is_nil(&id) {
                        __max_index777 = Some(0u64);
                    }
                    if !minicbor::Encode::<Ctx>::is_nil(&constitution) {
                        __max_index777 = Some(1u64);
                    }
                    if let Some(__i777) = __max_index777 {
                        __e777.array(__i777 + 2)?;
                    } else {
                        __e777.array(1)?;
                    }
                    __e777.i64(5)?;
                    if let Some(__i777) = __max_index777 {
                        if 0 <= __i777 {
                            minicbor::Encode::encode(id, __e777, __ctx777)?
                        }
                        if 1 <= __i777 {
                            minicbor::Encode::encode(constitution, __e777, __ctx777)?
                        }
                    }
                    Ok(())
                }
                Action::Info => {
                    __e777.array(1)?;
                    __e777.i64(6)?;
                    Ok(())
                }
            }
        }
    }
    impl<'bytes, Ctx> minicbor::Decode<'bytes, Ctx> for Action {
        fn decode(
            __d777: &mut minicbor::Decoder<'bytes>,
            __ctx777: &mut Ctx,
        ) -> core::result::Result<Action, minicbor::decode::Error> {
            let __p777 = __d777.position();
            let Some(__len777) = __d777.array()? else {
                return Err(
                    minicbor::decode::Error::message(
                            "flat enum requires definite-length array",
                        )
                        .at(__p777),
                )
            };
            if __len777 == 0 {
                return Err(
                    minicbor::decode::Error::message(
                            "flat enum requires non-empty array",
                        )
                        .at(__p777),
                );
            }
            let __p778 = __d777.position();
            match __d777.i64()? {
                0 => {
                    let mut id: core::option::Option<Option<Id>> = Some(None);
                    let mut update: core::option::Option<protocol::ParameterUpdate> = None;
                    let mut policy_hash: core::option::Option<
                        Option<Blake2b224Digest>,
                    > = Some(None);
                    for __i777 in 0..__len777 - 1 {
                        match __i777 {
                            0 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => id = Some(__v777),
                                    Err(e) if e.is_unknown_variant() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            1 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => update = Some(__v777),
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <protocol::ParameterUpdate as minicbor::Decode<
                                            Ctx,
                                        >>::nil()
                                            .is_some() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            2 => {
                                match minicbor::bytes::decode(__d777, __ctx777) {
                                    Ok(__v777) => policy_hash = Some(__v777),
                                    Err(e) if e.is_unknown_variant() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            _ => __d777.skip()?,
                        }
                    }
                    Ok(Action::ParameterChange {
                        id: if let Some(x) = id {
                            x
                        } else if let Some(def) = None {
                            def
                        } else if let Some(z) = <Option<
                            Id,
                        > as minicbor::Decode<Ctx>>::nil() {
                            z
                        } else {
                            return Err(
                                minicbor::decode::Error::missing_value(0)
                                    .with_message("Action::ParameterChange::id")
                                    .at(__p777),
                            )
                        },
                        update: if let Some(x) = update {
                            x
                        } else if let Some(def) = None {
                            def
                        } else if let Some(z) = <protocol::ParameterUpdate as minicbor::Decode<
                            Ctx,
                        >>::nil() {
                            z
                        } else {
                            return Err(
                                minicbor::decode::Error::missing_value(1)
                                    .with_message("Action::ParameterChange::update")
                                    .at(__p777),
                            )
                        },
                        policy_hash: if let Some(x) = policy_hash {
                            x
                        } else if let Some(def) = None {
                            def
                        } else if let Some(z) = Some(None) {
                            z
                        } else {
                            return Err(
                                minicbor::decode::Error::missing_value(2)
                                    .with_message("Action::ParameterChange::policy_hash")
                                    .at(__p777),
                            )
                        },
                    })
                }
                1 => {
                    let mut id: core::option::Option<Option<Id>> = Some(None);
                    let mut version: core::option::Option<protocol::Version> = None;
                    for __i777 in 0..__len777 - 1 {
                        match __i777 {
                            0 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => id = Some(__v777),
                                    Err(e) if e.is_unknown_variant() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            1 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => version = Some(__v777),
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <protocol::Version as minicbor::Decode<Ctx>>::nil()
                                            .is_some() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            _ => __d777.skip()?,
                        }
                    }
                    Ok(Action::HardForkInitialization {
                        id: if let Some(x) = id {
                            x
                        } else if let Some(def) = None {
                            def
                        } else if let Some(z) = <Option<
                            Id,
                        > as minicbor::Decode<Ctx>>::nil() {
                            z
                        } else {
                            return Err(
                                minicbor::decode::Error::missing_value(0)
                                    .with_message("Action::HardForkInitialization::id")
                                    .at(__p777),
                            )
                        },
                        version: if let Some(x) = version {
                            x
                        } else if let Some(def) = None {
                            def
                        } else if let Some(z) = <protocol::Version as minicbor::Decode<
                            Ctx,
                        >>::nil() {
                            z
                        } else {
                            return Err(
                                minicbor::decode::Error::missing_value(1)
                                    .with_message("Action::HardForkInitialization::version")
                                    .at(__p777),
                            )
                        },
                    })
                }
                2 => {
                    let mut withdrawals: core::option::Option<
                        Box<[(StakeAddress, Coin)]>,
                    > = None;
                    let mut policy_hash: core::option::Option<
                        Option<Blake2b224Digest>,
                    > = Some(None);
                    for __i777 in 0..__len777 - 1 {
                        match __i777 {
                            0 => {
                                match cbor_util::list_as_map::decode(__d777, __ctx777) {
                                    Ok(__v777) => withdrawals = Some(__v777),
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && {
                                            let __nil777: Option<Box<[(StakeAddress, Coin)]>> = cbor_util::list_as_map::nil();
                                            __nil777.is_some()
                                        } => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            1 => {
                                match minicbor::bytes::decode(__d777, __ctx777) {
                                    Ok(__v777) => policy_hash = Some(__v777),
                                    Err(e) if e.is_unknown_variant() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            _ => __d777.skip()?,
                        }
                    }
                    Ok(Action::TreasuryWithdrawals {
                        withdrawals: if let Some(x) = withdrawals {
                            x
                        } else if let Some(def) = None {
                            def
                        } else if let Some(z) = cbor_util::list_as_map::nil() {
                            z
                        } else {
                            return Err(
                                minicbor::decode::Error::missing_value(0)
                                    .with_message("Action::TreasuryWithdrawals::withdrawals")
                                    .at(__p777),
                            )
                        },
                        policy_hash: if let Some(x) = policy_hash {
                            x
                        } else if let Some(def) = None {
                            def
                        } else if let Some(z) = Some(None) {
                            z
                        } else {
                            return Err(
                                minicbor::decode::Error::missing_value(1)
                                    .with_message("Action::TreasuryWithdrawals::policy_hash")
                                    .at(__p777),
                            )
                        },
                    })
                }
                3 => {
                    let mut id: core::option::Option<Option<Id>> = Some(None);
                    for __i777 in 0..__len777 - 1 {
                        match __i777 {
                            0 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => id = Some(__v777),
                                    Err(e) if e.is_unknown_variant() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            _ => __d777.skip()?,
                        }
                    }
                    Ok(Action::NoConfidence {
                        id: if let Some(x) = id {
                            x
                        } else if let Some(def) = None {
                            def
                        } else if let Some(z) = <Option<
                            Id,
                        > as minicbor::Decode<Ctx>>::nil() {
                            z
                        } else {
                            return Err(
                                minicbor::decode::Error::missing_value(0)
                                    .with_message("Action::NoConfidence::id")
                                    .at(__p777),
                            )
                        },
                    })
                }
                4 => {
                    let mut id: core::option::Option<Option<Id>> = Some(None);
                    let mut remove: core::option::Option<Box<[Credential]>> = None;
                    let mut add: core::option::Option<
                        Box<[(Credential, epoch::Number)]>,
                    > = None;
                    let mut signature_threshold: core::option::Option<RealNumber> = None;
                    for __i777 in 0..__len777 - 1 {
                        match __i777 {
                            0 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => id = Some(__v777),
                                    Err(e) if e.is_unknown_variant() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            1 => {
                                match cbor_util::set::decode(__d777, __ctx777) {
                                    Ok(__v777) => remove = Some(__v777),
                                    Err(e) => return Err(e),
                                }
                            }
                            2 => {
                                match cbor_util::list_as_map::decode(__d777, __ctx777) {
                                    Ok(__v777) => add = Some(__v777),
                                    Err(e) => return Err(e),
                                }
                            }
                            3 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => signature_threshold = Some(__v777),
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <RealNumber as minicbor::Decode<Ctx>>::nil()
                                            .is_some() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            _ => __d777.skip()?,
                        }
                    }
                    Ok(Action::UpdateCommittee {
                        id: if let Some(x) = id {
                            x
                        } else if let Some(def) = None {
                            def
                        } else if let Some(z) = <Option<
                            Id,
                        > as minicbor::Decode<Ctx>>::nil() {
                            z
                        } else {
                            return Err(
                                minicbor::decode::Error::missing_value(0)
                                    .with_message("Action::UpdateCommittee::id")
                                    .at(__p777),
                            )
                        },
                        remove: if let Some(x) = remove {
                            x
                        } else if let Some(def) = None {
                            def
                        } else if let Some(z) = None {
                            z
                        } else {
                            return Err(
                                minicbor::decode::Error::missing_value(1)
                                    .with_message("Action::UpdateCommittee::remove")
                                    .at(__p777),
                            )
                        },
                        add: if let Some(x) = add {
                            x
                        } else if let Some(def) = None {
                            def
                        } else if let Some(z) = None {
                            z
                        } else {
                            return Err(
                                minicbor::decode::Error::missing_value(2)
                                    .with_message("Action::UpdateCommittee::add")
                                    .at(__p777),
                            )
                        },
                        signature_threshold: if let Some(x) = signature_threshold {
                            x
                        } else if let Some(def) = None {
                            def
                        } else if let Some(z) = <RealNumber as minicbor::Decode<
                            Ctx,
                        >>::nil() {
                            z
                        } else {
                            return Err(
                                minicbor::decode::Error::missing_value(3)
                                    .with_message(
                                        "Action::UpdateCommittee::signature_threshold",
                                    )
                                    .at(__p777),
                            )
                        },
                    })
                }
                5 => {
                    let mut id: core::option::Option<Option<Id>> = Some(None);
                    let mut constitution: core::option::Option<Constitution> = None;
                    for __i777 in 0..__len777 - 1 {
                        match __i777 {
                            0 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => id = Some(__v777),
                                    Err(e) if e.is_unknown_variant() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            1 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => constitution = Some(__v777),
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <Constitution as minicbor::Decode<Ctx>>::nil()
                                            .is_some() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            _ => __d777.skip()?,
                        }
                    }
                    Ok(Action::NewConstitution {
                        id: if let Some(x) = id {
                            x
                        } else if let Some(def) = None {
                            def
                        } else if let Some(z) = <Option<
                            Id,
                        > as minicbor::Decode<Ctx>>::nil() {
                            z
                        } else {
                            return Err(
                                minicbor::decode::Error::missing_value(0)
                                    .with_message("Action::NewConstitution::id")
                                    .at(__p777),
                            )
                        },
                        constitution: if let Some(x) = constitution {
                            x
                        } else if let Some(def) = None {
                            def
                        } else if let Some(z) = <Constitution as minicbor::Decode<
                            Ctx,
                        >>::nil() {
                            z
                        } else {
                            return Err(
                                minicbor::decode::Error::missing_value(1)
                                    .with_message("Action::NewConstitution::constitution")
                                    .at(__p777),
                            )
                        },
                    })
                }
                6 => Ok(Action::Info),
                n => Err(minicbor::decode::Error::unknown_variant(n).at(__p778)),
            }
        }
    }
    impl<Ctx> minicbor::CborLen<Ctx> for Action {
        fn cbor_len(&self, __ctx777: &mut Ctx) -> usize {
            0
                + {
                    match self {
                        Action::ParameterChange { id, update, policy_hash, .. } => {
                            let mut __num777 = 0;
                            let mut __len777 = 0;
                            if !minicbor::Encode::<Ctx>::is_nil(&id) {
                                __len777
                                    += (0usize - __num777) + 0
                                        + minicbor::CborLen::<Ctx>::cbor_len(&id, __ctx777);
                                __num777 = 0usize + 1;
                            }
                            if !minicbor::Encode::<Ctx>::is_nil(&update) {
                                __len777
                                    += (1usize - __num777) + 0
                                        + minicbor::CborLen::<Ctx>::cbor_len(&update, __ctx777);
                                __num777 = 1usize + 1;
                            }
                            if !core::option::Option::is_none(&policy_hash) {
                                __len777
                                    += (2usize - __num777) + 0
                                        + minicbor::bytes::cbor_len(&policy_hash, __ctx777);
                                __num777 = 2usize + 1;
                            }
                            __num777.cbor_len(__ctx777) + __len777 + 0.cbor_len(__ctx777)
                        }
                        Action::HardForkInitialization { id, version, .. } => {
                            let mut __num777 = 0;
                            let mut __len777 = 0;
                            if !minicbor::Encode::<Ctx>::is_nil(&id) {
                                __len777
                                    += (0usize - __num777) + 0
                                        + minicbor::CborLen::<Ctx>::cbor_len(&id, __ctx777);
                                __num777 = 0usize + 1;
                            }
                            if !minicbor::Encode::<Ctx>::is_nil(&version) {
                                __len777
                                    += (1usize - __num777) + 0
                                        + minicbor::CborLen::<Ctx>::cbor_len(&version, __ctx777);
                                __num777 = 1usize + 1;
                            }
                            __num777.cbor_len(__ctx777) + __len777 + 1.cbor_len(__ctx777)
                        }
                        Action::TreasuryWithdrawals { withdrawals, policy_hash, .. } => {
                            let mut __num777 = 0;
                            let mut __len777 = 0;
                            if !cbor_util::list_as_map::is_nil(&withdrawals) {
                                __len777
                                    += (0usize - __num777) + 0
                                        + cbor_util::list_as_map::cbor_len(&withdrawals, __ctx777);
                                __num777 = 0usize + 1;
                            }
                            if !core::option::Option::is_none(&policy_hash) {
                                __len777
                                    += (1usize - __num777) + 0
                                        + minicbor::bytes::cbor_len(&policy_hash, __ctx777);
                                __num777 = 1usize + 1;
                            }
                            __num777.cbor_len(__ctx777) + __len777 + 2.cbor_len(__ctx777)
                        }
                        Action::NoConfidence { id, .. } => {
                            let mut __num777 = 0;
                            let mut __len777 = 0;
                            if !minicbor::Encode::<Ctx>::is_nil(&id) {
                                __len777
                                    += (0usize - __num777) + 0
                                        + minicbor::CborLen::<Ctx>::cbor_len(&id, __ctx777);
                                __num777 = 0usize + 1;
                            }
                            __num777.cbor_len(__ctx777) + __len777 + 3.cbor_len(__ctx777)
                        }
                        Action::UpdateCommittee {
                            id,
                            remove,
                            add,
                            signature_threshold,
                            ..
                        } => {
                            let mut __num777 = 0;
                            let mut __len777 = 0;
                            if !minicbor::Encode::<Ctx>::is_nil(&id) {
                                __len777
                                    += (0usize - __num777) + 0
                                        + minicbor::CborLen::<Ctx>::cbor_len(&id, __ctx777);
                                __num777 = 0usize + 1;
                            }
                            if !(|_| false)(&remove) {
                                __len777
                                    += (1usize - __num777) + 0
                                        + cbor_util::set::cbor_len(&remove, __ctx777);
                                __num777 = 1usize + 1;
                            }
                            if !(|_| false)(&add) {
                                __len777
                                    += (2usize - __num777) + 0
                                        + cbor_util::list_as_map::cbor_len(&add, __ctx777);
                                __num777 = 2usize + 1;
                            }
                            if !minicbor::Encode::<Ctx>::is_nil(&signature_threshold) {
                                __len777
                                    += (3usize - __num777) + 0
                                        + minicbor::CborLen::<
                                            Ctx,
                                        >::cbor_len(&signature_threshold, __ctx777);
                                __num777 = 3usize + 1;
                            }
                            __num777.cbor_len(__ctx777) + __len777 + 4.cbor_len(__ctx777)
                        }
                        Action::NewConstitution { id, constitution, .. } => {
                            let mut __num777 = 0;
                            let mut __len777 = 0;
                            if !minicbor::Encode::<Ctx>::is_nil(&id) {
                                __len777
                                    += (0usize - __num777) + 0
                                        + minicbor::CborLen::<Ctx>::cbor_len(&id, __ctx777);
                                __num777 = 0usize + 1;
                            }
                            if !minicbor::Encode::<Ctx>::is_nil(&constitution) {
                                __len777
                                    += (1usize - __num777) + 0
                                        + minicbor::CborLen::<
                                            Ctx,
                                        >::cbor_len(&constitution, __ctx777);
                                __num777 = 1usize + 1;
                            }
                            __num777.cbor_len(__ctx777) + __len777 + 5.cbor_len(__ctx777)
                        }
                        Action::Info => 1 + 6.cbor_len(__ctx777),
                    }
                }
        }
    }
    pub struct Id {
        #[cbor(n(0), with = "minicbor::bytes")]
        transaction_id: transaction::Id,
        #[n(1)]
        index: u16,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Id {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "Id",
                "transaction_id",
                &self.transaction_id,
                "index",
                &&self.index,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Id {
        #[inline]
        fn clone(&self) -> Id {
            Id {
                transaction_id: ::core::clone::Clone::clone(&self.transaction_id),
                index: ::core::clone::Clone::clone(&self.index),
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Id {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Id {
        #[inline]
        fn eq(&self, other: &Id) -> bool {
            self.transaction_id == other.transaction_id && self.index == other.index
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for Id {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<transaction::Id>;
            let _: ::core::cmp::AssertParamIsEq<u16>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for Id {
        #[inline]
        fn partial_cmp(
            &self,
            other: &Id,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            match ::core::cmp::PartialOrd::partial_cmp(
                &self.transaction_id,
                &other.transaction_id,
            ) {
                ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                    ::core::cmp::PartialOrd::partial_cmp(&self.index, &other.index)
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for Id {
        #[inline]
        fn cmp(&self, other: &Id) -> ::core::cmp::Ordering {
            match ::core::cmp::Ord::cmp(&self.transaction_id, &other.transaction_id) {
                ::core::cmp::Ordering::Equal => {
                    ::core::cmp::Ord::cmp(&self.index, &other.index)
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for Id {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.transaction_id, state);
            ::core::hash::Hash::hash(&self.index, state)
        }
    }
    impl<Ctx> minicbor::Encode<Ctx> for Id {
        fn encode<__W777>(
            &self,
            __e777: &mut minicbor::Encoder<__W777>,
            __ctx777: &mut Ctx,
        ) -> core::result::Result<(), minicbor::encode::Error<__W777::Error>>
        where
            __W777: minicbor::encode::Write,
        {
            let mut __max_index777: core::option::Option<u64> = None;
            if !(|_| false)(&self.transaction_id) {
                __max_index777 = Some(0u64);
            }
            if !minicbor::Encode::<Ctx>::is_nil(&self.index) {
                __max_index777 = Some(1u64);
            }
            if let Some(__i777) = __max_index777 {
                __e777.array(__i777 + 1)?;
                if 0 <= __i777 {
                    minicbor::bytes::encode(&self.transaction_id, __e777, __ctx777)?
                }
                if 1 <= __i777 {
                    minicbor::Encode::encode(&self.index, __e777, __ctx777)?
                }
            } else {
                __e777.array(0)?;
            }
            Ok(())
        }
    }
    impl<'bytes, Ctx> minicbor::Decode<'bytes, Ctx> for Id {
        fn decode(
            __d777: &mut minicbor::Decoder<'bytes>,
            __ctx777: &mut Ctx,
        ) -> core::result::Result<Id, minicbor::decode::Error> {
            let __p777 = __d777.position();
            let mut transaction_id: core::option::Option<transaction::Id> = None;
            let mut index: core::option::Option<u16> = None;
            if let Some(__len777) = __d777.array()? {
                for __i777 in 0..__len777 {
                    match __i777 {
                        0 => {
                            match minicbor::bytes::decode(__d777, __ctx777) {
                                Ok(__v777) => transaction_id = Some(__v777),
                                Err(e) => return Err(e),
                            }
                        }
                        1 => {
                            match minicbor::Decode::decode(__d777, __ctx777) {
                                Ok(__v777) => index = Some(__v777),
                                Err(
                                    e,
                                ) if e.is_unknown_variant()
                                    && <u16 as minicbor::Decode<Ctx>>::nil().is_some() => {
                                    __d777.skip()?
                                }
                                Err(e) => return Err(e),
                            }
                        }
                        _ => __d777.skip()?,
                    }
                }
            } else {
                let mut __i777 = 0;
                while minicbor::data::Type::Break != __d777.datatype()? {
                    match __i777 {
                        0 => {
                            match minicbor::bytes::decode(__d777, __ctx777) {
                                Ok(__v777) => transaction_id = Some(__v777),
                                Err(e) => return Err(e),
                            }
                        }
                        1 => {
                            match minicbor::Decode::decode(__d777, __ctx777) {
                                Ok(__v777) => index = Some(__v777),
                                Err(
                                    e,
                                ) if e.is_unknown_variant()
                                    && <u16 as minicbor::Decode<Ctx>>::nil().is_some() => {
                                    __d777.skip()?
                                }
                                Err(e) => return Err(e),
                            }
                        }
                        _ => __d777.skip()?,
                    }
                    __i777 += 1;
                }
                __d777.skip()?
            }
            Ok(Id {
                transaction_id: if let Some(x) = transaction_id {
                    x
                } else if let Some(def) = None {
                    def
                } else if let Some(z) = None {
                    z
                } else {
                    return Err(
                        minicbor::decode::Error::missing_value(0)
                            .with_message("Id::transaction_id")
                            .at(__p777),
                    )
                },
                index: if let Some(x) = index {
                    x
                } else if let Some(def) = None {
                    def
                } else if let Some(z) = <u16 as minicbor::Decode<Ctx>>::nil() {
                    z
                } else {
                    return Err(
                        minicbor::decode::Error::missing_value(1)
                            .with_message("Id::index")
                            .at(__p777),
                    )
                },
            })
        }
    }
    impl<Ctx> minicbor::CborLen<Ctx> for Id {
        fn cbor_len(&self, __ctx777: &mut Ctx) -> usize {
            0
                + {
                    let mut __num777 = 0;
                    let mut __len777 = 0;
                    if !(|_| false)(&self.transaction_id) {
                        __len777
                            += (0usize - __num777) + 0
                                + minicbor::bytes::cbor_len(&self.transaction_id, __ctx777);
                        __num777 = 0usize + 1;
                    }
                    if !minicbor::Encode::<Ctx>::is_nil(&self.index) {
                        __len777
                            += (1usize - __num777) + 0
                                + minicbor::CborLen::<Ctx>::cbor_len(&self.index, __ctx777);
                        __num777 = 1usize + 1;
                    }
                    __num777.cbor_len(__ctx777) + __len777
                }
        }
    }
}
