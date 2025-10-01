use std::{
    error::Error,
    ffi::{OsStr, OsString},
    fs::File,
    io::Read,
    os::unix::ffi::OsStrExt,
};

use const_hex::FromHex;
use indicatif::ProgressIterator;
use minicbor::{CborLen as _, Decoder, Encode, Encoder};
use network::WithEncoded;

fn main() -> Result<(), Box<dyn Error>> {
    let mut buffer = Vec::new();

    let mut files_ordered = std::fs::read_dir(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/snapshots/mainnet/immutable"
    ))?
    .filter_map(|res| {
        res.ok()
            .and_then(|f| (f.path().extension() == Some(OsStr::from_bytes(b"chunk"))).then_some(f))
    })
    .collect::<Vec<_>>();
    files_ordered.sort_by_key(|dir_entry| dir_entry.file_name());
    let mut prev_era: Option<ledger::protocol::Era> = None;
    println!("Found {} files", files_ordered.len());

    for file in files_ordered.iter().progress() {
        let file_name_os_str = file.file_name();
        let file_name = file_name_os_str.to_str().ok_or("invalid file name")?;
        if !file_name.ends_with(".chunk") {
            continue;
        }

        let mut file = File::open(file.path())?;
        file.read_to_end(&mut buffer)?;
        let mut decoder = Decoder::new(&buffer);
        loop {
            if decoder.input()[decoder.position()..].is_empty() {
                break;
            };

            let (era, block) = cbor_util::array_decode(
                2,
                |d| {
                    let int = d.u8()?;
                    let era = match int {
                        0 | 1 => ledger::protocol::Era::Byron,
                        2 => ledger::protocol::Era::Shelley,
                        3 => ledger::protocol::Era::Allegra,
                        4 => ledger::protocol::Era::Mary,
                        5 => ledger::protocol::Era::Alonzo,
                        6 => ledger::protocol::Era::Babbage,
                        7 => ledger::protocol::Era::Conway,
                        _ => return Err(minicbor::decode::Error::message("invalid era")),
                    };
                    let block = if era > ledger::protocol::Era::Byron {
                        let start = d.position();
                        match d.decode::<ledger::Block>() {
                            Ok(b) => Some(b),
                            Err(e) => {
                                if let Some(pos) = e.position() {
                                    inspect_tokens(&d.input()[start..pos], Some(&d.input()[pos..]));
                                } else {
                                    inspect_tokens(&d.input()[start..], None);
                                }
                                println!("Error decoding block in file {}: {:?}", file_name, e);
                                None
                            }
                        }
                    } else {
                        d.skip()?;
                        None
                    };

                    Ok((era, block))
                },
                &mut decoder,
            )?;
            
            if Some(era) != prev_era {
                println!(
                    "New Era for block #{:?} in file {} : {:?}",
                    block.map(|b| b.header.body.block_number),
                    file_name,
                    era
                );
                prev_era = Some(era);
            }
        }

        buffer.clear();
    }
    Ok(())
}

// Basically eras + 1, because the tag 0 is Byron EBB block
fn db_block_thing(
    d: &mut minicbor::Decoder<'_>,
) -> Result<(ledger::protocol::Era, WithEncoded<ledger::Block>), minicbor::decode::Error> {
    cbor_util::array_decode(
        2,
        |d| {
            let int = d.u8()?;
            let era = match int {
                0 | 1 => ledger::protocol::Era::Byron,
                2 => ledger::protocol::Era::Shelley,
                3 => ledger::protocol::Era::Allegra,
                4 => ledger::protocol::Era::Mary,
                5 => ledger::protocol::Era::Alonzo,
                6 => ledger::protocol::Era::Babbage,
                7 => ledger::protocol::Era::Conway,
                _ => return Err(minicbor::decode::Error::message("invalid era")),
            };
            Ok((era, d.decode()?))
        },
        d,
    )
}

fn inspect_tokens(cbor: &[u8], after: Option<&[u8]>) {
    for token in Decoder::new(cbor).tokens() {
        let token = token.unwrap();
        println!("{}", token);
    }

    if let Some(after) = after {
        println!("===After:===");
        for token in Decoder::new(&after[..128]).tokens() {
            let token = token.unwrap();
            println!("{}", token);
        }
    }
}
