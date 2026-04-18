use anyhow::{bail, Context, Result};
use memmap2::Mmap;
use std::path::Path;

use super::hnsw::HnswIndex;

// Binary format v2 (zero-copy friendly):
// Header (fixed size):
//   [magic: 4 bytes "MALD"]
//   [version: u32]
//   [dim: u32]
//   [count: u32]
//   [max_layer: u32]
//   [entry_point: i64 (-1 if none)]
//   [vectors_offset: u64]   — byte offset to contiguous vector block
//   [graph_offset: u64]     — byte offset to adjacency data
//
// Node ID table (count * u32):
//   [id0, id1, id2, ...]
//
// Deleted flags (count * u8):
//   [del0, del1, ...]
//
// Vectors block (at vectors_offset, count * dim * f32):
//   contiguous f32 array, node i's vector at [i * dim .. (i+1) * dim]
//
// Graph block (at graph_offset):
//   For each node:
//     [num_layers: u32]
//     For each layer:
//       [num_neighbors: u32]
//       [neighbor_ids: num_neighbors * u32]

const MAGIC: &[u8; 4] = b"MALD";
const VERSION: u32 = 2;
const HEADER_SIZE: usize = 4 + 4 + 4 + 4 + 4 + 8 + 8 + 8; // 44 bytes

/// Get the current index format version.
/// Useful for migration tooling and diagnostics.
pub const fn current_version() -> u32 {
    VERSION
}

/// Check if an index file is compatible with the current version.
/// Returns Ok(version) if compatible, Err with details if not.
pub fn check_version(path: &std::path::Path) -> Result<u32> {
    let file = std::fs::File::open(path).context("Failed to open index file for version check")?;
    let mmap =
        unsafe { memmap2::Mmap::map(&file) }.context("Failed to mmap index for version check")?;

    if mmap.len() < 8 {
        bail!("Index file too small to contain valid header");
    }
    if &mmap[0..4] != MAGIC {
        bail!("Invalid index file: missing MALD magic header. File may be corrupted or not a MALD index.");
    }

    let mut vpos = 4usize;
    let version = read_u32(&mmap, &mut vpos)?;

    if version > VERSION {
        bail!(
            "Index version {version} is newer than supported version {VERSION}. Please upgrade MALD."
        );
    }

    Ok(version)
}

/// Write index to file (used for persistence).
pub fn write(path: &Path, index: &HnswIndex) -> Result<()> {
    use std::io::Write;
    let count = index.nodes.len();
    let dim = index.dim;

    // Collect nodes in a stable order (by id)
    let mut node_ids: Vec<u32> = index.nodes.keys().copied().collect();
    node_ids.sort();

    let ids_size = count * 4;
    let deleted_size = count;
    let vectors_offset = HEADER_SIZE + ids_size + deleted_size;
    let vectors_size = count * dim * 4;
    let graph_offset = vectors_offset + vectors_size;

    // Build graph data
    let mut graph_buf = Vec::new();
    for &id in &node_ids {
        let node = &index.nodes[&id];
        graph_buf.write_all(&(node.neighbors.len() as u32).to_le_bytes())?;
        for layer in &node.neighbors {
            graph_buf.write_all(&(layer.len() as u32).to_le_bytes())?;
            for &nid in layer {
                graph_buf.write_all(&nid.to_le_bytes())?;
            }
        }
    }

    let total_size = graph_offset + graph_buf.len();
    let mut buf = Vec::with_capacity(total_size);

    // Header
    buf.write_all(MAGIC)?;
    buf.write_all(&VERSION.to_le_bytes())?;
    buf.write_all(&(dim as u32).to_le_bytes())?;
    buf.write_all(&(count as u32).to_le_bytes())?;
    buf.write_all(&(index.max_layer as u32).to_le_bytes())?;
    let ep: i64 = index.entry_point.map(|e| e as i64).unwrap_or(-1);
    buf.write_all(&ep.to_le_bytes())?;
    buf.write_all(&(vectors_offset as u64).to_le_bytes())?;
    buf.write_all(&(graph_offset as u64).to_le_bytes())?;

    // Node IDs
    for &id in &node_ids {
        buf.write_all(&id.to_le_bytes())?;
    }

    // Deleted flags
    for &id in &node_ids {
        buf.push(index.nodes[&id].deleted as u8);
    }

    // Vectors (contiguous block)
    for &id in &node_ids {
        for &val in &index.nodes[&id].vector {
            buf.write_all(&val.to_le_bytes())?;
        }
    }

    // Graph adjacency
    buf.extend_from_slice(&graph_buf);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, &buf).context("Failed to write HNSW index")?;
    Ok(())
}

