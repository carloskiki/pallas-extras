pub mod governance {
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
            NewConstitution {
                #[n(0)]
                id: Option<Id>,
                #[n(1)]
                constitution: Constitution,
            },
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
                    Action::HardForkInitialization {
                        id: __self_0,
                        version: __self_1,
                    } => {
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
                    Action::HardForkInitialization {
                        id: __self_0,
                        version: __self_1,
                    } => {
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
                            Action::NewConstitution {
                                id: __self_0,
                                constitution: __self_1,
                            },
                            Action::NewConstitution {
                                id: __arg1_0,
                                constitution: __arg1_1,
                            },
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
                            ::core::option::Option::Some(
                                ::core::cmp::Ordering::Equal,
                            ) => {
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
                        Action::HardForkInitialization {
                            id: __self_0,
                            version: __self_1,
                        },
                        Action::HardForkInitialization {
                            id: __arg1_0,
                            version: __arg1_1,
                        },
                    ) => {
                        match ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0) {
                            ::core::option::Option::Some(
                                ::core::cmp::Ordering::Equal,
                            ) => ::core::cmp::PartialOrd::partial_cmp(__self_1, __arg1_1),
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
                            ::core::option::Option::Some(
                                ::core::cmp::Ordering::Equal,
                            ) => ::core::cmp::PartialOrd::partial_cmp(__self_1, __arg1_1),
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
                            ::core::option::Option::Some(
                                ::core::cmp::Ordering::Equal,
                            ) => {
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
                            ::core::option::Option::Some(
                                ::core::cmp::Ordering::Equal,
                            ) => ::core::cmp::PartialOrd::partial_cmp(__self_1, __arg1_1),
                            cmp => cmp,
                        }
                    }
                    _ => {
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
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
                    Action::HardForkInitialization {
                        id: __self_0,
                        version: __self_1,
                    } => {
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
                    Action::UpdateCommittee {
                        id,
                        remove,
                        add,
                        signature_threshold,
                        ..
                    } => {
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
                        let mut update: core::option::Option<
                            protocol::ParameterUpdate,
                        > = None;
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
                                __num777.cbor_len(__ctx777) + __len777
                                    + 0.cbor_len(__ctx777)
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
                                __num777.cbor_len(__ctx777) + __len777
                                    + 1.cbor_len(__ctx777)
                            }
                            Action::TreasuryWithdrawals {
                                withdrawals,
                                policy_hash,
                                ..
                            } => {
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
                                __num777.cbor_len(__ctx777) + __len777
                                    + 2.cbor_len(__ctx777)
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
                                __num777.cbor_len(__ctx777) + __len777
                                    + 3.cbor_len(__ctx777)
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
                                __num777.cbor_len(__ctx777) + __len777
                                    + 4.cbor_len(__ctx777)
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
                                __num777.cbor_len(__ctx777) + __len777
                                    + 5.cbor_len(__ctx777)
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
                match ::core::cmp::Ord::cmp(
                    &self.transaction_id,
                    &other.transaction_id,
                ) {
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
    pub mod voting {
        use minicbor::{CborLen, Decode, Encode};
        use crate::{Credential, crypto::Blake2b224Digest};
        use super::{action, Anchor};
        pub struct Procedure {
            #[n(0)]
            pub vote: Vote,
            #[n(1)]
            pub anchor: Option<Anchor>,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Procedure {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "Procedure",
                    "vote",
                    &self.vote,
                    "anchor",
                    &&self.anchor,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Procedure {
            #[inline]
            fn clone(&self) -> Procedure {
                Procedure {
                    vote: ::core::clone::Clone::clone(&self.vote),
                    anchor: ::core::clone::Clone::clone(&self.anchor),
                }
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Procedure {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Procedure {
            #[inline]
            fn eq(&self, other: &Procedure) -> bool {
                self.vote == other.vote && self.anchor == other.anchor
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for Procedure {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<Vote>;
                let _: ::core::cmp::AssertParamIsEq<Option<Anchor>>;
            }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialOrd for Procedure {
            #[inline]
            fn partial_cmp(
                &self,
                other: &Procedure,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                match ::core::cmp::PartialOrd::partial_cmp(&self.vote, &other.vote) {
                    ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                        ::core::cmp::PartialOrd::partial_cmp(&self.anchor, &other.anchor)
                    }
                    cmp => cmp,
                }
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Ord for Procedure {
            #[inline]
            fn cmp(&self, other: &Procedure) -> ::core::cmp::Ordering {
                match ::core::cmp::Ord::cmp(&self.vote, &other.vote) {
                    ::core::cmp::Ordering::Equal => {
                        ::core::cmp::Ord::cmp(&self.anchor, &other.anchor)
                    }
                    cmp => cmp,
                }
            }
        }
        #[automatically_derived]
        impl ::core::hash::Hash for Procedure {
            #[inline]
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                ::core::hash::Hash::hash(&self.vote, state);
                ::core::hash::Hash::hash(&self.anchor, state)
            }
        }
        impl<Ctx> minicbor::Encode<Ctx> for Procedure {
            fn encode<__W777>(
                &self,
                __e777: &mut minicbor::Encoder<__W777>,
                __ctx777: &mut Ctx,
            ) -> core::result::Result<(), minicbor::encode::Error<__W777::Error>>
            where
                __W777: minicbor::encode::Write,
            {
                let mut __max_index777: core::option::Option<u64> = None;
                if !minicbor::Encode::<Ctx>::is_nil(&self.vote) {
                    __max_index777 = Some(0u64);
                }
                if !minicbor::Encode::<Ctx>::is_nil(&self.anchor) {
                    __max_index777 = Some(1u64);
                }
                if let Some(__i777) = __max_index777 {
                    __e777.array(__i777 + 1)?;
                    if 0 <= __i777 {
                        minicbor::Encode::encode(&self.vote, __e777, __ctx777)?
                    }
                    if 1 <= __i777 {
                        minicbor::Encode::encode(&self.anchor, __e777, __ctx777)?
                    }
                } else {
                    __e777.array(0)?;
                }
                Ok(())
            }
        }
        impl<'bytes, Ctx> minicbor::Decode<'bytes, Ctx> for Procedure {
            fn decode(
                __d777: &mut minicbor::Decoder<'bytes>,
                __ctx777: &mut Ctx,
            ) -> core::result::Result<Procedure, minicbor::decode::Error> {
                let __p777 = __d777.position();
                let mut vote: core::option::Option<Vote> = None;
                let mut anchor: core::option::Option<Option<Anchor>> = Some(None);
                if let Some(__len777) = __d777.array()? {
                    for __i777 in 0..__len777 {
                        match __i777 {
                            0 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => vote = Some(__v777),
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <Vote as minicbor::Decode<Ctx>>::nil().is_some() => {
                                        __d777.skip()?
                                    }
                                    Err(e) => return Err(e),
                                }
                            }
                            1 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => anchor = Some(__v777),
                                    Err(e) if e.is_unknown_variant() => __d777.skip()?,
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
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => vote = Some(__v777),
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <Vote as minicbor::Decode<Ctx>>::nil().is_some() => {
                                        __d777.skip()?
                                    }
                                    Err(e) => return Err(e),
                                }
                            }
                            1 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => anchor = Some(__v777),
                                    Err(e) if e.is_unknown_variant() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            _ => __d777.skip()?,
                        }
                        __i777 += 1;
                    }
                    __d777.skip()?
                }
                Ok(Procedure {
                    vote: if let Some(x) = vote {
                        x
                    } else if let Some(def) = None {
                        def
                    } else if let Some(z) = <Vote as minicbor::Decode<Ctx>>::nil() {
                        z
                    } else {
                        return Err(
                            minicbor::decode::Error::missing_value(0)
                                .with_message("Procedure::vote")
                                .at(__p777),
                        )
                    },
                    anchor: if let Some(x) = anchor {
                        x
                    } else if let Some(def) = None {
                        def
                    } else if let Some(z) = <Option<
                        Anchor,
                    > as minicbor::Decode<Ctx>>::nil() {
                        z
                    } else {
                        return Err(
                            minicbor::decode::Error::missing_value(1)
                                .with_message("Procedure::anchor")
                                .at(__p777),
                        )
                    },
                })
            }
        }
        impl<Ctx> minicbor::CborLen<Ctx> for Procedure {
            fn cbor_len(&self, __ctx777: &mut Ctx) -> usize {
                0
                    + {
                        let mut __num777 = 0;
                        let mut __len777 = 0;
                        if !minicbor::Encode::<Ctx>::is_nil(&self.vote) {
                            __len777
                                += (0usize - __num777) + 0
                                    + minicbor::CborLen::<Ctx>::cbor_len(&self.vote, __ctx777);
                            __num777 = 0usize + 1;
                        }
                        if !minicbor::Encode::<Ctx>::is_nil(&self.anchor) {
                            __len777
                                += (1usize - __num777) + 0
                                    + minicbor::CborLen::<
                                        Ctx,
                                    >::cbor_len(&self.anchor, __ctx777);
                            __num777 = 1usize + 1;
                        }
                        __num777.cbor_len(__ctx777) + __len777
                    }
            }
        }
        #[cbor(transparent)]
        pub struct Set(
            #[cbor(with = "cbor_util::list_as_map")]
            pub Box<[(action::Id, Procedure)]>,
        );
        #[automatically_derived]
        impl ::core::fmt::Debug for Set {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Set", &&self.0)
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Set {
            #[inline]
            fn clone(&self) -> Set {
                Set(::core::clone::Clone::clone(&self.0))
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Set {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Set {
            #[inline]
            fn eq(&self, other: &Set) -> bool {
                self.0 == other.0
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for Set {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<Box<[(action::Id, Procedure)]>>;
            }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialOrd for Set {
            #[inline]
            fn partial_cmp(
                &self,
                other: &Set,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                ::core::cmp::PartialOrd::partial_cmp(&self.0, &other.0)
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Ord for Set {
            #[inline]
            fn cmp(&self, other: &Set) -> ::core::cmp::Ordering {
                ::core::cmp::Ord::cmp(&self.0, &other.0)
            }
        }
        #[automatically_derived]
        impl ::core::hash::Hash for Set {
            #[inline]
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                ::core::hash::Hash::hash(&self.0, state)
            }
        }
        impl<Ctx> minicbor::Encode<Ctx> for Set {
            fn encode<__W777>(
                &self,
                __e777: &mut minicbor::Encoder<__W777>,
                __ctx777: &mut Ctx,
            ) -> core::result::Result<(), minicbor::encode::Error<__W777::Error>>
            where
                __W777: minicbor::encode::Write,
            {
                cbor_util::list_as_map::encode(&self.0, __e777, __ctx777)
            }
        }
        impl<'bytes, Ctx> minicbor::Decode<'bytes, Ctx> for Set {
            fn decode(
                __d777: &mut minicbor::Decoder<'bytes>,
                __ctx777: &mut Ctx,
            ) -> core::result::Result<Set, minicbor::decode::Error> {
                Ok(Set(cbor_util::list_as_map::decode(__d777, __ctx777)?))
            }
        }
        impl<Ctx> minicbor::CborLen<Ctx> for Set {
            fn cbor_len(&self, __ctx777: &mut Ctx) -> usize {
                cbor_util::list_as_map::cbor_len(&self.0, __ctx777)
            }
        }
        #[cbor(index_only)]
        pub enum Vote {
            #[n(0)]
            No,
            #[n(1)]
            Yes,
            #[n(2)]
            Abstain,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Vote {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        Vote::No => "No",
                        Vote::Yes => "Yes",
                        Vote::Abstain => "Abstain",
                    },
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Vote {
            #[inline]
            fn clone(&self) -> Vote {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for Vote {}
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Vote {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Vote {
            #[inline]
            fn eq(&self, other: &Vote) -> bool {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                __self_discr == __arg1_discr
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for Vote {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {}
        }
        #[automatically_derived]
        impl ::core::cmp::PartialOrd for Vote {
            #[inline]
            fn partial_cmp(
                &self,
                other: &Vote,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Ord for Vote {
            #[inline]
            fn cmp(&self, other: &Vote) -> ::core::cmp::Ordering {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
            }
        }
        #[automatically_derived]
        impl ::core::hash::Hash for Vote {
            #[inline]
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                ::core::hash::Hash::hash(&__self_discr, state)
            }
        }
        impl<Ctx> minicbor::Encode<Ctx> for Vote {
            fn encode<__W777>(
                &self,
                __e777: &mut minicbor::Encoder<__W777>,
                __ctx777: &mut Ctx,
            ) -> core::result::Result<(), minicbor::encode::Error<__W777::Error>>
            where
                __W777: minicbor::encode::Write,
            {
                match self {
                    Vote::No => {
                        __e777.i64(0)?;
                        Ok(())
                    }
                    Vote::Yes => {
                        __e777.i64(1)?;
                        Ok(())
                    }
                    Vote::Abstain => {
                        __e777.i64(2)?;
                        Ok(())
                    }
                }
            }
        }
        impl<'bytes, Ctx> minicbor::Decode<'bytes, Ctx> for Vote {
            fn decode(
                __d777: &mut minicbor::Decoder<'bytes>,
                __ctx777: &mut Ctx,
            ) -> core::result::Result<Vote, minicbor::decode::Error> {
                let __p778 = __d777.position();
                match __d777.i64()? {
                    0 => Ok(Vote::No),
                    1 => Ok(Vote::Yes),
                    2 => Ok(Vote::Abstain),
                    n => Err(minicbor::decode::Error::unknown_variant(n).at(__p778)),
                }
            }
        }
        impl<Ctx> minicbor::CborLen<Ctx> for Vote {
            fn cbor_len(&self, __ctx777: &mut Ctx) -> usize {
                0
                    + {
                        match self {
                            Vote::No => 0.cbor_len(__ctx777),
                            Vote::Yes => 1.cbor_len(__ctx777),
                            Vote::Abstain => 2.cbor_len(__ctx777),
                        }
                    }
            }
        }
        pub enum Voter {
            ConstitutionalCommittee(Credential),
            DelegateRepresentative(Credential),
            StakePool { verifying_key_hash: Blake2b224Digest },
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Voter {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match self {
                    Voter::ConstitutionalCommittee(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "ConstitutionalCommittee",
                            &__self_0,
                        )
                    }
                    Voter::DelegateRepresentative(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "DelegateRepresentative",
                            &__self_0,
                        )
                    }
                    Voter::StakePool { verifying_key_hash: __self_0 } => {
                        ::core::fmt::Formatter::debug_struct_field1_finish(
                            f,
                            "StakePool",
                            "verifying_key_hash",
                            &__self_0,
                        )
                    }
                }
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Voter {
            #[inline]
            fn clone(&self) -> Voter {
                let _: ::core::clone::AssertParamIsClone<Credential>;
                let _: ::core::clone::AssertParamIsClone<Blake2b224Digest>;
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for Voter {}
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for Voter {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for Voter {
            #[inline]
            fn eq(&self, other: &Voter) -> bool {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                __self_discr == __arg1_discr
                    && match (self, other) {
                        (
                            Voter::ConstitutionalCommittee(__self_0),
                            Voter::ConstitutionalCommittee(__arg1_0),
                        ) => __self_0 == __arg1_0,
                        (
                            Voter::DelegateRepresentative(__self_0),
                            Voter::DelegateRepresentative(__arg1_0),
                        ) => __self_0 == __arg1_0,
                        (
                            Voter::StakePool { verifying_key_hash: __self_0 },
                            Voter::StakePool { verifying_key_hash: __arg1_0 },
                        ) => __self_0 == __arg1_0,
                        _ => unsafe { ::core::intrinsics::unreachable() }
                    }
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for Voter {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<Credential>;
                let _: ::core::cmp::AssertParamIsEq<Blake2b224Digest>;
            }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialOrd for Voter {
            #[inline]
            fn partial_cmp(
                &self,
                other: &Voter,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                match (self, other) {
                    (
                        Voter::ConstitutionalCommittee(__self_0),
                        Voter::ConstitutionalCommittee(__arg1_0),
                    ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
                    (
                        Voter::DelegateRepresentative(__self_0),
                        Voter::DelegateRepresentative(__arg1_0),
                    ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
                    (
                        Voter::StakePool { verifying_key_hash: __self_0 },
                        Voter::StakePool { verifying_key_hash: __arg1_0 },
                    ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
                    _ => {
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Ord for Voter {
            #[inline]
            fn cmp(&self, other: &Voter) -> ::core::cmp::Ordering {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
                    ::core::cmp::Ordering::Equal => {
                        match (self, other) {
                            (
                                Voter::ConstitutionalCommittee(__self_0),
                                Voter::ConstitutionalCommittee(__arg1_0),
                            ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                            (
                                Voter::DelegateRepresentative(__self_0),
                                Voter::DelegateRepresentative(__arg1_0),
                            ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                            (
                                Voter::StakePool { verifying_key_hash: __self_0 },
                                Voter::StakePool { verifying_key_hash: __arg1_0 },
                            ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                            _ => unsafe { ::core::intrinsics::unreachable() }
                        }
                    }
                    cmp => cmp,
                }
            }
        }
        #[automatically_derived]
        impl ::core::hash::Hash for Voter {
            #[inline]
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                ::core::hash::Hash::hash(&__self_discr, state);
                match self {
                    Voter::ConstitutionalCommittee(__self_0) => {
                        ::core::hash::Hash::hash(__self_0, state)
                    }
                    Voter::DelegateRepresentative(__self_0) => {
                        ::core::hash::Hash::hash(__self_0, state)
                    }
                    Voter::StakePool { verifying_key_hash: __self_0 } => {
                        ::core::hash::Hash::hash(__self_0, state)
                    }
                }
            }
        }
        impl Voter {
            fn tag(&self) -> u8 {
                match self {
                    Voter::ConstitutionalCommittee(Credential::VerificationKey(_)) => 0,
                    Voter::ConstitutionalCommittee(Credential::Script(_)) => 1,
                    Voter::DelegateRepresentative(Credential::VerificationKey(_)) => 2,
                    Voter::DelegateRepresentative(Credential::Script(_)) => 3,
                    Voter::StakePool { .. } => 4,
                }
            }
            fn bytes(&self) -> &Blake2b224Digest {
                match self {
                    Voter::ConstitutionalCommittee(cred)
                    | Voter::DelegateRepresentative(cred) => cred.as_ref(),
                    Voter::StakePool { verifying_key_hash: h } => h,
                }
            }
        }
        impl<C> Encode<C> for Voter {
            fn encode<W: minicbor::encode::Write>(
                &self,
                e: &mut minicbor::Encoder<W>,
                _: &mut C,
            ) -> Result<(), minicbor::encode::Error<W::Error>> {
                e.array(2)?.u8(self.tag())?.bytes(self.bytes())?.ok()
            }
        }
        impl<C> Decode<'_, C> for Voter {
            fn decode(
                d: &mut minicbor::Decoder<'_>,
                ctx: &mut C,
            ) -> Result<Self, minicbor::decode::Error> {
                cbor_util::array_decode(
                    2,
                    |d| {
                        let tag = d.u8()?;
                        let hash: Blake2b224Digest = minicbor::bytes::decode(d, ctx)?;
                        Ok(
                            match tag {
                                0 => {
                                    Voter::ConstitutionalCommittee(
                                        Credential::VerificationKey(hash),
                                    )
                                }
                                1 => {
                                    Voter::ConstitutionalCommittee(Credential::Script(hash))
                                }
                                2 => {
                                    Voter::DelegateRepresentative(
                                        Credential::VerificationKey(hash),
                                    )
                                }
                                3 => Voter::DelegateRepresentative(Credential::Script(hash)),
                                4 => {
                                    Voter::StakePool {
                                        verifying_key_hash: hash,
                                    }
                                }
                                _ => {
                                    return Err(
                                        minicbor::decode::Error::message("unknown voter tag")
                                            .at(d.position()),
                                    );
                                }
                            },
                        )
                    },
                    d,
                )
            }
        }
        impl<C> CborLen<C> for Voter {
            fn cbor_len(&self, ctx: &mut C) -> usize {
                2.cbor_len(ctx) + self.tag().cbor_len(ctx)
                    + minicbor::bytes::cbor_len(self.bytes(), ctx)
            }
        }
    }
    pub mod delegate_representative {
        use minicbor::{CborLen, Decode, Encode};
        use crate::{protocol::RealNumber, Credential};
        pub enum DelegateRepresentative {
            Credential(Credential),
            Abstain,
            NoConfidence,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for DelegateRepresentative {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match self {
                    DelegateRepresentative::Credential(__self_0) => {
                        ::core::fmt::Formatter::debug_tuple_field1_finish(
                            f,
                            "Credential",
                            &__self_0,
                        )
                    }
                    DelegateRepresentative::Abstain => {
                        ::core::fmt::Formatter::write_str(f, "Abstain")
                    }
                    DelegateRepresentative::NoConfidence => {
                        ::core::fmt::Formatter::write_str(f, "NoConfidence")
                    }
                }
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for DelegateRepresentative {
            #[inline]
            fn clone(&self) -> DelegateRepresentative {
                let _: ::core::clone::AssertParamIsClone<Credential>;
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for DelegateRepresentative {}
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for DelegateRepresentative {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for DelegateRepresentative {
            #[inline]
            fn eq(&self, other: &DelegateRepresentative) -> bool {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                __self_discr == __arg1_discr
                    && match (self, other) {
                        (
                            DelegateRepresentative::Credential(__self_0),
                            DelegateRepresentative::Credential(__arg1_0),
                        ) => __self_0 == __arg1_0,
                        _ => true,
                    }
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for DelegateRepresentative {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<Credential>;
            }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialOrd for DelegateRepresentative {
            #[inline]
            fn partial_cmp(
                &self,
                other: &DelegateRepresentative,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                match (self, other) {
                    (
                        DelegateRepresentative::Credential(__self_0),
                        DelegateRepresentative::Credential(__arg1_0),
                    ) => ::core::cmp::PartialOrd::partial_cmp(__self_0, __arg1_0),
                    _ => {
                        ::core::cmp::PartialOrd::partial_cmp(
                            &__self_discr,
                            &__arg1_discr,
                        )
                    }
                }
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Ord for DelegateRepresentative {
            #[inline]
            fn cmp(&self, other: &DelegateRepresentative) -> ::core::cmp::Ordering {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
                    ::core::cmp::Ordering::Equal => {
                        match (self, other) {
                            (
                                DelegateRepresentative::Credential(__self_0),
                                DelegateRepresentative::Credential(__arg1_0),
                            ) => ::core::cmp::Ord::cmp(__self_0, __arg1_0),
                            _ => ::core::cmp::Ordering::Equal,
                        }
                    }
                    cmp => cmp,
                }
            }
        }
        #[automatically_derived]
        impl ::core::hash::Hash for DelegateRepresentative {
            #[inline]
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                ::core::hash::Hash::hash(&__self_discr, state);
                match self {
                    DelegateRepresentative::Credential(__self_0) => {
                        ::core::hash::Hash::hash(__self_0, state)
                    }
                    _ => {}
                }
            }
        }
        impl DelegateRepresentative {
            fn tag(&self) -> u8 {
                match self {
                    DelegateRepresentative::Credential(
                        Credential::VerificationKey(_),
                    ) => 0,
                    DelegateRepresentative::Credential(Credential::Script(_)) => 1,
                    DelegateRepresentative::Abstain => 2,
                    DelegateRepresentative::NoConfidence => 3,
                }
            }
        }
        impl<C> Encode<C> for DelegateRepresentative {
            fn encode<W: minicbor::encode::Write>(
                &self,
                e: &mut minicbor::Encoder<W>,
                ctx: &mut C,
            ) -> Result<(), minicbor::encode::Error<W::Error>> {
                e.u8(self.tag())?;
                match self {
                    DelegateRepresentative::Credential(
                        Credential::VerificationKey(h) | Credential::Script(h),
                    ) => {
                        e.array(2)?.u8(self.tag())?;
                        minicbor::bytes::encode(h, e, ctx)?;
                    }
                    _ => {
                        e.array(1)?.u8(self.tag())?;
                    }
                }
                Ok(())
            }
        }
        impl<C> Decode<'_, C> for DelegateRepresentative {
            fn decode(
                d: &mut minicbor::Decoder<'_>,
                ctx: &mut C,
            ) -> Result<Self, minicbor::decode::Error> {
                if d.array()?.is_some_and(|l| l != 1 && l != 2) {
                    return Err(
                        minicbor::decode::Error::message("invalid array length")
                            .at(d.position()),
                    );
                }
                let tag = d.u8()?;
                Ok(
                    match tag {
                        0 => {
                            DelegateRepresentative::Credential(
                                Credential::VerificationKey(
                                    minicbor::bytes::decode(d, ctx)?,
                                ),
                            )
                        }
                        1 => {
                            DelegateRepresentative::Credential(
                                Credential::Script(minicbor::bytes::decode(d, ctx)?),
                            )
                        }
                        2 => DelegateRepresentative::Abstain,
                        3 => DelegateRepresentative::NoConfidence,
                        _ => {
                            return Err(
                                minicbor::decode::Error::message("invalid tag")
                                    .at(d.position()),
                            );
                        }
                    },
                )
            }
        }
        impl<C> CborLen<C> for DelegateRepresentative {
            fn cbor_len(&self, ctx: &mut C) -> usize {
                let tag = self.tag();
                tag.cbor_len(ctx)
                    + match self {
                        DelegateRepresentative::Credential(credential) => {
                            minicbor::bytes::cbor_len(credential.as_ref(), ctx)
                        }
                        _ => 0,
                    }
            }
        }
        pub struct VotingThresholds {
            #[n(0)]
            motion_no_confidence: RealNumber,
            #[n(1)]
            update_committee: RealNumber,
            #[n(2)]
            update_committee_no_confidence: RealNumber,
            #[n(3)]
            update_constitution: RealNumber,
            #[n(4)]
            hard_fork_initiation: RealNumber,
            #[n(5)]
            protocol_parameter_network_update: RealNumber,
            #[n(6)]
            protocol_parameter_economic_update: RealNumber,
            #[n(7)]
            protocol_parameter_technical_update: RealNumber,
            #[n(8)]
            protocol_parameter_security_update: RealNumber,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for VotingThresholds {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let names: &'static _ = &[
                    "motion_no_confidence",
                    "update_committee",
                    "update_committee_no_confidence",
                    "update_constitution",
                    "hard_fork_initiation",
                    "protocol_parameter_network_update",
                    "protocol_parameter_economic_update",
                    "protocol_parameter_technical_update",
                    "protocol_parameter_security_update",
                ];
                let values: &[&dyn ::core::fmt::Debug] = &[
                    &self.motion_no_confidence,
                    &self.update_committee,
                    &self.update_committee_no_confidence,
                    &self.update_constitution,
                    &self.hard_fork_initiation,
                    &self.protocol_parameter_network_update,
                    &self.protocol_parameter_economic_update,
                    &self.protocol_parameter_technical_update,
                    &&self.protocol_parameter_security_update,
                ];
                ::core::fmt::Formatter::debug_struct_fields_finish(
                    f,
                    "VotingThresholds",
                    names,
                    values,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for VotingThresholds {
            #[inline]
            fn clone(&self) -> VotingThresholds {
                let _: ::core::clone::AssertParamIsClone<RealNumber>;
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for VotingThresholds {}
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for VotingThresholds {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for VotingThresholds {
            #[inline]
            fn eq(&self, other: &VotingThresholds) -> bool {
                self.motion_no_confidence == other.motion_no_confidence
                    && self.update_committee == other.update_committee
                    && self.update_committee_no_confidence
                        == other.update_committee_no_confidence
                    && self.update_constitution == other.update_constitution
                    && self.hard_fork_initiation == other.hard_fork_initiation
                    && self.protocol_parameter_network_update
                        == other.protocol_parameter_network_update
                    && self.protocol_parameter_economic_update
                        == other.protocol_parameter_economic_update
                    && self.protocol_parameter_technical_update
                        == other.protocol_parameter_technical_update
                    && self.protocol_parameter_security_update
                        == other.protocol_parameter_security_update
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Eq for VotingThresholds {
            #[inline]
            #[doc(hidden)]
            #[coverage(off)]
            fn assert_receiver_is_total_eq(&self) -> () {
                let _: ::core::cmp::AssertParamIsEq<RealNumber>;
            }
        }
        #[automatically_derived]
        impl ::core::cmp::PartialOrd for VotingThresholds {
            #[inline]
            fn partial_cmp(
                &self,
                other: &VotingThresholds,
            ) -> ::core::option::Option<::core::cmp::Ordering> {
                match ::core::cmp::PartialOrd::partial_cmp(
                    &self.motion_no_confidence,
                    &other.motion_no_confidence,
                ) {
                    ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                        match ::core::cmp::PartialOrd::partial_cmp(
                            &self.update_committee,
                            &other.update_committee,
                        ) {
                            ::core::option::Option::Some(
                                ::core::cmp::Ordering::Equal,
                            ) => {
                                match ::core::cmp::PartialOrd::partial_cmp(
                                    &self.update_committee_no_confidence,
                                    &other.update_committee_no_confidence,
                                ) {
                                    ::core::option::Option::Some(
                                        ::core::cmp::Ordering::Equal,
                                    ) => {
                                        match ::core::cmp::PartialOrd::partial_cmp(
                                            &self.update_constitution,
                                            &other.update_constitution,
                                        ) {
                                            ::core::option::Option::Some(
                                                ::core::cmp::Ordering::Equal,
                                            ) => {
                                                match ::core::cmp::PartialOrd::partial_cmp(
                                                    &self.hard_fork_initiation,
                                                    &other.hard_fork_initiation,
                                                ) {
                                                    ::core::option::Option::Some(
                                                        ::core::cmp::Ordering::Equal,
                                                    ) => {
                                                        match ::core::cmp::PartialOrd::partial_cmp(
                                                            &self.protocol_parameter_network_update,
                                                            &other.protocol_parameter_network_update,
                                                        ) {
                                                            ::core::option::Option::Some(
                                                                ::core::cmp::Ordering::Equal,
                                                            ) => {
                                                                match ::core::cmp::PartialOrd::partial_cmp(
                                                                    &self.protocol_parameter_economic_update,
                                                                    &other.protocol_parameter_economic_update,
                                                                ) {
                                                                    ::core::option::Option::Some(
                                                                        ::core::cmp::Ordering::Equal,
                                                                    ) => {
                                                                        match ::core::cmp::PartialOrd::partial_cmp(
                                                                            &self.protocol_parameter_technical_update,
                                                                            &other.protocol_parameter_technical_update,
                                                                        ) {
                                                                            ::core::option::Option::Some(
                                                                                ::core::cmp::Ordering::Equal,
                                                                            ) => {
                                                                                ::core::cmp::PartialOrd::partial_cmp(
                                                                                    &self.protocol_parameter_security_update,
                                                                                    &other.protocol_parameter_security_update,
                                                                                )
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
                                                    cmp => cmp,
                                                }
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
                    cmp => cmp,
                }
            }
        }
        #[automatically_derived]
        impl ::core::cmp::Ord for VotingThresholds {
            #[inline]
            fn cmp(&self, other: &VotingThresholds) -> ::core::cmp::Ordering {
                match ::core::cmp::Ord::cmp(
                    &self.motion_no_confidence,
                    &other.motion_no_confidence,
                ) {
                    ::core::cmp::Ordering::Equal => {
                        match ::core::cmp::Ord::cmp(
                            &self.update_committee,
                            &other.update_committee,
                        ) {
                            ::core::cmp::Ordering::Equal => {
                                match ::core::cmp::Ord::cmp(
                                    &self.update_committee_no_confidence,
                                    &other.update_committee_no_confidence,
                                ) {
                                    ::core::cmp::Ordering::Equal => {
                                        match ::core::cmp::Ord::cmp(
                                            &self.update_constitution,
                                            &other.update_constitution,
                                        ) {
                                            ::core::cmp::Ordering::Equal => {
                                                match ::core::cmp::Ord::cmp(
                                                    &self.hard_fork_initiation,
                                                    &other.hard_fork_initiation,
                                                ) {
                                                    ::core::cmp::Ordering::Equal => {
                                                        match ::core::cmp::Ord::cmp(
                                                            &self.protocol_parameter_network_update,
                                                            &other.protocol_parameter_network_update,
                                                        ) {
                                                            ::core::cmp::Ordering::Equal => {
                                                                match ::core::cmp::Ord::cmp(
                                                                    &self.protocol_parameter_economic_update,
                                                                    &other.protocol_parameter_economic_update,
                                                                ) {
                                                                    ::core::cmp::Ordering::Equal => {
                                                                        match ::core::cmp::Ord::cmp(
                                                                            &self.protocol_parameter_technical_update,
                                                                            &other.protocol_parameter_technical_update,
                                                                        ) {
                                                                            ::core::cmp::Ordering::Equal => {
                                                                                ::core::cmp::Ord::cmp(
                                                                                    &self.protocol_parameter_security_update,
                                                                                    &other.protocol_parameter_security_update,
                                                                                )
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
                                                    cmp => cmp,
                                                }
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
                    cmp => cmp,
                }
            }
        }
        #[automatically_derived]
        impl ::core::hash::Hash for VotingThresholds {
            #[inline]
            fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                ::core::hash::Hash::hash(&self.motion_no_confidence, state);
                ::core::hash::Hash::hash(&self.update_committee, state);
                ::core::hash::Hash::hash(&self.update_committee_no_confidence, state);
                ::core::hash::Hash::hash(&self.update_constitution, state);
                ::core::hash::Hash::hash(&self.hard_fork_initiation, state);
                ::core::hash::Hash::hash(&self.protocol_parameter_network_update, state);
                ::core::hash::Hash::hash(
                    &self.protocol_parameter_economic_update,
                    state,
                );
                ::core::hash::Hash::hash(
                    &self.protocol_parameter_technical_update,
                    state,
                );
                ::core::hash::Hash::hash(&self.protocol_parameter_security_update, state)
            }
        }
        impl<Ctx> minicbor::Encode<Ctx> for VotingThresholds {
            fn encode<__W777>(
                &self,
                __e777: &mut minicbor::Encoder<__W777>,
                __ctx777: &mut Ctx,
            ) -> core::result::Result<(), minicbor::encode::Error<__W777::Error>>
            where
                __W777: minicbor::encode::Write,
            {
                let mut __max_index777: core::option::Option<u64> = None;
                if !minicbor::Encode::<Ctx>::is_nil(&self.motion_no_confidence) {
                    __max_index777 = Some(0u64);
                }
                if !minicbor::Encode::<Ctx>::is_nil(&self.update_committee) {
                    __max_index777 = Some(1u64);
                }
                if !minicbor::Encode::<
                    Ctx,
                >::is_nil(&self.update_committee_no_confidence) {
                    __max_index777 = Some(2u64);
                }
                if !minicbor::Encode::<Ctx>::is_nil(&self.update_constitution) {
                    __max_index777 = Some(3u64);
                }
                if !minicbor::Encode::<Ctx>::is_nil(&self.hard_fork_initiation) {
                    __max_index777 = Some(4u64);
                }
                if !minicbor::Encode::<
                    Ctx,
                >::is_nil(&self.protocol_parameter_network_update) {
                    __max_index777 = Some(5u64);
                }
                if !minicbor::Encode::<
                    Ctx,
                >::is_nil(&self.protocol_parameter_economic_update) {
                    __max_index777 = Some(6u64);
                }
                if !minicbor::Encode::<
                    Ctx,
                >::is_nil(&self.protocol_parameter_technical_update) {
                    __max_index777 = Some(7u64);
                }
                if !minicbor::Encode::<
                    Ctx,
                >::is_nil(&self.protocol_parameter_security_update) {
                    __max_index777 = Some(8u64);
                }
                if let Some(__i777) = __max_index777 {
                    __e777.array(__i777 + 1)?;
                    if 0 <= __i777 {
                        minicbor::Encode::encode(
                            &self.motion_no_confidence,
                            __e777,
                            __ctx777,
                        )?
                    }
                    if 1 <= __i777 {
                        minicbor::Encode::encode(
                            &self.update_committee,
                            __e777,
                            __ctx777,
                        )?
                    }
                    if 2 <= __i777 {
                        minicbor::Encode::encode(
                            &self.update_committee_no_confidence,
                            __e777,
                            __ctx777,
                        )?
                    }
                    if 3 <= __i777 {
                        minicbor::Encode::encode(
                            &self.update_constitution,
                            __e777,
                            __ctx777,
                        )?
                    }
                    if 4 <= __i777 {
                        minicbor::Encode::encode(
                            &self.hard_fork_initiation,
                            __e777,
                            __ctx777,
                        )?
                    }
                    if 5 <= __i777 {
                        minicbor::Encode::encode(
                            &self.protocol_parameter_network_update,
                            __e777,
                            __ctx777,
                        )?
                    }
                    if 6 <= __i777 {
                        minicbor::Encode::encode(
                            &self.protocol_parameter_economic_update,
                            __e777,
                            __ctx777,
                        )?
                    }
                    if 7 <= __i777 {
                        minicbor::Encode::encode(
                            &self.protocol_parameter_technical_update,
                            __e777,
                            __ctx777,
                        )?
                    }
                    if 8 <= __i777 {
                        minicbor::Encode::encode(
                            &self.protocol_parameter_security_update,
                            __e777,
                            __ctx777,
                        )?
                    }
                } else {
                    __e777.array(0)?;
                }
                Ok(())
            }
        }
        impl<'bytes, Ctx> minicbor::Decode<'bytes, Ctx> for VotingThresholds {
            fn decode(
                __d777: &mut minicbor::Decoder<'bytes>,
                __ctx777: &mut Ctx,
            ) -> core::result::Result<VotingThresholds, minicbor::decode::Error> {
                let __p777 = __d777.position();
                let mut motion_no_confidence: core::option::Option<RealNumber> = None;
                let mut update_committee: core::option::Option<RealNumber> = None;
                let mut update_committee_no_confidence: core::option::Option<
                    RealNumber,
                > = None;
                let mut update_constitution: core::option::Option<RealNumber> = None;
                let mut hard_fork_initiation: core::option::Option<RealNumber> = None;
                let mut protocol_parameter_network_update: core::option::Option<
                    RealNumber,
                > = None;
                let mut protocol_parameter_economic_update: core::option::Option<
                    RealNumber,
                > = None;
                let mut protocol_parameter_technical_update: core::option::Option<
                    RealNumber,
                > = None;
                let mut protocol_parameter_security_update: core::option::Option<
                    RealNumber,
                > = None;
                if let Some(__len777) = __d777.array()? {
                    for __i777 in 0..__len777 {
                        match __i777 {
                            0 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => motion_no_confidence = Some(__v777),
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <RealNumber as minicbor::Decode<Ctx>>::nil()
                                            .is_some() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            1 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => update_committee = Some(__v777),
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <RealNumber as minicbor::Decode<Ctx>>::nil()
                                            .is_some() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            2 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => update_committee_no_confidence = Some(__v777),
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <RealNumber as minicbor::Decode<Ctx>>::nil()
                                            .is_some() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            3 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => update_constitution = Some(__v777),
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <RealNumber as minicbor::Decode<Ctx>>::nil()
                                            .is_some() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            4 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => hard_fork_initiation = Some(__v777),
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <RealNumber as minicbor::Decode<Ctx>>::nil()
                                            .is_some() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            5 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => {
                                        protocol_parameter_network_update = Some(__v777);
                                    }
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <RealNumber as minicbor::Decode<Ctx>>::nil()
                                            .is_some() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            6 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => {
                                        protocol_parameter_economic_update = Some(__v777);
                                    }
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <RealNumber as minicbor::Decode<Ctx>>::nil()
                                            .is_some() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            7 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => {
                                        protocol_parameter_technical_update = Some(__v777);
                                    }
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <RealNumber as minicbor::Decode<Ctx>>::nil()
                                            .is_some() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            8 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => {
                                        protocol_parameter_security_update = Some(__v777);
                                    }
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
                } else {
                    let mut __i777 = 0;
                    while minicbor::data::Type::Break != __d777.datatype()? {
                        match __i777 {
                            0 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => motion_no_confidence = Some(__v777),
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <RealNumber as minicbor::Decode<Ctx>>::nil()
                                            .is_some() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            1 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => update_committee = Some(__v777),
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <RealNumber as minicbor::Decode<Ctx>>::nil()
                                            .is_some() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            2 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => update_committee_no_confidence = Some(__v777),
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <RealNumber as minicbor::Decode<Ctx>>::nil()
                                            .is_some() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            3 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => update_constitution = Some(__v777),
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <RealNumber as minicbor::Decode<Ctx>>::nil()
                                            .is_some() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            4 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => hard_fork_initiation = Some(__v777),
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <RealNumber as minicbor::Decode<Ctx>>::nil()
                                            .is_some() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            5 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => {
                                        protocol_parameter_network_update = Some(__v777);
                                    }
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <RealNumber as minicbor::Decode<Ctx>>::nil()
                                            .is_some() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            6 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => {
                                        protocol_parameter_economic_update = Some(__v777);
                                    }
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <RealNumber as minicbor::Decode<Ctx>>::nil()
                                            .is_some() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            7 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => {
                                        protocol_parameter_technical_update = Some(__v777);
                                    }
                                    Err(
                                        e,
                                    ) if e.is_unknown_variant()
                                        && <RealNumber as minicbor::Decode<Ctx>>::nil()
                                            .is_some() => __d777.skip()?,
                                    Err(e) => return Err(e),
                                }
                            }
                            8 => {
                                match minicbor::Decode::decode(__d777, __ctx777) {
                                    Ok(__v777) => {
                                        protocol_parameter_security_update = Some(__v777);
                                    }
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
                        __i777 += 1;
                    }
                    __d777.skip()?
                }
                Ok(VotingThresholds {
                    motion_no_confidence: if let Some(x) = motion_no_confidence {
                        x
                    } else if let Some(def) = None {
                        def
                    } else if let Some(z) = <RealNumber as minicbor::Decode<
                        Ctx,
                    >>::nil() {
                        z
                    } else {
                        return Err(
                            minicbor::decode::Error::missing_value(0)
                                .with_message("VotingThresholds::motion_no_confidence")
                                .at(__p777),
                        )
                    },
                    update_committee: if let Some(x) = update_committee {
                        x
                    } else if let Some(def) = None {
                        def
                    } else if let Some(z) = <RealNumber as minicbor::Decode<
                        Ctx,
                    >>::nil() {
                        z
                    } else {
                        return Err(
                            minicbor::decode::Error::missing_value(1)
                                .with_message("VotingThresholds::update_committee")
                                .at(__p777),
                        )
                    },
                    update_committee_no_confidence: if let Some(x) = update_committee_no_confidence {
                        x
                    } else if let Some(def) = None {
                        def
                    } else if let Some(z) = <RealNumber as minicbor::Decode<
                        Ctx,
                    >>::nil() {
                        z
                    } else {
                        return Err(
                            minicbor::decode::Error::missing_value(2)
                                .with_message(
                                    "VotingThresholds::update_committee_no_confidence",
                                )
                                .at(__p777),
                        )
                    },
                    update_constitution: if let Some(x) = update_constitution {
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
                                .with_message("VotingThresholds::update_constitution")
                                .at(__p777),
                        )
                    },
                    hard_fork_initiation: if let Some(x) = hard_fork_initiation {
                        x
                    } else if let Some(def) = None {
                        def
                    } else if let Some(z) = <RealNumber as minicbor::Decode<
                        Ctx,
                    >>::nil() {
                        z
                    } else {
                        return Err(
                            minicbor::decode::Error::missing_value(4)
                                .with_message("VotingThresholds::hard_fork_initiation")
                                .at(__p777),
                        )
                    },
                    protocol_parameter_network_update: if let Some(x) = protocol_parameter_network_update {
                        x
                    } else if let Some(def) = None {
                        def
                    } else if let Some(z) = <RealNumber as minicbor::Decode<
                        Ctx,
                    >>::nil() {
                        z
                    } else {
                        return Err(
                            minicbor::decode::Error::missing_value(5)
                                .with_message(
                                    "VotingThresholds::protocol_parameter_network_update",
                                )
                                .at(__p777),
                        )
                    },
                    protocol_parameter_economic_update: if let Some(x) = protocol_parameter_economic_update {
                        x
                    } else if let Some(def) = None {
                        def
                    } else if let Some(z) = <RealNumber as minicbor::Decode<
                        Ctx,
                    >>::nil() {
                        z
                    } else {
                        return Err(
                            minicbor::decode::Error::missing_value(6)
                                .with_message(
                                    "VotingThresholds::protocol_parameter_economic_update",
                                )
                                .at(__p777),
                        )
                    },
                    protocol_parameter_technical_update: if let Some(x) = protocol_parameter_technical_update {
                        x
                    } else if let Some(def) = None {
                        def
                    } else if let Some(z) = <RealNumber as minicbor::Decode<
                        Ctx,
                    >>::nil() {
                        z
                    } else {
                        return Err(
                            minicbor::decode::Error::missing_value(7)
                                .with_message(
                                    "VotingThresholds::protocol_parameter_technical_update",
                                )
                                .at(__p777),
                        )
                    },
                    protocol_parameter_security_update: if let Some(x) = protocol_parameter_security_update {
                        x
                    } else if let Some(def) = None {
                        def
                    } else if let Some(z) = <RealNumber as minicbor::Decode<
                        Ctx,
                    >>::nil() {
                        z
                    } else {
                        return Err(
                            minicbor::decode::Error::missing_value(8)
                                .with_message(
                                    "VotingThresholds::protocol_parameter_security_update",
                                )
                                .at(__p777),
                        )
                    },
                })
            }
        }
        impl<Ctx> minicbor::CborLen<Ctx> for VotingThresholds {
            fn cbor_len(&self, __ctx777: &mut Ctx) -> usize {
                0
                    + {
                        let mut __num777 = 0;
                        let mut __len777 = 0;
                        if !minicbor::Encode::<Ctx>::is_nil(&self.motion_no_confidence) {
                            __len777
                                += (0usize - __num777) + 0
                                    + minicbor::CborLen::<
                                        Ctx,
                                    >::cbor_len(&self.motion_no_confidence, __ctx777);
                            __num777 = 0usize + 1;
                        }
                        if !minicbor::Encode::<Ctx>::is_nil(&self.update_committee) {
                            __len777
                                += (1usize - __num777) + 0
                                    + minicbor::CborLen::<
                                        Ctx,
                                    >::cbor_len(&self.update_committee, __ctx777);
                            __num777 = 1usize + 1;
                        }
                        if !minicbor::Encode::<
                            Ctx,
                        >::is_nil(&self.update_committee_no_confidence) {
                            __len777
                                += (2usize - __num777) + 0
                                    + minicbor::CborLen::<
                                        Ctx,
                                    >::cbor_len(&self.update_committee_no_confidence, __ctx777);
                            __num777 = 2usize + 1;
                        }
                        if !minicbor::Encode::<Ctx>::is_nil(&self.update_constitution) {
                            __len777
                                += (3usize - __num777) + 0
                                    + minicbor::CborLen::<
                                        Ctx,
                                    >::cbor_len(&self.update_constitution, __ctx777);
                            __num777 = 3usize + 1;
                        }
                        if !minicbor::Encode::<Ctx>::is_nil(&self.hard_fork_initiation) {
                            __len777
                                += (4usize - __num777) + 0
                                    + minicbor::CborLen::<
                                        Ctx,
                                    >::cbor_len(&self.hard_fork_initiation, __ctx777);
                            __num777 = 4usize + 1;
                        }
                        if !minicbor::Encode::<
                            Ctx,
                        >::is_nil(&self.protocol_parameter_network_update) {
                            __len777
                                += (5usize - __num777) + 0
                                    + minicbor::CborLen::<
                                        Ctx,
                                    >::cbor_len(
                                        &self.protocol_parameter_network_update,
                                        __ctx777,
                                    );
                            __num777 = 5usize + 1;
                        }
                        if !minicbor::Encode::<
                            Ctx,
                        >::is_nil(&self.protocol_parameter_economic_update) {
                            __len777
                                += (6usize - __num777) + 0
                                    + minicbor::CborLen::<
                                        Ctx,
                                    >::cbor_len(
                                        &self.protocol_parameter_economic_update,
                                        __ctx777,
                                    );
                            __num777 = 6usize + 1;
                        }
                        if !minicbor::Encode::<
                            Ctx,
                        >::is_nil(&self.protocol_parameter_technical_update) {
                            __len777
                                += (7usize - __num777) + 0
                                    + minicbor::CborLen::<
                                        Ctx,
                                    >::cbor_len(
                                        &self.protocol_parameter_technical_update,
                                        __ctx777,
                                    );
                            __num777 = 7usize + 1;
                        }
                        if !minicbor::Encode::<
                            Ctx,
                        >::is_nil(&self.protocol_parameter_security_update) {
                            __len777
                                += (8usize - __num777) + 0
                                    + minicbor::CborLen::<
                                        Ctx,
                                    >::cbor_len(
                                        &self.protocol_parameter_security_update,
                                        __ctx777,
                                    );
                            __num777 = 8usize + 1;
                        }
                        __num777.cbor_len(__ctx777) + __len777
                    }
            }
        }
    }
    use minicbor::{CborLen, Decode, Encode};
    pub use action::Action;
    pub use delegate_representative::DelegateRepresentative;
    use crate::{
        address::shelley::StakeAddress, crypto::{Blake2b224Digest, Blake2b256Digest},
        transaction::Coin,
    };
    pub struct Anchor {
        #[cbor(n(0), with = "cbor_util::url")]
        url: Box<str>,
        #[cbor(n(1), with = "minicbor::bytes")]
        data_hash: Blake2b256Digest,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Anchor {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "Anchor",
                "url",
                &self.url,
                "data_hash",
                &&self.data_hash,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Anchor {
        #[inline]
        fn clone(&self) -> Anchor {
            Anchor {
                url: ::core::clone::Clone::clone(&self.url),
                data_hash: ::core::clone::Clone::clone(&self.data_hash),
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Anchor {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Anchor {
        #[inline]
        fn eq(&self, other: &Anchor) -> bool {
            self.url == other.url && self.data_hash == other.data_hash
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for Anchor {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<Box<str>>;
            let _: ::core::cmp::AssertParamIsEq<Blake2b256Digest>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for Anchor {
        #[inline]
        fn partial_cmp(
            &self,
            other: &Anchor,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            match ::core::cmp::PartialOrd::partial_cmp(&self.url, &other.url) {
                ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                    ::core::cmp::PartialOrd::partial_cmp(
                        &self.data_hash,
                        &other.data_hash,
                    )
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for Anchor {
        #[inline]
        fn cmp(&self, other: &Anchor) -> ::core::cmp::Ordering {
            match ::core::cmp::Ord::cmp(&self.url, &other.url) {
                ::core::cmp::Ordering::Equal => {
                    ::core::cmp::Ord::cmp(&self.data_hash, &other.data_hash)
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for Anchor {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.url, state);
            ::core::hash::Hash::hash(&self.data_hash, state)
        }
    }
    impl<Ctx> minicbor::Encode<Ctx> for Anchor {
        fn encode<__W777>(
            &self,
            __e777: &mut minicbor::Encoder<__W777>,
            __ctx777: &mut Ctx,
        ) -> core::result::Result<(), minicbor::encode::Error<__W777::Error>>
        where
            __W777: minicbor::encode::Write,
        {
            let mut __max_index777: core::option::Option<u64> = None;
            if !(|_| false)(&self.url) {
                __max_index777 = Some(0u64);
            }
            if !(|_| false)(&self.data_hash) {
                __max_index777 = Some(1u64);
            }
            if let Some(__i777) = __max_index777 {
                __e777.array(__i777 + 1)?;
                if 0 <= __i777 {
                    cbor_util::url::encode(&self.url, __e777, __ctx777)?
                }
                if 1 <= __i777 {
                    minicbor::bytes::encode(&self.data_hash, __e777, __ctx777)?
                }
            } else {
                __e777.array(0)?;
            }
            Ok(())
        }
    }
    impl<'bytes, Ctx> minicbor::Decode<'bytes, Ctx> for Anchor {
        fn decode(
            __d777: &mut minicbor::Decoder<'bytes>,
            __ctx777: &mut Ctx,
        ) -> core::result::Result<Anchor, minicbor::decode::Error> {
            let __p777 = __d777.position();
            let mut url: core::option::Option<Box<str>> = None;
            let mut data_hash: core::option::Option<Blake2b256Digest> = None;
            if let Some(__len777) = __d777.array()? {
                for __i777 in 0..__len777 {
                    match __i777 {
                        0 => {
                            match cbor_util::url::decode(__d777, __ctx777) {
                                Ok(__v777) => url = Some(__v777),
                                Err(e) => return Err(e),
                            }
                        }
                        1 => {
                            match minicbor::bytes::decode(__d777, __ctx777) {
                                Ok(__v777) => data_hash = Some(__v777),
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
                            match cbor_util::url::decode(__d777, __ctx777) {
                                Ok(__v777) => url = Some(__v777),
                                Err(e) => return Err(e),
                            }
                        }
                        1 => {
                            match minicbor::bytes::decode(__d777, __ctx777) {
                                Ok(__v777) => data_hash = Some(__v777),
                                Err(e) => return Err(e),
                            }
                        }
                        _ => __d777.skip()?,
                    }
                    __i777 += 1;
                }
                __d777.skip()?
            }
            Ok(Anchor {
                url: if let Some(x) = url {
                    x
                } else if let Some(def) = None {
                    def
                } else if let Some(z) = None {
                    z
                } else {
                    return Err(
                        minicbor::decode::Error::missing_value(0)
                            .with_message("Anchor::url")
                            .at(__p777),
                    )
                },
                data_hash: if let Some(x) = data_hash {
                    x
                } else if let Some(def) = None {
                    def
                } else if let Some(z) = None {
                    z
                } else {
                    return Err(
                        minicbor::decode::Error::missing_value(1)
                            .with_message("Anchor::data_hash")
                            .at(__p777),
                    )
                },
            })
        }
    }
    impl<Ctx> minicbor::CborLen<Ctx> for Anchor {
        fn cbor_len(&self, __ctx777: &mut Ctx) -> usize {
            0
                + {
                    let mut __num777 = 0;
                    let mut __len777 = 0;
                    if !(|_| false)(&self.url) {
                        __len777
                            += (0usize - __num777) + 0
                                + cbor_util::url::cbor_len(&self.url, __ctx777);
                        __num777 = 0usize + 1;
                    }
                    if !(|_| false)(&self.data_hash) {
                        __len777
                            += (1usize - __num777) + 0
                                + minicbor::bytes::cbor_len(&self.data_hash, __ctx777);
                        __num777 = 1usize + 1;
                    }
                    __num777.cbor_len(__ctx777) + __len777
                }
        }
    }
    pub struct Constitution {
        #[n(0)]
        pub anchor: Anchor,
        #[cbor(n(1), with = "minicbor::bytes")]
        pub script_hash: Option<Blake2b224Digest>,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Constitution {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "Constitution",
                "anchor",
                &self.anchor,
                "script_hash",
                &&self.script_hash,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Constitution {
        #[inline]
        fn clone(&self) -> Constitution {
            Constitution {
                anchor: ::core::clone::Clone::clone(&self.anchor),
                script_hash: ::core::clone::Clone::clone(&self.script_hash),
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Constitution {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Constitution {
        #[inline]
        fn eq(&self, other: &Constitution) -> bool {
            self.anchor == other.anchor && self.script_hash == other.script_hash
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for Constitution {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<Anchor>;
            let _: ::core::cmp::AssertParamIsEq<Option<Blake2b224Digest>>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for Constitution {
        #[inline]
        fn partial_cmp(
            &self,
            other: &Constitution,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            match ::core::cmp::PartialOrd::partial_cmp(&self.anchor, &other.anchor) {
                ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                    ::core::cmp::PartialOrd::partial_cmp(
                        &self.script_hash,
                        &other.script_hash,
                    )
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for Constitution {
        #[inline]
        fn cmp(&self, other: &Constitution) -> ::core::cmp::Ordering {
            match ::core::cmp::Ord::cmp(&self.anchor, &other.anchor) {
                ::core::cmp::Ordering::Equal => {
                    ::core::cmp::Ord::cmp(&self.script_hash, &other.script_hash)
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for Constitution {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.anchor, state);
            ::core::hash::Hash::hash(&self.script_hash, state)
        }
    }
    impl<Ctx> minicbor::Encode<Ctx> for Constitution {
        fn encode<__W777>(
            &self,
            __e777: &mut minicbor::Encoder<__W777>,
            __ctx777: &mut Ctx,
        ) -> core::result::Result<(), minicbor::encode::Error<__W777::Error>>
        where
            __W777: minicbor::encode::Write,
        {
            let mut __max_index777: core::option::Option<u64> = None;
            if !minicbor::Encode::<Ctx>::is_nil(&self.anchor) {
                __max_index777 = Some(0u64);
            }
            if !core::option::Option::is_none(&self.script_hash) {
                __max_index777 = Some(1u64);
            }
            if let Some(__i777) = __max_index777 {
                __e777.array(__i777 + 1)?;
                if 0 <= __i777 {
                    minicbor::Encode::encode(&self.anchor, __e777, __ctx777)?
                }
                if 1 <= __i777 {
                    minicbor::bytes::encode(&self.script_hash, __e777, __ctx777)?
                }
            } else {
                __e777.array(0)?;
            }
            Ok(())
        }
    }
    impl<'bytes, Ctx> minicbor::Decode<'bytes, Ctx> for Constitution {
        fn decode(
            __d777: &mut minicbor::Decoder<'bytes>,
            __ctx777: &mut Ctx,
        ) -> core::result::Result<Constitution, minicbor::decode::Error> {
            let __p777 = __d777.position();
            let mut anchor: core::option::Option<Anchor> = None;
            let mut script_hash: core::option::Option<Option<Blake2b224Digest>> = Some(
                None,
            );
            if let Some(__len777) = __d777.array()? {
                for __i777 in 0..__len777 {
                    match __i777 {
                        0 => {
                            match minicbor::Decode::decode(__d777, __ctx777) {
                                Ok(__v777) => anchor = Some(__v777),
                                Err(
                                    e,
                                ) if e.is_unknown_variant()
                                    && <Anchor as minicbor::Decode<Ctx>>::nil().is_some() => {
                                    __d777.skip()?
                                }
                                Err(e) => return Err(e),
                            }
                        }
                        1 => {
                            match minicbor::bytes::decode(__d777, __ctx777) {
                                Ok(__v777) => script_hash = Some(__v777),
                                Err(e) if e.is_unknown_variant() => __d777.skip()?,
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
                            match minicbor::Decode::decode(__d777, __ctx777) {
                                Ok(__v777) => anchor = Some(__v777),
                                Err(
                                    e,
                                ) if e.is_unknown_variant()
                                    && <Anchor as minicbor::Decode<Ctx>>::nil().is_some() => {
                                    __d777.skip()?
                                }
                                Err(e) => return Err(e),
                            }
                        }
                        1 => {
                            match minicbor::bytes::decode(__d777, __ctx777) {
                                Ok(__v777) => script_hash = Some(__v777),
                                Err(e) if e.is_unknown_variant() => __d777.skip()?,
                                Err(e) => return Err(e),
                            }
                        }
                        _ => __d777.skip()?,
                    }
                    __i777 += 1;
                }
                __d777.skip()?
            }
            Ok(Constitution {
                anchor: if let Some(x) = anchor {
                    x
                } else if let Some(def) = None {
                    def
                } else if let Some(z) = <Anchor as minicbor::Decode<Ctx>>::nil() {
                    z
                } else {
                    return Err(
                        minicbor::decode::Error::missing_value(0)
                            .with_message("Constitution::anchor")
                            .at(__p777),
                    )
                },
                script_hash: if let Some(x) = script_hash {
                    x
                } else if let Some(def) = None {
                    def
                } else if let Some(z) = Some(None) {
                    z
                } else {
                    return Err(
                        minicbor::decode::Error::missing_value(1)
                            .with_message("Constitution::script_hash")
                            .at(__p777),
                    )
                },
            })
        }
    }
    impl<Ctx> minicbor::CborLen<Ctx> for Constitution {
        fn cbor_len(&self, __ctx777: &mut Ctx) -> usize {
            0
                + {
                    let mut __num777 = 0;
                    let mut __len777 = 0;
                    if !minicbor::Encode::<Ctx>::is_nil(&self.anchor) {
                        __len777
                            += (0usize - __num777) + 0
                                + minicbor::CborLen::<
                                    Ctx,
                                >::cbor_len(&self.anchor, __ctx777);
                        __num777 = 0usize + 1;
                    }
                    if !core::option::Option::is_none(&self.script_hash) {
                        __len777
                            += (1usize - __num777) + 0
                                + minicbor::bytes::cbor_len(&self.script_hash, __ctx777);
                        __num777 = 1usize + 1;
                    }
                    __num777.cbor_len(__ctx777) + __len777
                }
        }
    }
    pub struct Proposal {
        #[n(0)]
        pub deposit: Coin,
        #[n(1)]
        pub account: StakeAddress,
        #[n(2)]
        pub action: Action,
        #[n(3)]
        pub anchor: Anchor,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Proposal {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field4_finish(
                f,
                "Proposal",
                "deposit",
                &self.deposit,
                "account",
                &self.account,
                "action",
                &self.action,
                "anchor",
                &&self.anchor,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Proposal {
        #[inline]
        fn clone(&self) -> Proposal {
            Proposal {
                deposit: ::core::clone::Clone::clone(&self.deposit),
                account: ::core::clone::Clone::clone(&self.account),
                action: ::core::clone::Clone::clone(&self.action),
                anchor: ::core::clone::Clone::clone(&self.anchor),
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Proposal {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Proposal {
        #[inline]
        fn eq(&self, other: &Proposal) -> bool {
            self.deposit == other.deposit && self.account == other.account
                && self.action == other.action && self.anchor == other.anchor
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for Proposal {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<Coin>;
            let _: ::core::cmp::AssertParamIsEq<StakeAddress>;
            let _: ::core::cmp::AssertParamIsEq<Action>;
            let _: ::core::cmp::AssertParamIsEq<Anchor>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for Proposal {
        #[inline]
        fn partial_cmp(
            &self,
            other: &Proposal,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            match ::core::cmp::PartialOrd::partial_cmp(&self.deposit, &other.deposit) {
                ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                    match ::core::cmp::PartialOrd::partial_cmp(
                        &self.account,
                        &other.account,
                    ) {
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                            match ::core::cmp::PartialOrd::partial_cmp(
                                &self.action,
                                &other.action,
                            ) {
                                ::core::option::Option::Some(
                                    ::core::cmp::Ordering::Equal,
                                ) => {
                                    ::core::cmp::PartialOrd::partial_cmp(
                                        &self.anchor,
                                        &other.anchor,
                                    )
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
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for Proposal {
        #[inline]
        fn cmp(&self, other: &Proposal) -> ::core::cmp::Ordering {
            match ::core::cmp::Ord::cmp(&self.deposit, &other.deposit) {
                ::core::cmp::Ordering::Equal => {
                    match ::core::cmp::Ord::cmp(&self.account, &other.account) {
                        ::core::cmp::Ordering::Equal => {
                            match ::core::cmp::Ord::cmp(&self.action, &other.action) {
                                ::core::cmp::Ordering::Equal => {
                                    ::core::cmp::Ord::cmp(&self.anchor, &other.anchor)
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
    }
    #[automatically_derived]
    impl ::core::hash::Hash for Proposal {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.deposit, state);
            ::core::hash::Hash::hash(&self.account, state);
            ::core::hash::Hash::hash(&self.action, state);
            ::core::hash::Hash::hash(&self.anchor, state)
        }
    }
    impl<Ctx> minicbor::Encode<Ctx> for Proposal {
        fn encode<__W777>(
            &self,
            __e777: &mut minicbor::Encoder<__W777>,
            __ctx777: &mut Ctx,
        ) -> core::result::Result<(), minicbor::encode::Error<__W777::Error>>
        where
            __W777: minicbor::encode::Write,
        {
            let mut __max_index777: core::option::Option<u64> = None;
            if !minicbor::Encode::<Ctx>::is_nil(&self.deposit) {
                __max_index777 = Some(0u64);
            }
            if !minicbor::Encode::<Ctx>::is_nil(&self.account) {
                __max_index777 = Some(1u64);
            }
            if !minicbor::Encode::<Ctx>::is_nil(&self.action) {
                __max_index777 = Some(2u64);
            }
            if !minicbor::Encode::<Ctx>::is_nil(&self.anchor) {
                __max_index777 = Some(3u64);
            }
            if let Some(__i777) = __max_index777 {
                __e777.array(__i777 + 1)?;
                if 0 <= __i777 {
                    minicbor::Encode::encode(&self.deposit, __e777, __ctx777)?
                }
                if 1 <= __i777 {
                    minicbor::Encode::encode(&self.account, __e777, __ctx777)?
                }
                if 2 <= __i777 {
                    minicbor::Encode::encode(&self.action, __e777, __ctx777)?
                }
                if 3 <= __i777 {
                    minicbor::Encode::encode(&self.anchor, __e777, __ctx777)?
                }
            } else {
                __e777.array(0)?;
            }
            Ok(())
        }
    }
    impl<'bytes, Ctx> minicbor::Decode<'bytes, Ctx> for Proposal {
        fn decode(
            __d777: &mut minicbor::Decoder<'bytes>,
            __ctx777: &mut Ctx,
        ) -> core::result::Result<Proposal, minicbor::decode::Error> {
            let __p777 = __d777.position();
            let mut deposit: core::option::Option<Coin> = None;
            let mut account: core::option::Option<StakeAddress> = None;
            let mut action: core::option::Option<Action> = None;
            let mut anchor: core::option::Option<Anchor> = None;
            if let Some(__len777) = __d777.array()? {
                for __i777 in 0..__len777 {
                    match __i777 {
                        0 => {
                            match minicbor::Decode::decode(__d777, __ctx777) {
                                Ok(__v777) => deposit = Some(__v777),
                                Err(
                                    e,
                                ) if e.is_unknown_variant()
                                    && <Coin as minicbor::Decode<Ctx>>::nil().is_some() => {
                                    __d777.skip()?
                                }
                                Err(e) => return Err(e),
                            }
                        }
                        1 => {
                            match minicbor::Decode::decode(__d777, __ctx777) {
                                Ok(__v777) => account = Some(__v777),
                                Err(
                                    e,
                                ) if e.is_unknown_variant()
                                    && <StakeAddress as minicbor::Decode<Ctx>>::nil()
                                        .is_some() => __d777.skip()?,
                                Err(e) => return Err(e),
                            }
                        }
                        2 => {
                            match minicbor::Decode::decode(__d777, __ctx777) {
                                Ok(__v777) => action = Some(__v777),
                                Err(
                                    e,
                                ) if e.is_unknown_variant()
                                    && <Action as minicbor::Decode<Ctx>>::nil().is_some() => {
                                    __d777.skip()?
                                }
                                Err(e) => return Err(e),
                            }
                        }
                        3 => {
                            match minicbor::Decode::decode(__d777, __ctx777) {
                                Ok(__v777) => anchor = Some(__v777),
                                Err(
                                    e,
                                ) if e.is_unknown_variant()
                                    && <Anchor as minicbor::Decode<Ctx>>::nil().is_some() => {
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
                            match minicbor::Decode::decode(__d777, __ctx777) {
                                Ok(__v777) => deposit = Some(__v777),
                                Err(
                                    e,
                                ) if e.is_unknown_variant()
                                    && <Coin as minicbor::Decode<Ctx>>::nil().is_some() => {
                                    __d777.skip()?
                                }
                                Err(e) => return Err(e),
                            }
                        }
                        1 => {
                            match minicbor::Decode::decode(__d777, __ctx777) {
                                Ok(__v777) => account = Some(__v777),
                                Err(
                                    e,
                                ) if e.is_unknown_variant()
                                    && <StakeAddress as minicbor::Decode<Ctx>>::nil()
                                        .is_some() => __d777.skip()?,
                                Err(e) => return Err(e),
                            }
                        }
                        2 => {
                            match minicbor::Decode::decode(__d777, __ctx777) {
                                Ok(__v777) => action = Some(__v777),
                                Err(
                                    e,
                                ) if e.is_unknown_variant()
                                    && <Action as minicbor::Decode<Ctx>>::nil().is_some() => {
                                    __d777.skip()?
                                }
                                Err(e) => return Err(e),
                            }
                        }
                        3 => {
                            match minicbor::Decode::decode(__d777, __ctx777) {
                                Ok(__v777) => anchor = Some(__v777),
                                Err(
                                    e,
                                ) if e.is_unknown_variant()
                                    && <Anchor as minicbor::Decode<Ctx>>::nil().is_some() => {
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
            Ok(Proposal {
                deposit: if let Some(x) = deposit {
                    x
                } else if let Some(def) = None {
                    def
                } else if let Some(z) = <Coin as minicbor::Decode<Ctx>>::nil() {
                    z
                } else {
                    return Err(
                        minicbor::decode::Error::missing_value(0)
                            .with_message("Proposal::deposit")
                            .at(__p777),
                    )
                },
                account: if let Some(x) = account {
                    x
                } else if let Some(def) = None {
                    def
                } else if let Some(z) = <StakeAddress as minicbor::Decode<Ctx>>::nil() {
                    z
                } else {
                    return Err(
                        minicbor::decode::Error::missing_value(1)
                            .with_message("Proposal::account")
                            .at(__p777),
                    )
                },
                action: if let Some(x) = action {
                    x
                } else if let Some(def) = None {
                    def
                } else if let Some(z) = <Action as minicbor::Decode<Ctx>>::nil() {
                    z
                } else {
                    return Err(
                        minicbor::decode::Error::missing_value(2)
                            .with_message("Proposal::action")
                            .at(__p777),
                    )
                },
                anchor: if let Some(x) = anchor {
                    x
                } else if let Some(def) = None {
                    def
                } else if let Some(z) = <Anchor as minicbor::Decode<Ctx>>::nil() {
                    z
                } else {
                    return Err(
                        minicbor::decode::Error::missing_value(3)
                            .with_message("Proposal::anchor")
                            .at(__p777),
                    )
                },
            })
        }
    }
    impl<Ctx> minicbor::CborLen<Ctx> for Proposal {
        fn cbor_len(&self, __ctx777: &mut Ctx) -> usize {
            0
                + {
                    let mut __num777 = 0;
                    let mut __len777 = 0;
                    if !minicbor::Encode::<Ctx>::is_nil(&self.deposit) {
                        __len777
                            += (0usize - __num777) + 0
                                + minicbor::CborLen::<
                                    Ctx,
                                >::cbor_len(&self.deposit, __ctx777);
                        __num777 = 0usize + 1;
                    }
                    if !minicbor::Encode::<Ctx>::is_nil(&self.account) {
                        __len777
                            += (1usize - __num777) + 0
                                + minicbor::CborLen::<
                                    Ctx,
                                >::cbor_len(&self.account, __ctx777);
                        __num777 = 1usize + 1;
                    }
                    if !minicbor::Encode::<Ctx>::is_nil(&self.action) {
                        __len777
                            += (2usize - __num777) + 0
                                + minicbor::CborLen::<
                                    Ctx,
                                >::cbor_len(&self.action, __ctx777);
                        __num777 = 2usize + 1;
                    }
                    if !minicbor::Encode::<Ctx>::is_nil(&self.anchor) {
                        __len777
                            += (3usize - __num777) + 0
                                + minicbor::CborLen::<
                                    Ctx,
                                >::cbor_len(&self.anchor, __ctx777);
                        __num777 = 3usize + 1;
                    }
                    __num777.cbor_len(__ctx777) + __len777
                }
        }
    }
}
