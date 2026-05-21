use crate::open;
use ledger::{
    Block, Unique,
    babbage::certificate,
    conway::{
        self,
        block::{self, header},
        protocol,
    },
    crypto::ed25519_dalek::pkcs8::PublicKeyBytes,
    shelley::certificate::Vrf,
    slot,
};
use std::path::{Path, PathBuf};
use tinycbor::{Encode, Encoder, encoded::With};

#[test]
fn new() {
    suite(|_| {});
}

#[test]
fn partial() {
    suite(|path| {
        copy_files(path, 0);
        copy_files(path, 1);
        copy_files(path, 2);
    })
}

#[test]
fn full() {
    suite(|path| {
        copy_files(path, 0);
        copy_files(path, 1);
    })
}

#[test]
fn ebb() {
   suite(|path| {
        copy_files(path, 0);
        copy_files(path, 1);
        copy_files(path, 3);
    });
}

#[test]
fn first_slot() {
    suite(|path| {
        copy_files(path, 0);
        copy_files(path, 1);
        copy_files(path, 20);
    });
}

#[test]
fn ebb_ambiguous() {
    suite(|path| {
        copy_files(path, 0);
        copy_files(path, 1);
        copy_files(path, 21);
    });
}

#[test]
fn skipped() {
    suite(|path| {
        copy_files(path, 0);
        copy_files(path, 1);
        copy_files(path, 7600);
    });
}

fn suite(setup: impl Fn(&Path)) {
    let temp_dir = tempfile::tempdir().unwrap();
    setup(temp_dir.path());
    round_trip::<0>(temp_dir.path());
    let temp_dir = tempfile::tempdir().unwrap();
    setup(temp_dir.path());
    round_trip::<1>(temp_dir.path());
    let temp_dir = tempfile::tempdir().unwrap();
    setup(temp_dir.path());
    round_trip::<2>(temp_dir.path());
    let temp_dir = tempfile::tempdir().unwrap();
    setup(temp_dir.path());
    round_trip::<10>(temp_dir.path());
}

fn copy_files(dir: &Path, chunk_number: u32) {
    for file in ["chunk", "primary", "secondary"] {
        let file_name = format!("{chunk_number:05}.{file}");
        let source: PathBuf =
            AsRef::<Path>::as_ref(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/"))
                .join(&file_name);
        let destination = dir.join(&file_name);
        std::fs::copy(source, destination).unwrap();
    }
}

fn round_trip<const N: usize>(path: &std::path::Path) {
    let (mut reader, mut writer) = open::<N>(path).unwrap();
    let mut blocks = Vec::new();

    reader.read_chunk(&mut blocks).unwrap();
    assert!(blocks.is_empty());

    reader.range = 0..100;
    reader.read_chunk(&mut blocks).unwrap();
    for block in &blocks {
        block.decode().unwrap();
    }

    let new_tip = reader.tip().map(|t| t + 1).unwrap_or(0);
    if new_tip > 1 {
        writer
            .append(&block(new_tip - 2, &mut Vec::new()))
            .unwrap_err();
    }
    if new_tip >= crate::CHUNK_SIZE {
        writer
            .append(&block(new_tip - crate::CHUNK_SIZE, &mut Vec::new()))
            .unwrap_err();
    }

    let mut new_block_buffer = Vec::new();
    let new_block = block(new_tip, &mut new_block_buffer);
    writer.append(&new_block).unwrap();

    reader.range = 0..(new_tip + 1);
    while !reader.range.is_empty() {
        blocks.clear();
        reader.read_chunk(&mut blocks).unwrap();
        for block in &blocks {
            block.decode().unwrap();
        }
    }
    assert_eq!(
        &blocks.last().unwrap().decode().unwrap(),
        new_block.as_ref()
    );
    blocks.clear();

    new_block_buffer.clear();
    let new_block = block(new_tip + crate::CHUNK_SIZE, &mut new_block_buffer);
    writer.append(&new_block).unwrap();

    reader.range = (new_tip)..(new_tip + crate::CHUNK_SIZE + 1);
    reader.read_chunk(&mut blocks).unwrap();
    assert!(blocks.len() == 1);
    blocks.clear();

    reader.read_chunk(&mut blocks).unwrap();
    assert_eq!(&blocks[0].decode().unwrap(), new_block.as_ref());
    assert!(blocks.len() == 1);
    blocks.clear();

    new_block_buffer.clear();
    let new_block = block(new_tip + 10 * crate::CHUNK_SIZE, &mut new_block_buffer);
    writer.append(&new_block).unwrap();

    reader.range = (new_tip + 5 * crate::CHUNK_SIZE)..(new_tip + 15 * crate::CHUNK_SIZE);
    for _ in 0..5 {
        reader.read_chunk(&mut blocks).unwrap();
        assert!(blocks.is_empty());
    }
    reader.read_chunk(&mut blocks).unwrap();
    assert_eq!(&blocks[0].decode().unwrap(), new_block.as_ref());
    assert!(blocks.len() == 1);
}

fn block<'a>(slot: slot::Number, buffer: &'a mut Vec<u8>) -> With<'a, Block<'a>> {
    let block = Block::Conway(conway::Block {
        header: block::Header {
            body: header::Body {
                number: 0,
                slot,
                previous: None,
                issuer: &PublicKeyBytes([0; _]),
                vrf: &PublicKeyBytes([0; _]),
                vrf_result: Vrf {
                    output: &[0; _],
                    proof: &[0; _],
                },
                size: 0,
                body_hash: &[0; _],
                certificate: certificate::Operational {
                    // Safety: byte arrays only.
                    signer: &const { unsafe { std::mem::zeroed() } },
                    sequence_number: 0,
                    period: 0,
                    // Safety: byte arrays only.
                    signature: &const { unsafe { std::mem::zeroed() } },
                },
                version: protocol::Version {
                    major: protocol::version::Fork::Chang,
                    minor: 0,
                },
            },
            // Safety: byte arrays only.
            signature: &const { unsafe { std::mem::zeroed() } },
        },
        transaction_bodies: Vec::new(),
        transaction_witness_sets: Vec::new(),
        transaction_data: Unique::new(),
        invalid_transactions: Vec::new(),
    });
    block.encode(&mut Encoder(&mut *buffer));

    With::from((block, buffer.as_slice()))
}
