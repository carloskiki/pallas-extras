use crate::transaction::Input;
use hashbrown::HashMap;
use ledger::{
    Coin,
    byron::{
        self, address,
        protocol::FeePolicy,
        transaction::{self, Witness, witness::data},
    },
    crypto::{
        self, Blake2b224Digest,
        blake2::Digest,
        ed25519_dalek::{self, ed25519::signature::MultipartVerifier},
    },
};
use tinycbor::Encode;
use yoke::Yoke;

pub struct Environment {
    network_magic: [u8; 5],
    max_transaction_size: u64,
    fee_policy: FeePolicy,
}

pub struct Signal<'a> {
    pub transaction: &'a byron::Transaction<'a>,
    pub transaction_hash: &'a crypto::Blake2b256Digest,
    pub witnesses: &'a [Witness<'a>],
    pub size: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum Error<'a> {
    #[error("transaction input missing from state")]
    MissingInput(&'a ledger::transaction::Input),
    #[error("witness is invalid {0}")]
    InvalidWitness(#[from] ed25519_dalek::SignatureError),
    #[error("witness does not belong to the address")]
    AddressMismatch,
    #[error("transaction size {size} exceeds maximum allowed {max_size}")]
    OversizedTransaction { size: u64, max_size: u64 },
    #[error("transaction attributes size {size} exceeds maximum allowed")]
    OversizedAttributes { size: usize },
    #[error("paid fee {calculated_fee} is less than minimum required {minimum_fee}")]
    InsufficientFee {
        minimum_fee: Coin,
        calculated_fee: Coin,
    },
    #[error("network magic {actual} does not match expected {expected}")]
    InvalidNetworkMagic { expected: u32, actual: u32 },
}

pub type State = HashMap<Input, Yoke<Output<'static>, Box<[u8]>>>;

impl crate::State for State {
    type Environment = Environment;

    type Signal<'a> = Signal<'a>;

    type Error<'a> = Error<'a>;

    fn transition<'a>(
        &mut self,
        env: &Self::Environment,
        Signal {
            transaction,
            transaction_hash,
            witnesses,
            size,
        }: &'a Self::Signal<'a>,
    ) -> Result<(), Self::Error<'a>> {
        const HASH_DELIMITER: &[u8; 2] = &[0x58, 0x20];

        // 1. Validate transaction size
        if *size > env.max_transaction_size {
            return Err(Error::OversizedTransaction {
                size: *size,
                max_size: env.max_transaction_size,
            });
        }

        // 2. Validate witnesses against addresses, and calculate total input value, storing whether
        //    the transaction is `redeem`.
        let mut redeem = true;
        let input_value: Coin = transaction
            .inputs
            .iter()
            .zip(witnesses.iter())
            .map(|(input, witness)| -> Result<Coin, Error> {
                let input_utxo = &self.get(input).ok_or(Error::MissingInput(input))?.get().0;

                let (verifying_key, signature, witness_type, address_data) = match witness {
                    Witness::VerifyingKey(data::VerifyingKey { key, signature }) => (
                        key.key,
                        signature,
                        address::Type::VerifyingKey,
                        address::Data::VerifyingKey(key),
                    ),
                    Witness::Redeemer(data::Redeemer { key, signature }) => (
                        key.0,
                        signature,
                        address::Type::Redeem,
                        address::Data::Redeem(key),
                    ),
                };
                let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&verifying_key)?;
                verifying_key.multipart_verify(
                    &[
                        &[match witness_type {
                            address::Type::VerifyingKey => 1,
                            address::Type::Redeem => 2,
                        }],
                        &env.network_magic,
                        HASH_DELIMITER,
                        *transaction_hash,
                    ],
                    signature,
                )?;

                let address::Payload {
                    address_type,
                    root_digest,
                    attributes,
                } = &input_utxo.address.payload;
                if *address_type != witness_type
                    || *root_digest
                        != &byron::address::root_digest(
                            address::Type::VerifyingKey,
                            address_data,
                            attributes,
                        )
                {
                    return Err(Error::AddressMismatch);
                }
                redeem &= *address_type == address::Type::Redeem;

                Ok(input_utxo.amount)
            })
            .sum::<Result<Coin, Error>>()?;

        // 3. Validate sufficient fee.
        let minimum_fee = if redeem {
            0
        } else {
            env.fee_policy.constant + env.fee_policy.coefficient * size
        };
        let output_value: Coin = transaction.outputs.iter().map(|output| output.amount).sum();
        if input_value < output_value + minimum_fee {
            return Err(Error::InsufficientFee {
                minimum_fee,
                calculated_fee: input_value.saturating_sub(output_value),
            });
        }

        // 4. Validate attributes size.
        // Decoder ensures that the attributes are an empty map, so this is a stronger requirement
        // than the original implementation, but no data on mainnet has attributes.

        // 5. Validate network magic.
        // On mainnet, the network magic field on `Address` is not used, so we skip validation.

        // 6. Update state.
        transaction.inputs.iter().for_each(|input| {
            self.remove(input);
        });
        transaction
            .outputs
            .iter()
            .enumerate()
            .for_each(|(index, output)| {
                // Reallocate the output bytes
                let key_derivation_path = output.address.payload.attributes.key_derivation_path;
                let mut bytes = Vec::with_capacity(
                    output.address.payload.root_digest.len()
                        + key_derivation_path
                            .map_or(0, |path| path.len()),
                );
                bytes.extend_from_slice(output.address.payload.root_digest);
                if let Some(path) = key_derivation_path {
                    bytes.extend_from_slice(path);
                }
                let bytes = bytes.into_boxed_slice();
                let output = Yoke::attach_to_cart(bytes, |bytes| {
                    let (root_digest, attributes): (&Blake2b224Digest, _) = bytes
                        .split_first_chunk()
                        .expect("bytes contains at least the root digest.");
                    let mut output = *output;
                    output.address.payload.root_digest = root_digest;
                    output.address.payload.attributes.key_derivation_path = if attributes.is_empty() {
                });
            });
    }
}

#[derive(yoke::Yokeable)]
pub struct Output<'a>(pub transaction::Output<'a>);