/// Read index from file. Vectors are accessed via mmap for zero-copy reads
/// during search. Graph structure is deserialized into memory.
pub fn read(path: &Path) -> Result<HnswIndex> {
    let file = std::fs::File::open(path).context("Failed to open HNSW index")?;
    let mmap = unsafe { Mmap::map(&file) }.context("Failed to mmap HNSW index")?;

    if mmap.len() < HEADER_SIZE {
        bail!("HNSW index file too small");
    }
    if &mmap[0..4] != MAGIC {
        bail!("Invalid HNSW index file (bad magic)");
    }

    let mut pos = 4;
    let version = read_u32(&mmap, &mut pos)?;
    if version != VERSION {
        // Fall back to v1 reader for backwards compatibility
        if version == 1 {
            tracing::info!("Migrating index from v1 to v2 format");
            return read_v1(&mmap);
        }
        if version > VERSION {
            bail!(
                "Index version {version} is newer than supported version {VERSION}. Please upgrade MALD to read this index."
            );
        }
        bail!(
            "Unsupported HNSW index version {version}. Expected version {VERSION}. Try rebuilding the index with `mald reindex`."
        );
    }

    let dim = read_u32(&mmap, &mut pos)? as usize;
    let count = read_u32(&mmap, &mut pos)? as usize;
    let max_layer = read_u32(&mmap, &mut pos)? as usize;
    let ep_val = read_i64(&mmap, &mut pos)?;
    let entry_point = if ep_val >= 0 {
        Some(ep_val as u32)
    } else {
        None
    };
    let vectors_offset = read_u64(&mmap, &mut pos)? as usize;
    let graph_offset = read_u64(&mmap, &mut pos)? as usize;

    // Sanity checks: prevent OOM from corrupt headers
    let file_len = mmap.len();
    if dim == 0 || dim > 4096 {
        bail!("Invalid dimension {dim} in index header (expected 1-4096). Index may be corrupted.");
    }
    if max_layer > 64 {
        bail!("Invalid max_layer {max_layer} in index header (expected 0-64). Index may be corrupted.");
    }
    // count * (4 bytes ID + 1 byte deleted + dim*4 bytes vector) must fit in the file
    let min_data_per_node = 4 + 1 + dim * 4;
    if count > 0 && count > file_len / min_data_per_node {
        bail!(
            "Invalid node count {count} for file size {file_len} (would require at least {} bytes). Index may be corrupted.",
            count as u64 * min_data_per_node as u64
        );
    }
    if vectors_offset > file_len || graph_offset > file_len {
        bail!(
            "Invalid offsets in header: vectors_offset={vectors_offset}, graph_offset={graph_offset}, file_size={file_len}. Index may be corrupted."
        );
    }

    // Read node IDs
    let mut node_ids = Vec::with_capacity(count);
    for _ in 0..count {
        node_ids.push(read_u32(&mmap, &mut pos)?);
    }

    // Read deleted flags
    let mut deleted_flags = Vec::with_capacity(count);
    for _ in 0..count {
        if pos >= mmap.len() {
            bail!(
                "Index truncated: expected deleted flag at offset {pos}, but file is only {} bytes",
                mmap.len()
            );
        }
        deleted_flags.push(mmap[pos] != 0);
        pos += 1;
    }

    // Read vectors via zero-copy slicing from mmap
    // Each vector is dim * 4 bytes starting at vectors_offset
    let mut nodes = std::collections::HashMap::with_capacity(count);

    // Read graph adjacency
    let mut gpos = graph_offset;
    for i in 0..count {
        let id = node_ids[i];

        // Zero-copy vector read: interpret mmap bytes as f32 slice
        let vec_start = vectors_offset + i * dim * 4;
        let vec_end = vec_start + dim * 4;
        if vec_end > mmap.len() {
            bail!(
                "Index truncated: vector {} at offset {vec_start}..{vec_end} exceeds file size {}",
                i,
                mmap.len()
            );
        }
        let vector: Vec<f32> = mmap[vec_start..vec_end]
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();

        let num_layers = read_u32(&mmap, &mut gpos)? as usize;
        let mut neighbors = Vec::with_capacity(num_layers);
        for _ in 0..num_layers {
            let num_neighbors = read_u32(&mmap, &mut gpos)? as usize;
            let mut layer = Vec::with_capacity(num_neighbors);
            for _ in 0..num_neighbors {
                layer.push(read_u32(&mmap, &mut gpos)?);
            }
            neighbors.push(layer);
        }

        nodes.insert(
            id,
            super::hnsw::Node {
                id,
                vector,
                neighbors,
                deleted: deleted_flags[i],
            },
        );
    }

    // Compute deleted_count from flags
    let deleted_count = deleted_flags.iter().filter(|&&d| d).count();

    Ok(HnswIndex {
        nodes,
        entry_point,
        max_layer,
        dim,
        deleted_count,
    })
}

