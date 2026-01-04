use std::{
    error::Error,
    ffi::{OsStr, OsString},
    fs::File,
    io::Read,
    os::unix::ffi::OsStrExt,
};

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
        let mut start = [0u8; 2];
        file.read_exact(&mut start)?;

        if start[1] > 1 {
            println!("Shelly era: {file_name}");
            continue;
        }
        
        // file.read_to_end(&mut buffer)?;
        // let mut decoder = Decoder(&buffer);
        // loop {
        //     if decoder.0.is_empty() {
        //         break;
        //     };

        //     match ledger::byron::Block::decode(&mut decoder) {
        //         Ok(_) => {
        //             println!("Decoded block");
        //         }
        //         Err(e) => panic!("{e}"),
        //     };
        // }
        // 
        // buffer.clear();
    }
    Ok(())
}
