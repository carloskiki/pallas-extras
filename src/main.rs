use std::{error::Error, ffi::OsStr, fs::File, io::Read, os::unix::ffi::OsStrExt};

use tinycbor::{Decode, Decoder};

fn main() -> Result<(), Box<dyn Error>> {
    let mut buffer: Vec<u8> = Vec::new();

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
    println!("Found {} files", files_ordered.len());

    for file in files_ordered {
        let file_name_os_str = file.file_name();
        let file_name = file_name_os_str.to_str().ok_or("invalid file name")?;
        if !file_name.ends_with(".chunk") {
            continue;
        }

        let mut file = File::open(file.path())?;
        file.read_to_end(&mut buffer)?;
        let mut decoder = Decoder(&buffer);

        loop {
            let bytes = decoder.0;
            if decoder.0.is_empty() {
                break;
            };

            match ledger::Block::decode(&mut decoder) {
                Ok(_) => {}
                Err(e) => {
                    let next_item = tinycbor::Any::decode(&mut Decoder(bytes))?;
                    for token in Decoder(next_item.as_ref()) {
                        let token = token?;
                        println!("{token}");
                    }

                    panic!("{e}")
                }
            };
        }

        buffer.clear();
    }

    Ok(())
}
