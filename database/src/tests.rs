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

#[test]
fn truncate() {
    truncate_round_trip::<0>();
    truncate_round_trip::<1>();
    truncate_round_trip::<2>();
    truncate_preserves_cache();
    truncate_to_empty();
    truncate_to_ebb();
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

fn truncate_round_trip<const N: usize>() {
    const ZERO: &[u8] = b"zero";
    const ONE: &[u8] = b"one";
    const BOUNDARY: &[u8] = b"boundary";
    const LATER: &[u8] = b"later";
    const LAST_CHUNK: &[u8] = b"last chunk";
    const REPLACEMENT: &[u8] = b"replacement";

    let temp_dir = tempfile::tempdir().unwrap();
    let (reader, mut writer) = open::<N>(temp_dir.path()).unwrap();
    writer.append(ZERO, 0..0, [0; _], 0).unwrap();
    writer.append(ONE, 0..0, [0; _], 1).unwrap();
    writer.append(BOUNDARY, 0..0, [0; _], CHUNK_SIZE).unwrap();
    writer.append(LATER, 0..0, [0; _], CHUNK_SIZE + 1).unwrap();
    writer
        .append(LAST_CHUNK, 0..0, [0; _], CHUNK_SIZE * 2)
        .unwrap();

    writer.truncate(CHUNK_SIZE).unwrap();
    assert_eq!(reader.tip(), Some(CHUNK_SIZE));
    assert_eq!(
        read_blocks(&reader, 0..(CHUNK_SIZE + 1)),
        [ZERO, ONE, BOUNDARY]
    );
    assert!(!temp_dir.path().join("00002.secondary").exists());
    assert!(!temp_dir.path().join("00002.chunk").exists());

    writer
        .append(REPLACEMENT, 0..0, [0; _], CHUNK_SIZE + 1)
        .unwrap();
    writer.append(LATER, 0..0, [0; _], CHUNK_SIZE + 2).unwrap();
    writer.truncate(CHUNK_SIZE + 1).unwrap();
    writer.truncate(CHUNK_SIZE * 10).unwrap();
    assert_eq!(reader.tip(), Some(CHUNK_SIZE + 1));
    assert_eq!(
        read_blocks(&reader, 0..(CHUNK_SIZE + 2)),
        [ZERO, ONE, BOUNDARY, REPLACEMENT]
    );

    drop(writer);
    drop(reader);
    let (reader, mut writer) = open::<N>(temp_dir.path()).unwrap();
    assert_eq!(reader.tip(), Some(CHUNK_SIZE + 1));
    assert_eq!(
        read_blocks(&reader, 0..(CHUNK_SIZE + 2)),
        [ZERO, ONE, BOUNDARY, REPLACEMENT]
    );

    writer.truncate(0).unwrap();
    assert_eq!(reader.tip(), Some(0));
    assert_eq!(read_blocks(&reader, 0..1), [ZERO]);
}

fn truncate_to_empty() {
    let temp_dir = tempfile::tempdir().unwrap();
    let (reader, mut writer) = open::<0>(temp_dir.path()).unwrap();
    writer.append(b"later", 0..0, [0; _], 10).unwrap();

    writer.truncate(9).unwrap();
    assert_eq!(reader.tip(), None);
    assert_eq!(
        std::fs::metadata(temp_dir.path().join("00000.chunk"))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        std::fs::metadata(temp_dir.path().join("00000.secondary"))
            .unwrap()
            .len(),
        0
    );

    writer.append(b"replacement", 0..0, [0; _], 9).unwrap();
    assert_eq!(reader.tip(), Some(9));
}

fn truncate_preserves_cache() {
    let temp_dir = tempfile::tempdir().unwrap();
    let (reader, mut writer) = open::<3>(temp_dir.path()).unwrap();
    for chunk_number in 0..=5 {
        writer
            .append(
                &[chunk_number as u8],
                0..0,
                [0; _],
                chunk_number * CHUNK_SIZE,
            )
            .unwrap();
    }

    writer.truncate(3 * CHUNK_SIZE).unwrap();
    {
        let cache = reader.0.read().unwrap();
        assert_eq!(cache.pointer, 0);
        assert_eq!(
            cache
                .completed
                .each_ref()
                .map(|entry| entry.get().is_some()),
            [false, false, true]
        );
    }

    // Chunk 2 comes from the preserved entry. Chunk 1 must be reloaded rather than resolving to
    // the stale entry for the removed chunk 4.
    assert_eq!(
        read_blocks(&reader, CHUNK_SIZE..(3 * CHUNK_SIZE)),
        [Bytes::from_static(&[1]), Bytes::from_static(&[2])]
    );
}

fn truncate_to_ebb() {
    let temp_dir = tempfile::tempdir().unwrap();
    for chunk_number in 0..=4 {
        copy_files(temp_dir.path(), chunk_number);
    }
    let (reader, mut writer) = open::<2>(temp_dir.path()).unwrap();
    let boundary = CHUNK_SIZE * 3;

    writer.truncate(boundary).unwrap();
    assert_eq!(reader.tip(), Some(boundary));
    assert_eq!(read_blocks(&reader, boundary..(boundary + 1)).len(), 1);
    assert!(!temp_dir.path().join("00004.secondary").exists());
    assert!(!temp_dir.path().join("00004.chunk").exists());

    writer
        .append(b"first block after EBB", 0..0, [0; _], boundary)
        .unwrap();
    assert_eq!(reader.tip(), Some(boundary));
    assert_eq!(read_blocks(&reader, boundary..(boundary + 1)).len(), 2);
}

fn read_blocks<const N: usize>(
    reader: &crate::Reader<N>,
    range: std::ops::Range<u64>,
) -> Vec<Bytes> {
    let mut read = reader.read(range);
    let mut blocks = Vec::new();
    while let Some(chunk) = read.next() {
        blocks.extend(chunk.unwrap().map(Result::unwrap));
    }
    blocks
}
