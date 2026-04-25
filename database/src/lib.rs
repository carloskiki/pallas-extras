use core::range::RangeInclusive;
use std::{
    cell::{Cell, UnsafeCell},
    fs::File,
    future::Future,
    io,
    mem::MaybeUninit,
    ops::{Range, RangeBounds},
    os::unix::fs::FileExt,
    path::PathBuf,
    pin::{Pin, pin},
    sync::{
        Arc, RwLock,
        atomic::{self, AtomicU32, AtomicU64, AtomicUsize},
    },
    task::{Poll, ready},
    vec::IntoIter,
};

use bytes::{Bytes, BytesMut};
use ledger::{Block, slot};
use tinycbor::encoded::{Lazy, With};
use tokio_stream::Stream;
use zerocopy::IntoBytes;

// We can provide Windows support by using `seek_read`/`seek_write`.
//
// This is simply not a priority.
#[cfg(not(unix))]
compile_error!("This library is only supported on Unix-like systems");

mod chunk;
mod primary;
mod secondary;

/// Reader for blocks from the database.
pub struct Reader<const N: usize> {
    state: State<N>,
    range: Range<slot::Number>,
}

/// State of the reader.
enum State<const N: usize> {
    /// Reading blocks from a chunk file.
    Pending {
        /// Handle to the reading task.
        handle: tokio::task::JoinHandle<io::Result<Blocks<N>>>,
        /// Whether the reading task has been cancelled.
        ///
        /// This happens if the `Reader` stops being polled and its block range is changed. In this
        /// case the current reading task is no longer relevant.
        cancelled: bool,
    },
    /// The blocks are ready to be returned by the stream.
    Ready(Blocks<N>),
}

struct Blocks<const N: usize> {
    /// The shared data.
    ///
    /// This is always `Some`. `None` simply allows `shared` to be `std::mem::take`n in the reading
    /// task, without needing to clone the `Arc`.
    shared: Option<Arc<Shared<N>>>,
    /// Buffer used to reading from files.
    buffer: BytesMut,
    /// Blocks read that have not yet been returned by the stream.
    ///
    /// They are stored in reverse order, and popped from the back.
    blocks: Vec<Lazy<Bytes, Block<'static>>>,
}

struct Shared<const N: usize> {
    // FIXME: Use `std::fs::Dir` API when stable.
    directory: PathBuf,
    cache: RwLock<Cache<N>>,
}

/// Cache maintaining metadata about the current chunk and `N` most recently filled chunks.
///
/// This saves 5 syscalls for accessing the `secondary` file, and opening and closing the
/// `chunk` file.
///
/// ```txt
/// Without cache: `open(secondary)`, `read(secondary)`, `open(chunk)`, `read(chunk)`,
/// `close(chunk)`, `close(secondary)`.
///
/// With cache: `read(chunk)`.
/// ```
/// Note: These syscalls do not happen per block, but per chunk.
pub struct Cache<const N: usize> {
    chunks: [Option<chunk::Data>; N],
    pointer: usize,
    current: Current,
}

impl<const N: usize> Cache<N> {
    pub fn from_slot(dir: impl Into<PathBuf>, slot: slot::Number) -> Self {
        todo!()
    }

    pub fn new(dir: impl Into<PathBuf>) -> io::Result<Self> {
        let mut last_chunk = 0;
        let dir = dir.into();
        let dir_iter = std::fs::read_dir(&dir)?;
        let file_name_err = |file_name: &std::ffi::OsString| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid file name in database directory: {file_name:?}"),
            )
        };
        for entry in dir_iter {
            let file_name = entry?.file_name();
            let bytes = &file_name.as_encoded_bytes()[..5];
            let num = std::str::from_utf8(bytes)
                .map_err(|_| file_name_err(&file_name))?
                .parse::<u32>()
                .map_err(|_| file_name_err(&file_name))?;
            last_chunk = last_chunk.max(num);
        }

        todo!();
    }
}

