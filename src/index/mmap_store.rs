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

/// Write index to file (used for persistence).
pub fn write(path: &Path, index: &HnswIndex) -> Result<()> {
    use std::io::Write;
    let count = index.nodes.len();
    let dim = index.dim;

    // Collect nodes in a stable order (by id)
    let mut node_ids: Vec<u32> = index.nodes.keys().copied().collect();
    node_ids.sort();

    // Build ID-to-slot mapping
    let _id_to_slot: std::collections::HashMap<u32, usize> = node_ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();

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
    let version = read_u32(&mmap, &mut pos);
    if version != VERSION {
        // Fall back to v1 reader for backwards compatibility
        if version == 1 {
            return read_v1(&mmap);
        }
        bail!("Unsupported HNSW index version: {version}");
    }

    let dim = read_u32(&mmap, &mut pos) as usize;
    let count = read_u32(&mmap, &mut pos) as usize;
    let max_layer = read_u32(&mmap, &mut pos) as usize;
    let ep_val = read_i64(&mmap, &mut pos);
    let entry_point = if ep_val >= 0 {
        Some(ep_val as u32)
    } else {
        None
    };
    let vectors_offset = read_u64(&mmap, &mut pos) as usize;
    let graph_offset = read_u64(&mmap, &mut pos) as usize;

    // Read node IDs
    let mut node_ids = Vec::with_capacity(count);
    for _ in 0..count {
        node_ids.push(read_u32(&mmap, &mut pos));
    }

    // Read deleted flags
    let mut deleted_flags = Vec::with_capacity(count);
    for _ in 0..count {
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
        let vector: Vec<f32> = mmap[vec_start..vec_end]
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();

        let num_layers = read_u32(&mmap, &mut gpos) as usize;
        let mut neighbors = Vec::with_capacity(num_layers);
        for _ in 0..num_layers {
            let num_neighbors = read_u32(&mmap, &mut gpos) as usize;
            let mut layer = Vec::with_capacity(num_neighbors);
            for _ in 0..num_neighbors {
                layer.push(read_u32(&mmap, &mut gpos));
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

    Ok(HnswIndex {
        nodes,
        entry_point,
        max_layer,
        dim,
    })
}

/// Read v1 format for backwards compatibility.
fn read_v1(data: &[u8]) -> Result<HnswIndex> {
    let mut pos = 4; // skip magic already checked
    let _version = read_u32(data, &mut pos); // 1
    let dim = read_u32(data, &mut pos) as usize;
    let count = read_u32(data, &mut pos) as usize;
    let max_layer = read_u32(data, &mut pos) as usize;
    let ep_val = read_i64(data, &mut pos);
    let entry_point = if ep_val >= 0 {
        Some(ep_val as u32)
    } else {
        None
    };

    let mut nodes = std::collections::HashMap::new();
    for _ in 0..count {
        let id = read_u32(data, &mut pos);
        let deleted = data[pos] != 0;
        pos += 1;

        let mut vector = Vec::with_capacity(dim);
        for _ in 0..dim {
            vector.push(read_f32(data, &mut pos));
        }

        let num_layers = read_u32(data, &mut pos) as usize;
        let mut neighbors = Vec::with_capacity(num_layers);
        for _ in 0..num_layers {
            let num_neighbors = read_u32(data, &mut pos) as usize;
            let mut layer = Vec::with_capacity(num_neighbors);
            for _ in 0..num_neighbors {
                layer.push(read_u32(data, &mut pos));
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
    })
}

fn read_u32(data: &[u8], pos: &mut usize) -> u32 {
    let val = u32::from_le_bytes(data[*pos..*pos + 4].try_into().unwrap());
    *pos += 4;
    val
}

fn read_i64(data: &[u8], pos: &mut usize) -> i64 {
    let val = i64::from_le_bytes(data[*pos..*pos + 8].try_into().unwrap());
    *pos += 8;
    val
}

fn read_u64(data: &[u8], pos: &mut usize) -> u64 {
    let val = u64::from_le_bytes(data[*pos..*pos + 8].try_into().unwrap());
    *pos += 8;
    val
}

fn read_f32(data: &[u8], pos: &mut usize) -> f32 {
    let val = f32::from_le_bytes(data[*pos..*pos + 4].try_into().unwrap());
    *pos += 4;
    val
}
