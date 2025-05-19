use std::str::FromStr;

use bip32::ExtendedVerifyingKey;
use digest::KeyInit;
use ed25519_dalek::{Signer, SigningKey, ed25519::signature::Keypair};
use kes::Evolve;
use ledger::{
    Asset, Block, Certificate, Script,
    address::shelley::{Address, StakeAddress},
    asset::Bundle,
    block, certificate, protocol, script, transaction, witness,
};
use minicbor::{CborLen, Encode};

const BYTES32: [u8; 32] = [
    43, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 22, 3, 4, 5, 6, 7, 8, 8, 4, 5, 6, 67, 3, 34, 213, 4, 4,
    12, 34, 2,
];

const BYTES80: [u8; 80] = [0; 80];

#[test]
fn block() {
    let vrf_certificate = block::header::VrfCertificate {
        hash: [0; 64].into(),
        proof: BYTES80,
    };

    let kes_signing_key: kes::sum::Pow6<
        kes::SingleUse<ed25519_dalek::SigningKey>,
        ledger::crypto::Blake2b256,
    > = kes::sum::Pow6::new(&BYTES32.into());
    let mock_message = b"message";
    let kes_signature: kes::sum::Pow6Signature<_, _, _> = kes_signing_key.sign(mock_message);
    let kes_verifying_key = kes_signing_key.verifying_key();
    let signing_key = SigningKey::from_bytes(&BYTES32);
    let signature = signing_key.sign(mock_message);

    let body = block::header::Body {
        block_number: 42,
        slot: 64,
        previous_hash: Some(BYTES32),
        issuer_vkey: BYTES32,
        vrf_vkey: BYTES32,
        nonce_vrf: vrf_certificate.clone(),
        leader_vrf: vrf_certificate,
        block_body_size: 3292323,
        block_body_hash: BYTES32,
        operational_certificate: block::header::OperationalCertificate {
            kes_verifying_key,
            sequence_number: 43,
            kes_start_period: kes_signing_key.period() as u64,
            signature,
        },
        protocol_version: protocol::Version {
            major: protocol::MajorVersion::Mary,
            minor: 0,
        },
    };

    let header = block::Header {
        body,
        signature: kes_signature,
    };
    check_len(&header);

    // let block = Block {
    //     header,
    //     transaction_bodies: Box::new([transaction::Body {
    //         inputs: Box::new([transaction::Input {
    //             id: BYTES32,
    //             index: 1,
    //         }]),
    //         outputs: Box::new([transaction::Output {
    //             address: Address::from_str(
    //                 "addr1gx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer5pnz75xxcrzqf96k",
    //             )
    //             .unwrap(),
    //             amount: 3456,
    //             datum: Some(transaction::Datum::Hash(BYTES32)),
    //             script_ref: Some(Script::Native(script::native::Script::All(Box::new([
    //                 script::native::Script::Vkey([0; 28]),
    //             ])))),
    //         }]),
    //         fee: 345,
    //         ttl: Some(255),
    //         certificates: Box::new([Certificate::MoveRewards {
    //             from_treasury: true,
    //             to: certificate::RewardTarget::StakeAddresses(Box::new([(
    //                 StakeAddress::from_str(
    //                     "stake1uyehkck0lajq8gr28t9uxnuvgcqrc6070x3k9r8048z8y5gh6ffgw",
    //                 )
    //                 .unwrap(),
    //                 2345,
    //             )])),
    //         }]),
    //         withdrawals: Box::new([]),
    //         update: Some(protocol::Update {
    //             proposed: Box::new([(
    //                 [0; 28],
    //                 protocol::ParameterUpdate(Box::new([
    //                     protocol::Parameter::MinfeeA(34),
    //                     protocol::Parameter::MinfeeB(14),
    //                 ])),
    //             )]),
    //             epoch: 678,
    //         }),
    //         data_hash: None,
    //         validity_start: Some(45),
    //         mint: Some(Asset(Box::new([(
    //             [0; 28],
    //             Bundle(Box::new([(BYTES32, 45)])),
    //         )]))),
    //         script_data_hash: None,
    //         collateral: Box::new([]),
    //         required_signers: Box::new([]),
    //         collateral_return: None,
    //         total_collateral: None,
    //         reference_inputs: Box::new([]),
    //     }]),
    //     witness_sets: Box::new([witness::Set {
    //         verifying_keys: Box::new([]),
    //         native_scripts: Box::new([]),
    //         bootstraps: Box::new([witness::Bootstrap {
    //             key: ExtendedVerifyingKey::new(signing_key.verifying_key().to_bytes(), BYTES32)
    //                 .unwrap(),
    //             signature,
    //             attributes: Box::new([1, 2, 3]),
    //         }]),
    //         plutus_v1: Box::new([]),
    //         plutus_data: Box::new([]),
    //         redeemers: Box::new([]),
    //         plutus_v2: Box::new([]),
    //     }]),
    //     auxiliary_data: Box::new([(
    //         2,
    //         transaction::Data {
    //             metadata: Box::new([(
    //                 45,
    //                 transaction::Metadatum::Map(Box::new([(
    //                     transaction::Metadatum::Text("hello".into()),
    //                     transaction::Metadatum::Integer(-456),
    //                 )])),
    //             )]),
    //             auxiliary_scripts: Box::new([]),
    //             plutus_v1: Box::new([]),
    //             plutus_v2: Box::new([]),
    //         },
    //     )]),
    //     invalid_transactions: Box::new([]),
    // };
    // check_len(&block);
}

fn check_len<T: CborLen<()> + Encode<()>>(value: &T) {
    let mut encoder = minicbor::Encoder::new(Vec::new());
    value.encode(&mut encoder, &mut ()).unwrap();

    assert_eq!(value.cbor_len(&mut ()), encoder.writer().len());
}
