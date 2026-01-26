use const_hex::FromHex;
use ledger::Block;
use minicbor::Decoder;
use std::{fs::File, io::Read};


fn hfc(decoder: &mut Decoder<'_>) -> anyhow::Result<()> {
    let len = decoder.array()?;
    assert!(len == Some(2));
    decoder.u8()?;
    
    Ok(())
}

#[test]
fn block() -> anyhow::Result<()> {
    let mut read_buffer = Vec::new();
    let mut write_buffer: Vec<u8> = Vec::new();
    for entry in std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/"))? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        
        if !entry.file_type()?.is_file()
            || file_name.contains("byron")
            || file_name.contains("genesis")
            || file_name.contains("conway") {
                continue;
        }

        File::open(entry.path())?.read_to_end(&mut read_buffer)?;
        let binary: Vec<u8> = FromHex::from_hex(&read_buffer)?;
        
        let mut decoder = minicbor::Decoder::new(&binary);

        hfc(&mut decoder)?;
        let block: Block = decoder.decode()?;
        minicbor::encode(&block, &mut write_buffer)?;
        assert_eq!(&read_buffer, &write_buffer);

        read_buffer.clear();
        write_buffer.clear();
    }

    Ok(())
}
