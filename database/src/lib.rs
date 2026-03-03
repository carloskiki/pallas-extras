// Database architecture:
// - Streaming interface for blocks from the DB. Index for
//   get(index) that returns a stream starting at the provided index.
// - Write interface to append blocks to the DB.
//   The write should be done first, and at the very end the primary/secondary file should be
//   updated to have the block indicated as present. If the program crashes at any point while
//   writing, the database should not be corrupted, and the block should simply not appear in the
//   database.
// - Get the tip index.
