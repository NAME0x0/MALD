use anyhow::Result;

use crate::index::hnsw::HnswIndex;

/// Benchmark the HNSW vector index: insert and query throughput.
pub async fn run(dim: usize, count: usize) -> Result<()> {
    println!("MALD Benchmark — HNSW Vector Index\n");
    println!("Dimension: {}, Vectors: {}\n", dim, count);

    // Generate random vectors
    let mut rng = rand_simple();
    let vectors: Vec<Vec<f32>> = (0..count)
        .map(|_| (0..dim).map(|_| next_f32(&mut rng)).collect())
        .collect();

    // Insert benchmark
    let mut index = HnswIndex::new(dim);
    let start = std::time::Instant::now();
    for (i, v) in vectors.iter().enumerate() {
        index.insert(i as u32, v.clone());
    }
    let insert_elapsed = start.elapsed();
    let insert_per_sec = count as f64 / insert_elapsed.as_secs_f64();
    println!(
        "Insert: {} vectors in {:.2}ms ({:.0} vec/s)",
        count,
        insert_elapsed.as_secs_f64() * 1000.0,
        insert_per_sec
    );

    // Query benchmark
    let queries = 100usize;
    let k = 10;
    let query_vectors: Vec<Vec<f32>> = (0..queries)
        .map(|_| (0..dim).map(|_| next_f32(&mut rng)).collect())
        .collect();

    let start = std::time::Instant::now();
    for qv in &query_vectors {
        let _ = index.search(qv, k);
    }
    let query_elapsed = start.elapsed();
    let query_per_sec = queries as f64 / query_elapsed.as_secs_f64();
    let avg_ms = query_elapsed.as_secs_f64() * 1000.0 / queries as f64;
    println!(
        "Search: {} queries in {:.2}ms ({:.0} q/s, {:.3}ms avg)",
        queries,
        query_elapsed.as_secs_f64() * 1000.0,
        query_per_sec,
        avg_ms
    );

    // Persistence benchmark
    let tmp = std::env::temp_dir().join("mald-bench-hnsw.bin");
    let start = std::time::Instant::now();
    index.save(&tmp)?;
    let save_elapsed = start.elapsed();
    let file_size = std::fs::metadata(&tmp).map(|m| m.len()).unwrap_or(0);

    let start = std::time::Instant::now();
    let _loaded = HnswIndex::load(&tmp)?;
    let load_elapsed = start.elapsed();
    let _ = std::fs::remove_file(&tmp);

    println!(
        "Save:   {:.2}ms ({:.1} KB)",
        save_elapsed.as_secs_f64() * 1000.0,
        file_size as f64 / 1024.0
    );
    println!("Load:   {:.2}ms", load_elapsed.as_secs_f64() * 1000.0,);

    println!("\nAll benchmarks completed.");
    Ok(())
}

// Simple deterministic RNG (xorshift32) to avoid pulling in rand crate overhead
fn rand_simple() -> u32 {
    42
}

fn next_f32(state: &mut u32) -> f32 {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    (*state as f32) / u32::MAX as f32
}
