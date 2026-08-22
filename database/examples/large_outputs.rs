//! Find transactions from any era with an individual output worth more than 1 billion Ada.

use ledger::{
    Block,
    crypto::{self, DigestWriter, blake2::Blake2b256, digest::Digest},
};
use std::{env, error::Error, fmt::Write, io, path::PathBuf};
use tinycbor::{Decode, Decoder, Encode, Encoder};

const LOVELACE_PER_ADA: u64 = 1_000_000;
const MIN_OUTPUT: u64 = 1_000_000_000 * LOVELACE_PER_ADA;

fn main() -> Result<(), Box<dyn Error>> {
    let data = env::args_os().nth(1).map(PathBuf::from).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: cargo run --example large_outputs -- <database-directory>",
        )
    })?;
    // We don't need any caching, because we are processing all blocks in order once.
    let (reader, _) = database::open::<0>(data)?;
    let Some(tip) = reader.tip() else {
        return Ok(());
    };

    let mut chunks = reader.read(0..tip.saturating_add(1));
    while let Some(blocks) = chunks.next() {
        for block in blocks? {
            let bytes = block.map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "block checksum mismatch")
            })?;
            let block = Block::decode(&mut Decoder(&bytes))?;
            match block {
                Block::Boundary(_) => {}
                Block::Byron(block) => {
                    for payload in block.body.transactions {
                        let payload: &ledger::byron::transaction::Payload<'_> = payload.as_ref();
                        let transaction: &ledger::byron::Transaction = payload.transaction.as_ref();
                        let mut encoder: Encoder<crypto::DigestWriter<Blake2b256>> = Encoder::default();
                        payload.transaction.as_ref().encode(&mut encoder);
                        report_large_outputs(
                            "Byron",
                            transaction.outputs.iter().map(|output| output.amount),
                            || encoder.0.0.finalize().into(),
                        );
                    }
                }
                Block::Shelley(block) => {
                    for body in &block.transaction_bodies {
                        report_encoded_transaction(
                            "Shelley",
                            body,
                            body.outputs.iter().map(|output| output.amount),
                        );
                    }
                }
                Block::Allegra(block) => {
                    for body in &block.transaction_bodies {
                        report_encoded_transaction(
                            "Allegra",
                            body,
                            body.outputs.iter().map(|output| output.amount),
                        );
                    }
                }
                Block::Mary(block) => {
                    for body in &block.transaction_bodies {
                        report_encoded_transaction(
                            "Mary",
                            body,
                            body.outputs
                                .iter()
                                .map(|output| mary_lovelace(&output.value)),
                        );
                    }
                }
                Block::Alonzo(block) => {
                    for body in &block.transaction_bodies {
                        report_encoded_transaction(
                            "Alonzo",
                            body,
                            body.outputs
                                .iter()
                                .map(|output| mary_lovelace(&output.value)),
                        );
                    }
                }
                Block::Babbage(block) => {
                    for body in &block.transaction_bodies {
                        report_encoded_transaction(
                            "Babbage",
                            body,
                            body.outputs
                                .iter()
                                .map(|output| mary_lovelace(&output.value)),
                        );
                    }
                }
                Block::Conway(block) => {
                    for body in &block.transaction_bodies {
                        report_encoded_transaction(
                            "Conway",
                            body,
                            body.outputs
                                .iter()
                                .map(|output| conway_lovelace(&output.value)),
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

fn report_encoded_transaction<T: Encode>(
    era: &str,
    body: &T,
    outputs: impl IntoIterator<Item = u64>,
) {
    report_large_outputs(era, outputs, || encoded_transaction_id(body));
}

fn report_large_outputs(
    era: &str,
    outputs: impl IntoIterator<Item = u64>,
    transaction_id: impl FnOnce() -> [u8; 32],
) {
    let mut outputs = outputs
        .into_iter()
        .enumerate()
        .filter(|(_, amount)| *amount > MIN_OUTPUT)
        .peekable();
    if outputs.peek().is_none() {
        return;
    }

    let transaction_id = hex(&transaction_id());
    for (index, amount) in outputs {
        println!(
            "{era} {transaction_id} output {index}: {}.{:06} Ada",
            amount / LOVELACE_PER_ADA,
            amount % LOVELACE_PER_ADA,
        );
    }
}

fn encoded_transaction_id(body: &impl Encode) -> [u8; 32] {
    let mut encoder = Encoder(DigestWriter(Blake2b256::new()));
    body.encode(&mut encoder)
        .expect("writing to a digest cannot fail");
    encoder.0.0.finalize().into()
}

fn mary_lovelace(value: &ledger::mary::transaction::Value<'_>) -> u64 {
    match value {
        ledger::mary::transaction::Value::Lovelace(lovelace)
        | ledger::mary::transaction::Value::Other { lovelace, .. } => *lovelace,
    }
}

fn conway_lovelace(value: &ledger::conway::transaction::Value<'_>) -> u64 {
    match value {
        ledger::conway::transaction::Value::Lovelace(lovelace)
        | ledger::conway::transaction::Value::Other { lovelace, .. } => *lovelace,
    }
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}
