use tinycbor::{CborLen, Decode, Encode, Encoder, Write};
use tinycbor_derive::{Encode, Decode, CborLen};

// TODO: Make sure that all three fields are actually used on mainnet.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, CborLen)]
#[cbor(map)]
pub struct Attributes<'a> {
    #[cbor(n(0), optional)]
    distribution: Option<&'a [u8]>,
    #[cbor(n(1), optional)]
    key_derivation_path: Option<&'a [u8]>,
    #[cbor(n(2), with = "NetworkMagic", optional)]
    network_magic: Option<u32>,
}

#[repr(transparent)]
struct NetworkMagic(Option<u32>);

impl From<NetworkMagic> for Option<u32> {
    fn from(nm: NetworkMagic) -> Self {
        nm.0
    }
}

impl From<&Option<u32>> for &NetworkMagic {
    fn from(opt: &Option<u32>) -> Self {
        // This is safe because NetworkMagic is #[repr(transparent)]
        unsafe { &*(opt as *const Option<u32> as *const NetworkMagic) }
    }
}

impl CborLen for NetworkMagic {
    fn cbor_len(&self) -> usize {
        let len = self.0.cbor_len();
        len + len.cbor_len()
    }
}

impl Encode for NetworkMagic {
    fn encode<W: Write>(&self, e: &mut Encoder<W>) -> Result<(), W::Error> {
        let len = self.0.cbor_len();
        // CBOR bytestring with length `len` header: 
        e.0.write_all(&[0x40 | (len as u8)])?; // We know that `len` fits in info since its at most
                                               // 4 bytes.
        self.0.encode(e)
    }
}

impl Decode<'_> for NetworkMagic {
    type Error = tinycbor::num::Error;
    
    fn decode(d: &mut tinycbor::Decoder<'_>) -> Result<Self, Self::Error> {
        let bytes: &[u8] = Decode::decode(d)?;
        let mut decoder = tinycbor::Decoder(bytes);
        let value: u32 = Decode::decode(&mut decoder)?;
        Ok(NetworkMagic(Some(value)))
    }
}
