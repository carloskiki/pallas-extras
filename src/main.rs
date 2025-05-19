use std::{error::Error, fs::File, io::Read};

use indicatif::ProgressIterator;
use minicbor::{CborLen as _, Decoder, Encode, Encoder};
use network::WithEncoded;
use const_hex::FromHex;

const SKIPS: &[u16] = &[7779];

fn main() -> Result<(), Box<dyn Error>> {
    let mut buffer = Vec::new();
    let mut encoder = Encoder::new(Vec::new());
    
    for file in std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/db/immutable"))?.progress_count(55671) {
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
            let block = match db_block_thing(&mut decoder) {
                Err(e) if e.is_end_of_input() => break,
                Err(e) => {
                    inspect_tokens(&decoder.input()[start..decoder.position()]);

                    return Err(e.into());
                }
                Ok(WithEncoded {
                    value: block,
                    encoded,
                }) => block,
            };
            encoder.encode(&block.transaction_bodies)?;
            let bytes = encoder.writer();
            let encoded = bytes.len();
            let calculated = block.transaction_bodies.cbor_len(&mut ());
            if  encoded != calculated {
                inspect_tokens(bytes);
                
                println!("Actual len: {encoded}, calculated: {calculated}");
                panic!();
            }
            encoder.writer_mut().clear()
        }
        buffer.clear();
    }
    Ok(())
}

// Basically eras + 1, because the tag 0 is Byron EBB block
fn db_block_thing(
    d: &mut minicbor::Decoder<'_>,
) -> Result<WithEncoded<ledger::Block>, minicbor::decode::Error> {
    cbor_util::array_decode(
        2,
        |d| {
            let _ = d.u8()?;
            d.decode()
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