/// Read v1 format for backwards compatibility.
fn read_v1(data: &[u8]) -> Result<HnswIndex> {
    let mut pos = 4; // skip magic already checked
    let _version = read_u32(data, &mut pos)?; // 1
    let dim = read_u32(data, &mut pos)? as usize;
    let count = read_u32(data, &mut pos)? as usize;
    let max_layer = read_u32(data, &mut pos)? as usize;
    let ep_val = read_i64(data, &mut pos)?;
    let entry_point = if ep_val >= 0 {
        Some(ep_val as u32)
    } else {
        None
    };

    let mut nodes = std::collections::HashMap::new();
    let mut deleted_count = 0usize;
    for _ in 0..count {
        let id = read_u32(data, &mut pos)?;
        if pos >= data.len() {
            bail!("V1 index truncated: expected deleted flag at offset {pos}");
        }
        let deleted = data[pos] != 0;
        if deleted {
            deleted_count += 1;
        }
        pos += 1;

        let mut vector = Vec::with_capacity(dim);
        for _ in 0..dim {
            vector.push(read_f32(data, &mut pos)?);
        }

        let num_layers = read_u32(data, &mut pos)? as usize;
        let mut neighbors = Vec::with_capacity(num_layers);
        for _ in 0..num_layers {
            let num_neighbors = read_u32(data, &mut pos)? as usize;
            let mut layer = Vec::with_capacity(num_neighbors);
            for _ in 0..num_neighbors {
                layer.push(read_u32(data, &mut pos)?);
            }
            neighbors.push(layer);
        }

        nodes.insert(
            id,
            super::hnsw::Node {
                id,
                vector,
                neighbors,
                deleted,
            },
        );
    }

    Ok(HnswIndex {
        nodes,
        entry_point,
        max_layer,
        dim,
        deleted_count,
    })
}

fn read_u32(data: &[u8], pos: &mut usize) -> Result<u32> {
    let end = *pos + 4;
    if end > data.len() {
        bail!(
            "Index truncated: expected u32 at offset {}, but file is only {} bytes",
            *pos,
            data.len()
        );
    }
    let val = u32::from_le_bytes(data[*pos..end].try_into().unwrap());
    *pos = end;
    Ok(val)
}

fn read_i64(data: &[u8], pos: &mut usize) -> Result<i64> {
    let end = *pos + 8;
    if end > data.len() {
        bail!(
            "Index truncated: expected i64 at offset {}, but file is only {} bytes",
            *pos,
            data.len()
        );
    }
    let val = i64::from_le_bytes(data[*pos..end].try_into().unwrap());
    *pos = end;
    Ok(val)
}