// Database architecture:
// - Streaming interface for blocks from the DB. Index for
//   get(index) that returns a stream starting at the provided index.
// - Write interface to append blocks to the DB.
//   The write should be done first, and at the very end the primary/secondary file should be
//   updated to have the block indicated as present. If the program crashes at any point while
//   writing, the database should not be corrupted, and the block should simply not appear in the
//   database.
// - Get the tip index.

const CHUNK_SIZE: slot::Number = 21_600;

// Procedure to read:
// - For each chunk that overlap with the range:
// - Get the primary file from path.
// - Get primary file size.
// - Get start and end offsets.
// - Read the start and end from the primary file.
// - Read block metadata from the secondary file.
// - Read blocks from the chunk file.
// - 6 syscalls per chunk file.

// TODO:
// - Lock the database (this could be ensured with a lock on the volatile db)

pub struct Writer<const N: usize>(Arc<Shared<N>>);

impl<const N: usize> Stream for Reader<N> {
    type Item = io::Result<Lazy<Bytes, Block<'static>>>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let self_ = self.get_mut();
        match &mut self_.state {
            State::Ready(Blocks {
                blocks,
                buffer,
                shared,
            }) => {
                if let Some(block) = blocks.pop() {
                    return Poll::Ready(Some(Ok(block)));
                }

                if self_.range.is_empty() {
                    return Poll::Ready(None);
                }

                let Some(shared) = shared.take() else {
                    unreachable!();
                };
                let mut buffer = std::mem::take(buffer);
                let blocks = std::mem::take(blocks);
                let range = self_.range.clone();
                let chunk_number = (range.start / CHUNK_SIZE) as u32;
                let join_handle = tokio::task::spawn_blocking(move || -> io::Result<Blocks<N>> {
                    let cache = shared
                        .cache
                        .read()
                        .expect("writer should not panic while holding the cache lock");
                     if chunk_number > cache.current.chunk_number {
                        // The chunk does not exist yet.
                        drop(cache);
                        return Ok(Blocks {
                            shared: Some(shared),
                            buffer,
                            blocks,
                        });
                    }
                    
                    let (block_info, size, file) = if chunk_number == cache.current.chunk_number {
                        // Syncrhonize with the `Release` store of the writer.
                        let len_size = cache.current.len_size.load(atomic::Ordering::Acquire);
                        let len = (len_size & 0xFFFF_FFFF) as usize;
                        let size = (len_size >> 32) as u32;
                        // Safety: `len` is the number of initialized entries in `entries`. These are
                        // never modified after initialization, so we can safely read them.
                        let block_info = unsafe {
                            std::slice::from_raw_parts(
                                cache.current.entries.get() as *const BlockInfo,
                                len,
                            )
                        };

                        (block_info, size, &cache.current.chunk_file)
                    } else {
                        let (mut index, underflow) = cache.pointer.overflowing_sub(
                            (cache.current.chunk_number - chunk_number) as usize,
                        );
                        if underflow {
                            index = index.wrapping_add(N);
                        }
                        cache.chunks.get_mut(index).and_then(|c| c.as_ref()).ok_or_else(|| {
                            io::Error::new(
                                io::ErrorKind::NotFound,
                                format!("chunk file for chunk {chunk_number} not found in cache"),
                            )
                        })?
                        
                        && let Some(chunk::Data {
                            block_info,
                            size,
                            file,
                        }) = {
                            cache.chunks.get(index).and_then(|c| c.as_ref())
                        }
                    }
                    
                    todo!()
                        
                });

                // Spawn the task and change the state.
                todo!()

                    
            }
            State::Pending { handle, cancelled } => {
                let mut new_blocks = ready!(Pin::new(handle).poll(cx))??;
                if *cancelled {
                    std::hint::cold_path();
                    new_blocks.blocks.clear();
                } else {
                    self_.state = State::Ready(new_blocks);
                    self_.range.start =
                        (self_.range.start + CHUNK_SIZE - 1) / CHUNK_SIZE * CHUNK_SIZE;
                }
            }
        }
        Pin::new(self_).poll_next(cx)
    }
}

/// Entry for a chunk in the cache.
#[derive(Clone, Copy)]
pub struct BlockInfo {
    slot: slot::Number,
    offset: u32,
    crc: u32,
}

