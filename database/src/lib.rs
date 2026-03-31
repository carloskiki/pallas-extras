use ledger::{Block, slot};
use tokio_stream::Stream;

// Database architecture:
// - Streaming interface for blocks from the DB. Index for
//   get(index) that returns a stream starting at the provided index.
// - Write interface to append blocks to the DB.
//   The write should be done first, and at the very end the primary/secondary file should be
//   updated to have the block indicated as present. If the program crashes at any point while
//   writing, the database should not be corrupted, and the block should simply not appear in the
//   database.
// - Get the tip index.

// Additionally to this number of slots, each chunk can contain an extra EBB block at the start.
const CHUNK_SIZE: usize = 21_600;

// Procedure to read:
// - For each chunk that overlap with the range:
// - Read the start and end from the primary file.
// - Read block metadata from the secondary file.
// - Read blocks from the chunk file.
// - 3 syscalls per chunk file.

// TODO:
// - Lock the database (this could be ensured with a lock on the volatile db)

pub struct Immutable;

impl Immutable {
    pub fn get<R: std::ops::RangeBounds<slot::Number>>(
        range: R,
    ) /* -> impl Stream<Item = Block<'static>> */ {
    }
}
