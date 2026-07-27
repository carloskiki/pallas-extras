//! Measure sequential immutable database iteration throughput and memory usage.

use std::{env, error::Error, hint::black_box, io, path::PathBuf, time::Instant};
use sysinfo::{Process, ProcessRefreshKind, ProcessesToUpdate, System, get_current_pid};

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
    let rss_mib = rss_bytes()? as f64 / (1024.0 * 1024.0);

    println!("data: {}", data.display());
    println!("blocks: {block_count}");
    println!("elapsed: {elapsed:.3?}");
    println!("throughput: {blocks_per_minute:.0} blocks/min");
    println!("RSS at completion: {rss_mib:.2} MiB");

    Ok(())
}

fn rss_bytes() -> io::Result<u64> {
    let pid = get_current_pid().map_err(io::Error::other)?;
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        false,
        ProcessRefreshKind::nothing().with_memory(),
    );
    let process: &Process = system.process(pid).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "current process is unavailable from sysinfo",
        )
    })?;
    Ok(process.memory())
}