// TODO: restructure and ensure this is true.
// SAFETY: `Current` is `Sync` because the only method that internally mutates `Current` is
// `append`, and the safety contract ensures that there are no concurrent calls to `append`.
unsafe impl Sync for Current {}

struct Current {
    /// We can have up to `CHUNK_SIZE` regular blocks, plus one EBB.
    entries: UnsafeCell<[MaybeUninit<BlockInfo>; CHUNK_SIZE as usize + 1]>,
    /// The chunk number.
    chunk_number: u32,
    /// The lower 32 bits encode the number of entries in `entries`, and the upper 32 bits encode
    /// the total size of the chunk file.
    len_size: AtomicU64,
    /// The chunk file is opened in read and write mode.
    chunk_file: File,
    /// The primary file is opened in write mode.
    primary_file: File,
    /// The secondary file is opened in read and write mode.
    secondary_file: File,
}

impl Current {
    /// Get the last slot in the current chunk.
    ///
    /// This panic if the current chunk is empty. The current chunk should never be empty, unless
    fn current_slot(&self) -> slot::Number {
        let Some(last) = self.len.load(atomic::Ordering::Acquire).checked_sub(1) else {};

        // Safety: `len` is the number of initialized entries in `entries`, so `entries[last]`
        // is initialized.
        let last_entry = unsafe { (&*self.entries.get())[last].assume_init() };
        last_entry.slot + 1
    }

    /// Append a block to the current chunk.
    ///
    /// This function does not enforce the slot number of the block to fit within this chunk.
    ///
    /// ### Safety
    ///
    /// This function internally mutates `Self`. This allows concurrent reads to the cache,
    /// making things much faster.
    ///
    /// As such, `append` must not be called concurrently with itself. The caller
    /// must ensure that calls to this function are syncrhonized externally, for example by
    /// enforcing a single writer, or by using a mutex.
    unsafe fn append(
        &self,
        block: &[u8],
        last_relative_slot: u64,
        back_fill_count: usize,
        entry: secondary::Entry,
    ) -> io::Result<()> {
        self.chunk_file.write_all_at(block, entry.offset.get())?;

        let len_size = self.len_size.load(atomic::Ordering::Relaxed);
        let len = (len_size & 0xFFFF_FFFF) as usize;

        let secondary_offset = (len * std::mem::size_of::<secondary::Entry>()) as u32;
        let offsets = secondary_offset.to_be_bytes().repeat(back_fill_count);
        self.secondary_file
            .write_all_at(&offsets, last_relative_slot * 4)?;

        self.secondary_file
            .write_all_at(entry.as_bytes(), secondary_offset as u64)?;

        // Safety:
        // - `append` is the only function that accesses `entries` at `len`.
        // - The caller ensures that there are no concurrent calls to `append`.
        //   => We have exclusive access to `entries[len]`.
        unsafe { (&mut *self.entries.get())[len].write(entry.block_info()) };

        // `Release` so that readers see the new entry if they see the new `len_size`.
        let new_len_size = len_size + 1 + ((block.len() as u64) << 32);
        self.len_size.store(new_len_size, atomic::Ordering::Release);

        Ok(())
    }
}

fn read_buf(file: &File, buffer: &mut BytesMut, offset: u64, size: usize) -> io::Result<()> {
    buffer.clear();
    buffer.reserve(size);
    // Safety: The `File::read_exact` method only writes to the buffer, the buffer is fully
    // initialized if `read_exact` is successful. This is theoretically unsound because we
    // `assume_init` uninitialized memory. This is what `tokio` does, so it should be fine in
    // practice.
    // FIXME: once `File::read_exact_buf` is stabilized, we can avoid unsafe. Copied directly
    // from the `tokio` implementation in the meantime.
    let buf: &mut [u8] = unsafe { buffer.spare_capacity_mut()[..size].assume_init_mut() };
    file.read_exact_at(buf, offset)?;
    // Safety: The buffer is initilized up to `size` after the read.
    unsafe { buffer.set_len(size) };
    Ok(())
}

/// Drop guard that aborts the process when dropped.
struct AbortOnDrop;
impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        std::process::abort();
    }
}
