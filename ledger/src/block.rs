pub mod header;

pub use header::Header;
use minicbor::{CborLen, Decode, Encode};

use crate::{transaction, witness};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, CborLen)]
pub struct Block {
    #[n(0)]
    pub header: Header,
    #[cbor(n(1), with = "cbor_util::boxed_slice")]
    pub transaction_bodies: Box<[transaction::Body]>,
    #[cbor(n(2), with = "cbor_util::boxed_slice")]
    pub witness_sets: Box<[witness::Set]>,
    #[cbor(n(3), with = "cbor_util::list_as_map")]
    pub auxiliary_data: Box<[(u16, transaction::Data)]>,
    #[cbor(
        n(4),
        decode_with = "cbor_util::boxed_slice::decode",
        nil = "cbor_util::boxed_slice::nil",
        encode_with = "cbor_util::boxed_slice::encode"
    )]
    pub invalid_transactions: Box<[u16]>,
}


pub type Number = u64;

// enum ByronBlock {
//     Ebb {
//       head: EbbHead,  
//       stake_holders: Box<[Blake2b224Digest]>,
//       extra: Box<[Box<[u8]>]>
//     },
//     Main {
//         head: ByronHead,
//     },
// }
// 
// struct ByronHead {
//     protocol_magic: u32,
//     previous_block: Blake2b256Digest,
//     body_proof: BlockProof,
// }
// 
// struct BlockProof {
//     tx_proof: TxProof,
//     ssc_proof: SscProof,
//     dlg_proof: Blake2b256Digest,
//     upd_proof: Blake2b256Digest,
// }
// 
// struct TxProof(u32, Blake2b256Digest, Blake2b256Digest);
// 
// enum SscProof {
//     A(Blake2b256Digest, Blake2b256Digest),
//     B(Blake2b256Digest, Blake2b256Digest),
//     C(Blake2b256Digest, Blake2b256Digest),
//     D(Blake2b256Digest),
// }
// 
// struct EbbHead {
//     protocol_magic: u32,
//     previous_block: Blake2b256Digest,
//     body_proof: Blake2b256Digest,
//     concensus_data: EbbConcensusData,
//     extra_data: Box<[Box<[u8]>]>
// }
// 
// struct EbbConcensusData {
//     epoch: epoch::Number,
//     difficulty: u64, // Array with one element
// }
// 
// struct BlockConcensusData {
//     slot_id: SlotId,
//     verifying_key: Box<[u8]>, 
// 
// }
// 
// struct SlotId {
//     epoch: epoch::Number,
//     slot: slot::Number,
// }
