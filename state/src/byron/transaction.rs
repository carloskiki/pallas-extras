use ledger::{
    Coin,
    byron::{
        self, address,
        protocol::FeePolicy,
        transaction::{Output, Witness, witness::data},
    },
    crypto::{
        self,
        ed25519_dalek::{self, ed25519::signature::MultipartVerifier},
    },
    transaction::Reference,
};
use std::collections::HashMap;

pub type State = HashMap<Reference, Output>;

pub struct Environment {
    network_magic: [u8; 5],
    max_transaction_size: u64,
    fee_policy: FeePolicy,
}

pub struct Signal<'a> {
    // TODO: do we want to move the transaction here, avoiding a clone? We can answer this once we
    // have validation workflow complete. Gut is _no_, because most things are not copied into the
    // state.
    pub transaction: &'a byron::Transaction,
    pub transaction_hash: crypto::Blake2b256Digest,
    pub witnesses: &'a [Witness<'a>],
    pub size: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum Error<'a> {
    #[error("transaction input missing from state")]
    MissingInput(&'a ledger::transaction::Reference),
    #[error("witness at index {index} is invalid")]
    InvalidWitness {
        index: usize,
        #[source]
        error: ed25519_dalek::SignatureError,
    },
    #[error("witness at index {0} does not belong to the address")]
    AddressMismatch(usize),
    #[error("transaction size exceeds maximum allowed")]
    OversizedTransaction,
    #[error("paid fee {calculated_fee} is less than minimum required {minimum_fee}")]
    InsufficientFee {
        minimum_fee: Coin,
        calculated_fee: Coin,
    },
}

pub fn transition<'a>(
    state: &mut State,
    env: &Environment,
    Signal {
        transaction,
        transaction_hash,
        witnesses,
        size,
    }: &'a Signal<'a>,
) -> Result<(), Error<'a>> {
    const HASH_DELIMITER: &[u8; 2] = &[0x58, 0x20];

    // 1. Validate transaction size
    if *size > env.max_transaction_size {
        return Err(Error::OversizedTransaction);
    }

    // 2. Validate witnesses against addresses, and calculate total input value, storing whether
    //    the transaction is `redeem`.
    let mut redeem = true;
    let input_value: Coin = transaction
        .inputs
        .iter()
        .zip(witnesses.iter())
        .enumerate()
        .map(|(index, (input, witness))| -> Result<Coin, Error> {
            let input_utxo = &state.get(input).ok_or(Error::MissingInput(input))?;

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
            let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&verifying_key)
                .map_err(|error| Error::InvalidWitness { index, error })?;
            verifying_key
                .multipart_verify(
                    &[
                        &[match witness_type {
                            address::Type::VerifyingKey => 1,
                            address::Type::Redeem => 2,
                        }],
                        &env.network_magic,
                        HASH_DELIMITER,
                        transaction_hash,
                    ],
                    signature,
                )
                .map_err(|error| Error::InvalidWitness { index, error })?;

            let address::Payload {
                address_type,
                root_digest,
                attributes,
            } = &input_utxo.address.payload;
            if *address_type != witness_type
                || *root_digest
                    != byron::address::root_digest(
                        address::Type::VerifyingKey,
                        address_data,
                        attributes,
                    )
            {
                return Err(Error::AddressMismatch(index));
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
        state.remove(input);
    });
    transaction
        .outputs
        .iter()
        .enumerate()
        .for_each(|(index, output)| {
            let input = Reference {
                id: *transaction_hash,
                index: index as u16,
            };
            state.insert(input, output.clone());
        });

    Ok(())
}
