//! Immutable database implementation.
//!
//! This implementation matches the database format used by the `IntersectMBO` implementation.
//! It allows multiple readers and a single writer to operate concurrently. The database is not
//! locked on the file system. It is expected that users of this library ensure that only one
//! instance of the database is open at a time.
//!
//! The design is deliberately very simple and low level, to reduce code size and maximize
//! performance. Readers and the writer almost never block each other, except when truncating or
//! appending at chunk boundaries, which currently happens every 21,600 slots. All operations are
//! syncrhonous (blocking).

use bytes::BytesMut;
use core::sync::atomic;
use crossbeam_utils::CachePadded;
use once_cell::sync::OnceCell;
use std::path::{Path, PathBuf};
use std::{
    cell::UnsafeCell,
    fs::{File, OpenOptions},
    io,
    mem::MaybeUninit,
    os::unix::fs::FileExt,
    sync::{Arc, RwLock, atomic::AtomicU64},
};

// We can provide Windows support by using `seek_read`/`seek_write`.
//
// This is simply not a priority.
#[cfg(not(unix))]
compile_error!("This library only supports on Unix-like systems");

mod reader;
mod secondary;
#[cfg(test)]
mod tests;
mod writer;

pub use reader::Reader;
pub use writer::Writer;

const CHUNK_SIZE: u64 = 21_600;

/// Open a database at the given directory, returning a [`Reader`] and [`Writer`].
///
/// Failure to provide a well-formed database (e.g. missing files, corrupted files) results in
/// unspecified behavior.
///
/// The `N` parameter specifies the number of chunks to cache in memory. A chunk is approximately
/// 16KB-20KB depending on the number of blocks it contains. The `IntersectMBO` implementation uses
/// `N = 500`.
pub fn open<const N: usize>(dir: impl Into<PathBuf>) -> io::Result<(Reader<N>, Writer<N>)> {
    let mut chunk_number = 0;
    let dir = dir.into();
    let dir_iter = std::fs::read_dir(&dir)?;
    for entry in dir_iter {
        let file_name = entry?.file_name();
        if let Some((bytes, rest)) = &file_name.as_encoded_bytes().split_at_checked(5)
            && let Ok(num_str) = std::str::from_utf8(bytes)
            && let Ok(num) = num_str.parse::<u64>()
            && rest == b".secondary"
        {
            chunk_number = chunk_number.max(num);
        }
    }

    let (chunk_file, secondary_file) = open_or_create(&dir, chunk_number)?;
    let data = secondary::read(&mut BytesMut::new(), &secondary_file)?;
    let size = chunk_file.metadata()?.len();

    // This large struct is potentially stored on the stack before being moved to the heap,
    // especially in debug mode. This is < 512KB, so it should be fine, but could be avoided.
    let mut shared = Arc::new(RwLock::new(Cache {
        directory: dir,
        chunk_file,
        secondary_file,
        chunk_number,
        entries: UnsafeCell::new([MaybeUninit::uninit(); CHUNK_SIZE as usize + 1]),
        len_size: CachePadded::from(AtomicU64::from(0)),
        completed: [const { OnceCell::new() }; N],
        pointer: 0,
    }));
    let cache_mut = Arc::get_mut(&mut shared)
        .expect("no other references to shared exist")
        .get_mut()
        .expect("cache is not poisoned");
    let mut len = 0;
    cache_mut
        .entries
        .get_mut()
        .iter_mut()
        .zip(data)
        .for_each(|(entry, block_info)| {
            len += 1;
            entry.write(block_info);
        });
    *cache_mut.len_size.get_mut() = (size << 32) | len;

    Ok((Reader(shared.clone()), Writer(shared)))
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
struct Cache<const N: usize> {
    // FIXME: Use `std::fs::Dir` API when stable.
    directory: PathBuf,
    /// Metadata about previous chunks.
    // FIXME: use `std::sync::OnceLock` when `get_or_try_init` is stable.
    completed: [OnceCell<ChunkData>; N],
    /// points to the oldest chunk in `completed`.
    pointer: usize,
    /// We can have up to `CHUNK_SIZE` regular blocks, plus one EBB.
    entries: UnsafeCell<[MaybeUninit<BlockInfo>; CHUNK_SIZE as usize + 1]>,
    /// The chunk number.
    chunk_number: u64,
    /// The lower 32 bits encode the number of entries in `entries`, and the upper 32 bits encode
    /// the total size of the chunk file.
    len_size: CachePadded<AtomicU64>,
    /// The chunk file is opened in read and write mode.
    chunk_file: File,
    /// The secondary file is opened in read and write mode.
    secondary_file: File,
}

impl<const N: usize> Cache<N> {
    fn len_size(&self, ordering: atomic::Ordering) -> (usize, u32) {
        let len_size = self.len_size.load(ordering);
        ((len_size & 0xFFFF_FFFF) as usize, (len_size >> 32) as u32)
    }

    fn current_chunk_data<'a>(
        &'a self,
        ordering: atomic::Ordering,
    ) -> (&'a [BlockInfo], u32, &'a File) {
        // Syncrhonize with the `Release` store of the writer.
        let (len, size) = self.len_size(ordering);
        // Safety: `len` is the number of initialized entries in `entries`. These are
        // never modified after initialization.
        let block_info = unsafe {
            std::slice::from_raw_parts::<'a, _>(self.entries.get() as *const BlockInfo, len)
        };
        (block_info, size, &self.chunk_file)
    }
}

/// # Safety
///
/// The only `!Sync` field is `entries`. It is handled safely:
/// - The unique writer only modifies `entries[len]`, and then monotonically increases `len` before
///   `Release`.
/// - The readers `Acquire` `len` and only read `entries[..len]`.
/// - Truncation modifies `entries` and decreases `len` only while holding the exclusive cache
///   lock, so no reader or append can access them concurrently.
unsafe impl<const N: usize> Sync for Cache<N> {}

/// Data for a chunk, read from the `secondary` file.
struct ChunkData {
    pub block_info: Box<[BlockInfo]>,
    pub size: u32,
    pub file: File,
}

/// Entry for a block in the cache.
#[derive(Clone, Copy)]
struct BlockInfo {
    slot: u64,
    offset: u32,
    crc: u32,
}

fn path_prefix(path: &Path, chunk_number: u64) -> PathBuf {
    path.join(format!("{chunk_number:05}"))
}

/// Returns the chunk file, primary file, and secondary file for the given chunk number.
///
/// Creates the files if they do not exist.
fn open_or_create(path: &Path, chunk_number: u64) -> io::Result<(File, File)> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true);
    let mut path = path_prefix(path, chunk_number).with_extension("chunk");
    let chunk_file = options.open(&path)?;
    path.set_extension("secondary");
    let secondary_file = options.open(&path)?;
    Ok((chunk_file, secondary_file))
}

/// Reads from the given file, appending the buffer.
fn read_buf(file: &File, buffer: &mut BytesMut, offset: u64, size: usize) -> io::Result<()> {
    buffer.reserve(size);
    // Safety: The `File::read_exact` method only writes to the buffer, the buffer is fully
    // initialized if `read_exact` is successful. This is may be unsound because we `assume_init`
    // uninitialized memory This is what `tokio` does, so it should be fine in practice.
    // 
    // FIXME: once `File::read_exact_buf` is stabilized, we can avoid unsafe. Copied directly
    // from the `tokio` implementation in the meantime.
    unsafe {
        let buf: &mut [u8] = buffer.spare_capacity_mut()[..size].assume_init_mut();
        file.read_exact_at(buf, offset)?;
        buffer.set_len(size);
    }
    Ok(())
}
