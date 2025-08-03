use std::{error::Error, fs::File, io::Read};

use const_hex::FromHex;
use indicatif::ProgressIterator;
use minicbor::{CborLen as _, Decoder, Encode, Encoder};

const SKIPS: &[u16] = &[7779];

// First byte is version, then all quadruples are offsets
// Offsets in primary are u32s
//
// Primary File profile:
// Secondary offsets = 56
// slots per chunk = 21602
// Version = 1
//
//
// Secondary file layout
// Should be 56 bytes
// Block offset: u64 BE - 8
// Header offset: u16 BE - 2
// Header size: u16 BE - 2
// CRC: to figure u32 BE - 4
// Header hash: 32 bytes
// block or EBB: u64 BE (either slot or epoch number, depending if EBB) - 8

fn main() -> Result<(), Box<dyn Error>> {
    let mut buffer = Vec::new();
    let expected = 56;
    let offsets_expected = 21602;

    for file in std::fs::read_dir(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/snapshots/preprod/immutable"
    ))? {
        let file = file?;
        let os_file_name = file.file_name();
        let file_name = os_file_name.to_str().unwrap();
        let path = file.path();
        if !file_name.ends_with(".primary") {
            continue
        }

        let mut file = std::fs::File::open(path)?;
        file.read_to_end(&mut buffer)?;
        let offsets = buffer[1..]
            .chunks(4)
            .map(|bytes| {
                u32::from_be_bytes(bytes.try_into().unwrap())
            })
            .collect::<Vec<_>>();
        if offsets_expected != offsets.len() {
            println!("things: {}", file_name);
        }
        
        let gap = offsets.windows(2).all(|slice| {
            slice[0] == slice[1] || slice[1] - slice[0] == expected
        });
        assert!(gap);
        buffer.clear()
    }

    
    Ok(())
}

fn empty_asset<T>(asset: &ledger::Asset<T>) -> bool {
    asset.0.is_empty() || asset.0.iter().any(|(_, bundles)| bundles.0.is_empty())
}

// Basically eras + 1, because the tag 0 is Byron EBB block
fn db_block_thing(
    d: &mut minicbor::Decoder<'_>,
) -> Result<Option<(ledger::Block, u8)>, minicbor::decode::Error> {
    cbor_util::array_decode(
        2,
        |d| {
            let tag = d.u8()?;
            if tag < 2 {
                return Ok(None);
            }
            Ok(Some((d.decode()?, tag)))
        },
        d,
    )
}

fn inspect_tokens(cbor: &[u8]) {
    for token in Decoder::new(cbor).tokens() {
        let token = token.unwrap();
        println!("{}", token);
    }
}

// Conway first block: 2344009
