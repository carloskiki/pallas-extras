use std::{error::Error, fs::File, io::Read};

use minicbor::{Decoder, Encode, Encoder};
use network::WithEncoded;

const SKIPS: &[u16] = &[7779];

fn main() -> Result<(), Box<dyn Error>> {
    let mut buffer = Vec::new();
    let mut encoder = Encoder::new(Vec::new());
    for file in std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/db/immutable"))? {
        let file_data = file?;
        let file_name_os_str = file_data.file_name();
        let file_name = file_name_os_str.to_str().ok_or("invalid file name")?;
        if !file_name.ends_with(".chunk") {
            continue;
        }

        let mut file = File::open(file_data.path())?;
        file.read_to_end(&mut buffer)?;
        let mut decoder = Decoder::new(&buffer);
        loop {
            let start = decoder.position();
            let block = match network::hard_fork_combinator::decode::<WithEncoded<ledger::Block>, _>(
                &mut decoder,
                &mut (),
            ) {
                Err(e) if e.is_end_of_input() => break,
                Err(e) => {
                    inspect_tokens(&decoder.input()[start..decoder.position()]);
                    println!("{}", decoder.position());
                    
                    return Err(e.into())
                },
                Ok((
                    WithEncoded {
                        value: block,
                        encoded,
                    },
                    _,
                )) => block,
            };
            if block.header.body.block_number == 2344009 {
                println!("Jackpot!! {}", file_name);
            }

        }
        buffer.clear();
        // block.encode(&mut encoder, &mut ())?;

        // if encoder.writer().as_slice() != &*encoded && !SKIPS.contains(&file_name.strip_suffix(".chunk").unwrap().parse()?) {
        //     let x = Decoder::new(encoder.writer().as_slice())
        //         .tokens()
        //         .collect::<Vec<_>>();
        //     let y = Decoder::new(&encoded).tokens().collect::<Vec<_>>();
        //     for (x, y) in x.into_iter().zip(y.into_iter()) {
        //         let x = x?;
        //         let y = y?;
        //         if x != y {
        //             println!("Expected: {:#?}, got: {:#?}", y, x);
        //             println!("For {}", file_name);
        //             panic!();
        //         }
        //         println!("{:#?}", x);
        //     }
        // }
        // encoder.writer_mut().clear();
        // if decoder.input()[decoder.position()..].is_empty() {
        //     break;
        // }
    }
    Ok(())
}

fn inspect_tokens(cbor: &[u8]) {
    dbg!(Decoder::new(cbor).tokens().collect::<Vec<_>>());
}

// Conway first block: 2344009
