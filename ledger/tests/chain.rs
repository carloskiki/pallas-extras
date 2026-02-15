// use std::{error::Error, ffi::OsStr, fs::File, io::Read, os::unix::ffi::OsStrExt};
//
// use tinycbor::{Decode, Decoder};
//
// fn main() -> Result<(), Box<dyn Error>> {
//     let mut buffer: Vec<u8> = Vec::new();
//
//     let mut files_ordered = std::fs::read_dir(concat!(
//         env!("CARGO_MANIFEST_DIR"),
//         "/snapshots/mainnet/immutable"
//     ))?
//     .filter_map(|res| {
//         res.ok()
//             .and_then(|f| (f.path().extension() == Some(OsStr::from_bytes(b"chunk"))).then_some(f))
//     })
//     .collect::<Vec<_>>();
//     files_ordered.sort_by_key(|dir_entry| dir_entry.file_name());
//     println!("Found {} files", files_ordered.len());
//
//     let mut era = 0;
//     for file in files_ordered {
//         let file_name_os_str = file.file_name();
//         let file_name = file_name_os_str.to_str().ok_or("invalid file name")?;
//         if !file_name.ends_with(".chunk") || file_name < "05350.chunk" {
//             continue;
//         }
//         if file_name.split('.').next().unwrap().parse::<u32>()? % 50 == 0 {
//             println!("Processing file {file_name}");
//         }
//
//         let mut file = File::open(file.path())?;
//         file.read_to_end(&mut buffer)?;
//         let mut decoder = Decoder(&buffer);
//
//         loop {
//             let bytes = decoder.0;
//             if decoder.0.is_empty() {
//                 break;
//             };
//
//             match ledger::Block::decode(&mut decoder) {
//                 Ok(b) => match b {
//                     ledger::Block::Shelley(_) if era != 1 => {
//                         era = 1;
//                         println!("Entered Shelley era at file {file_name}");
//                     }
//                     ledger::Block::Allegra(_) if era != 2 => {
//                         era = 2;
//                         println!("Entered Allegra era at file {file_name}");
//                     }
//                     ledger::Block::Mary(_) if era != 3 => {
//                         era = 3;
//                         println!("Entered Mary era at file {file_name}");
//                     }
//                     ledger::Block::Alonzo(_) => {
//                         if era != 4 {
//                             era = 4;
//                             println!("Entered Alonzo era at file {file_name}");
//                         }
//                     }
//                     ledger::Block::Babbage(_) => {
//                         if era != 5 {
//                             era = 5;
//                             println!("Entered Babbage era at file {file_name}");
//                         }
//                     }
//                     _ => {}
//                 },
//                 Err(e) => {
//                     let decoder_pos = bytes.len() - decoder.0.len();
//
//                     let next_item = tinycbor::Any::decode(&mut Decoder(&bytes[decoder_pos - 1..]))?;
//                     for token in Decoder(next_item.as_ref()) {
//                         let token = token?;
//                         println!("{token}");
//                     }
//
//                     panic!("{:?}", anyhow::anyhow!(e));
//                 }
//             };
//         }
//
//         buffer.clear();
//     }
//
//     Ok(())
// }

fn main() {
    println!("Placeholder for full chain decoding test.");
}