fn read_u64(data: &[u8], pos: &mut usize) -> Result<u64> {
    let end = *pos + 8;
    if end > data.len() {
        bail!(
            "Index truncated: expected u64 at offset {}, but file is only {} bytes",
            *pos,
            data.len()
        );
    }
    let val = u64::from_le_bytes(data[*pos..end].try_into().unwrap());
    *pos = end;
    Ok(val)
}

fn read_f32(data: &[u8], pos: &mut usize) -> Result<f32> {
    let end = *pos + 4;
    if end > data.len() {
        bail!(
            "Index truncated: expected f32 at offset {}, but file is only {} bytes",
            *pos,
            data.len()
        );
    }
    let val = f32::from_le_bytes(data[*pos..end].try_into().unwrap());
    *pos = end;
    Ok(val)
}

#[cfg(test)]
mod tests {
    use super::super::hnsw::HnswIndex;
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_current_version() {
        assert_eq!(current_version(), 2);
    }

    #[test]
    fn test_write_then_read_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test_index.bin");
        let dim = 4;

        let mut index = HnswIndex::new(dim);
        index.insert(0, vec![1.0, 0.0, 0.0, 0.0]);
        index.insert(1, vec![0.0, 1.0, 0.0, 0.0]);
        index.insert(2, vec![0.0, 0.0, 1.0, 0.0]);

        write(&path, &index).unwrap();
        assert!(path.exists());

        let loaded = read(&path).unwrap();
        assert_eq!(loaded.dim, dim);
        assert_eq!(loaded.len(), 3);
        assert!(loaded.entry_point.is_some());
    }

    #[test]
    fn test_write_then_read_empty_index() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("empty_index.bin");
        let dim = 8;

        let index = HnswIndex::new(dim);
        write(&path, &index).unwrap();

        let loaded = read(&path).unwrap();
        assert_eq!(loaded.dim, dim);
        assert_eq!(loaded.len(), 0);
        assert!(loaded.is_empty());
        assert!(loaded.entry_point.is_none());
    }

    #[test]
    fn test_check_version_valid_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("valid.bin");
        let dim = 4;

        let index = HnswIndex::new(dim);
        write(&path, &index).unwrap();

        let version = check_version(&path).unwrap();
        assert_eq!(version, VERSION);
    }

    #[test]
    fn test_check_version_wrong_magic() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("bad_magic.bin");

        // Write a file with wrong magic bytes but valid size
        let data = b"BAAD\x02\x00\x00\x00extra_padding_data";
        std::fs::write(&path, data).unwrap();

        let result = check_version(&path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("magic") || err_msg.contains("Invalid"),
            "Error should mention invalid magic, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_check_version_file_too_small() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("tiny.bin");

        // Write only 4 bytes (less than needed for magic + version)
        std::fs::write(&path, b"MALD").unwrap();

        let result = check_version(&path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("too small") || err_msg.contains("truncated"),
            "Error should mention file too small, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_roundtrip_preserves_vectors() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("vectors.bin");
        let dim = 4;

        let mut index = HnswIndex::new(dim);
        let vec_a = vec![0.1, 0.2, 0.3, 0.4];
        let vec_b = vec![0.5, 0.6, 0.7, 0.8];
        index.insert(10, vec_a.clone());
        index.insert(20, vec_b.clone());

        write(&path, &index).unwrap();
        let loaded = read(&path).unwrap();

        // Verify vectors are preserved
        let node_10 = loaded.nodes.get(&10).expect("Node 10 should exist");
        let node_20 = loaded.nodes.get(&20).expect("Node 20 should exist");

        for (a, b) in node_10.vector.iter().zip(vec_a.iter()) {
            assert!((a - b).abs() < 1e-6, "Vector values should be preserved");
        }
        for (a, b) in node_20.vector.iter().zip(vec_b.iter()) {
            assert!((a - b).abs() < 1e-6, "Vector values should be preserved");
        }
    }

    #[test]
    fn test_read_nonexistent_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.bin");
        let result = read(&path);
        assert!(result.is_err());
    }
}
