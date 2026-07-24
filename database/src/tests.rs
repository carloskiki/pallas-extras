use crate::{CHUNK_SIZE, open};
use bytes::Bytes;
use ledger::Block;
use std::path::{Path, PathBuf};
use tinycbor::{Decode, Decoder};

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
        copy_files(path, 2);
        copy_files(path, 3);
    });
}

#[test]
fn first_slot() {
    suite(|path| {
        copy_files(path, 0);
        copy_files(path, 1);
        copy_files(path, 2);
        copy_files(path, 3);
        copy_files(path, 4);
    });
}

#[test]
fn ebb_ambiguous() {
    suite(|path| {
        copy_files(path, 0);
        copy_files(path, 1);
        copy_files(path, 2);
        copy_files(path, 3);
        copy_files(path, 4);
        copy_files(path, 5);
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
    for file in ["chunk", "secondary"] {
        let file_name = format!("{chunk_number:05}.{file}");
        let source: PathBuf =
            AsRef::<Path>::as_ref(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/"))
                .join(&file_name);
        let destination = dir.join(&file_name);
        std::fs::copy(source, destination).unwrap();
    }
}

fn round_trip<const N: usize>(path: &std::path::Path) {
    let (reader, mut writer) = open::<N>(path).unwrap();
    let new_tip = reader.tip().map(|t| t + 1).unwrap_or(0);
    const NEW_BLOCK: &[u8] = b"new block";

    {
        let mut read = reader.read(0..100);
        while let Some(chunk) = read.next() {
            for block in chunk.unwrap() {
                Block::decode(&mut Decoder(&block.unwrap())).unwrap();
            }
        }

        if new_tip > 1 {
            writer.append(&[], 0..0, [0; _], 0).unwrap_err();
        }
        if new_tip >= CHUNK_SIZE {
            writer.append(&[], 0..0, [0; _], CHUNK_SIZE).unwrap_err();
        }
    }

    writer.append(NEW_BLOCK, 0..2, [0; _], new_tip).unwrap();

    {
        let mut read = reader.read(0..(new_tip + 1));
        let mut last_block = Bytes::new();
        while let Some(chunk) = read.next() {
            last_block = chunk
                .unwrap()
                .inspect(|block| {
                    if block.is_err() {
                        panic!("block is invalid");
                    }
                })
                .last()
                .unwrap()
                .unwrap();
        }
        assert_eq!(&last_block, NEW_BLOCK,);
    }

    writer
        .append(NEW_BLOCK, 0..2, [0; _], new_tip + CHUNK_SIZE)
        .unwrap();

    {
        let mut read = reader.read(new_tip..(new_tip + CHUNK_SIZE + 1));
        let mut chunk = read.next().unwrap().unwrap();
        let block = chunk.next().unwrap().unwrap();
        assert_eq!(&block, NEW_BLOCK);
        assert!(chunk.next().is_none());
        let mut chunk = read.next().unwrap().unwrap();
        let block = chunk.next().unwrap().unwrap();
        assert_eq!(&block, NEW_BLOCK);
        assert!(chunk.next().is_none());
        assert!(read.next().is_none());
    }

    writer
        .append(NEW_BLOCK, 0..2, [0; _], new_tip + CHUNK_SIZE * 5)
        .unwrap_err();
}
