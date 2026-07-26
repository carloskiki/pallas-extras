//! Measure sequential immutable database iteration throughput and memory usage.

use std::{env, error::Error, hint::black_box, io, mem::MaybeUninit, path::PathBuf, time::Instant};

fn main() -> Result<(), Box<dyn Error>> {
    let Some(data) = env::args_os()
        .skip(1)
        // Cargo supplies this argument to custom benchmark harnesses.
        .find(|argument| argument != "--bench")
        .map(PathBuf::from)
    else {
        eprintln!("Skipping benchmark because no database directory was specified.");
        eprintln!("usage: cargo bench --bench iterate -- <database-directory>");
        return Ok(());
    };
    // Sequential iteration does not benefit from retaining completed chunks in the cache.
    let (reader, _) = database::open::<0>(data.clone())?;
    let Some(tip) = reader.tip() else {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "benchmark data is empty").into());
    };

    let start = Instant::now();
    let mut block_count = 0_u64;
    let mut chunks = reader.read(0..tip.saturating_add(1));
    while let Some(blocks) = chunks.next() {
        for block in blocks? {
            let block = block.map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "block checksum mismatch")
            })?;
            black_box(&block);
            block_count += 1;
        }
    }
    let elapsed = start.elapsed();
    let blocks_per_minute = block_count as f64 * 60.0 / elapsed.as_secs_f64();
    let peak_rss_mib = peak_rss_bytes()? as f64 / (1024.0 * 1024.0);

    println!("data: {}", data.display());
    println!("blocks: {block_count}");
    println!("elapsed: {elapsed:.3?}");
    println!("throughput: {blocks_per_minute:.0} blocks/min");
    println!("peak RSS: {peak_rss_mib:.2} MiB");

    Ok(())
}

fn peak_rss_bytes() -> io::Result<u64> {
    let mut usage = MaybeUninit::<libc::rusage>::uninit();
    // Safety: `getrusage` initializes `usage` when it returns success, which is checked below.
    if unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    // Safety: the successful `getrusage` call initialized `usage`.
    let peak_rss = unsafe { usage.assume_init() }.ru_maxrss as u64;

    // Darwin reports bytes; Linux and the other supported Unix targets report KiB.
    #[cfg(target_os = "macos")]
    return Ok(peak_rss);
    #[cfg(not(target_os = "macos"))]
    return Ok(peak_rss * 1024);
}
