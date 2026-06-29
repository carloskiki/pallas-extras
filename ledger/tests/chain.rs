use std::{error::Error, ffi::OsStr, fs::File, io::Read, os::unix::ffi::OsStrExt};

use tinycbor::{Decode, Decoder};

fn main() -> Result<(), Box<dyn Error>> {
    let Ok("true") = std::env::var("PARSE_CHAIN_TEST").as_deref() else {
        println!("Skipping chain parsing test. Set PARSE_CHAIN_TEST=true to run it.");
        return Ok(());
    };

    let mut buffer: Vec<u8> = Vec::new();

    let mut files_ordered = std::fs::read_dir(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../snapshots/mainnet/immutable"
    ))?
    .filter_map(|res| {
        res.ok()
            .and_then(|f| (f.path().extension() == Some(OsStr::from_bytes(b"chunk"))).then_some(f))
    })
    .collect::<Vec<_>>();
    files_ordered.sort_by_key(|dir_entry| dir_entry.file_name());
    println!("Found {} files", files_ordered.len());

    for file in files_ordered {
        let file_name_os_str = file.file_name();
        let file_name = file_name_os_str.to_str().ok_or("invalid file name")?;
        if !file_name.ends_with(".chunk") {
            continue;
        }
        if file_name.split('.').next().unwrap().parse::<u32>()? % 100 == 0 {
            println!("Processing file {file_name}");
        }

        let mut file = File::open(file.path())?;
        file.read_to_end(&mut buffer)?;
        let mut decoder = Decoder(&buffer);

        loop {
            let bytes = decoder.0;
            if decoder.0.is_empty() {
                break;
            };

            if let Err(e) = ledger::Block::decode(&mut decoder) {
                let decoder_pos = bytes.len() - decoder.0.len();

                let next_item = tinycbor::Any::decode(&mut Decoder(&bytes[decoder_pos - 1..]))?;
                for token in Decoder(next_item.as_ref()) {
                    let token = token?;
                    println!("{token}");
                }

                panic!("{:?}", anyhow::anyhow!(e));
            }
        }

        buffer.clear();
    }

    Ok(())
}
