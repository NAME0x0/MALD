use anyhow::Result;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::path::Path;
use wide::f32x8;

const M: usize = 16; // max connections per layer
const M_MAX0: usize = 32; // max connections at layer 0
const EF_CONSTRUCTION: usize = 200;
const EF_SEARCH: usize = 50;
fn ml() -> f64 {
    1.0 / (M as f64).ln()
}

#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct Node {
    pub(crate) id: u32,
    pub(crate) vector: Vec<f32>,
    pub(crate) neighbors: Vec<Vec<u32>>,
    pub(crate) deleted: bool,
}

pub struct HnswIndex {
    pub(crate) nodes: HashMap<u32, Node>,
    pub(crate) entry_point: Option<u32>,
    pub(crate) max_layer: usize,
    pub(crate) dim: usize,
    /// Cached count of deleted nodes for O(1) len() computation.
    pub(crate) deleted_count: usize,
}

/// Candidate for min-heap (closest first when popped).
/// Used for the exploration frontier.
#[derive(PartialEq, Clone, Copy)]
struct Candidate {
    id: u32,
    dist: f32,
}

impl Eq for Candidate {}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Min-heap: reverse ordering (smallest dist = highest priority)
        other
            .dist
            .partial_cmp(&self.dist)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

/// Candidate for max-heap (farthest first when popped).
/// Used for the results set to efficiently evict worst candidates.
#[derive(PartialEq, Clone, Copy)]
struct MaxCandidate {
    id: u32,
    dist: f32,
}

impl Eq for MaxCandidate {}

impl PartialOrd for MaxCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MaxCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Max-heap: normal ordering (largest dist = highest priority)
        self.dist
            .partial_cmp(&other.dist)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

/// SIMD-accelerated cosine distance using 8-wide f32 vectors.
/// Falls back to scalar for non-aligned remainders.
/// Returns 1.0 - cosine_similarity (0.0 = identical, 2.0 = opposite).
#[inline]
fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "vector dimension mismatch");

    let chunks = a.len() / 8;
    let remainder = a.len() % 8;

    let mut dot_acc = f32x8::ZERO;
    let mut norm_a_acc = f32x8::ZERO;
    let mut norm_b_acc = f32x8::ZERO;

    // Process 8 elements at a time with SIMD
    for i in 0..chunks {
        let offset = i * 8;
        let va = f32x8::new(a[offset..offset + 8].try_into().unwrap());
        let vb = f32x8::new(b[offset..offset + 8].try_into().unwrap());

        dot_acc += va * vb;
        norm_a_acc += va * va;
        norm_b_acc += vb * vb;
    }

    // Horizontal sum of SIMD accumulators
    let arr_dot: [f32; 8] = dot_acc.into();
    let arr_a: [f32; 8] = norm_a_acc.into();
    let arr_b: [f32; 8] = norm_b_acc.into();

    let mut dot: f32 = arr_dot.iter().sum();
    let mut norm_a: f32 = arr_a.iter().sum();
    let mut norm_b: f32 = arr_b.iter().sum();

    // Handle remainder with scalar ops
    let tail_start = chunks * 8;
    for i in 0..remainder {
        let idx = tail_start + i;
        dot += a[idx] * b[idx];
        norm_a += a[idx] * a[idx];
        norm_b += b[idx] * b[idx];
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        1.0
    } else {
        1.0 - (dot / denom)
    }
}

fn random_level() -> usize {
    let r = fastrand::f64().max(f64::MIN_POSITIVE);
    (-r.ln() * ml()).floor() as usize
}

impl HnswIndex {
    pub fn new(dim: usize) -> Self {
        Self {
            nodes: HashMap::new(),
            entry_point: None,
            max_layer: 0,
            dim,
            deleted_count: 0,
        }
    }

    /// Returns the count of non-deleted nodes. O(1) via cached counter.
    pub fn len(&self) -> usize {
        self.nodes.len() - self.deleted_count
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn insert(&mut self, id: u32, vector: Vec<f32>) {
        assert_eq!(vector.len(), self.dim);
        let level = random_level();

        let mut node = Node {
            id,
            vector,
            neighbors: vec![Vec::new(); level + 1],
            deleted: false,
        };

        if self.entry_point.is_none() {
            self.entry_point = Some(id);
            self.max_layer = level;
            self.nodes.insert(id, node);
            return;
        }

        let ep = self.entry_point.unwrap();
        let mut current = ep;

        // Greedy search from top layer down to level+1
        for l in (level + 1..=self.max_layer).rev() {
            current = self.greedy_closest(current, &node.vector, l);
        }

        // For layers min(level, max_layer) down to 0, search and connect
        let insert_layers = level.min(self.max_layer);
        for l in (0..=insert_layers).rev() {
            let neighbors = self.search_layer(current, &node.vector, EF_CONSTRUCTION, l);

            let m = if l == 0 { M_MAX0 } else { M };
            let selected: Vec<u32> = neighbors.iter().take(m).map(|c| c.id).collect();

            node.neighbors[l] = selected.clone();

            // Add bidirectional connections
            for &neighbor_id in &selected {
                if let Some(neighbor) = self.nodes.get_mut(&neighbor_id) {
                    if l < neighbor.neighbors.len() {
                        neighbor.neighbors[l].push(id);
                        if neighbor.neighbors[l].len() > m {
                            // Prune neighbors: collect IDs and compute distances without cloning vectors
                            let neighbor_ids: Vec<u32> = neighbor.neighbors[l].clone();
                            let mut scored: Vec<(u32, f32)> =
                                Vec::with_capacity(neighbor_ids.len());

                            // Get reference to neighbor's vector for distance computation
                            let nv = &self.nodes[&neighbor_id].vector;
                            for &nid in &neighbor_ids {
                                let d = self
                                    .nodes
                                    .get(&nid)
                                    .map(|n2| cosine_distance(nv, &n2.vector))
                                    .unwrap_or(f32::MAX);
                                scored.push((nid, d));
                            }
                            scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                            scored.truncate(m);
                            // Re-borrow mutably to assign
                            if let Some(neighbor) = self.nodes.get_mut(&neighbor_id) {
                                neighbor.neighbors[l] =
                                    scored.into_iter().map(|(nid, _)| nid).collect();
                            }
                        }
                    }
                }
            }

            if !neighbors.is_empty() {
                current = neighbors[0].id;
            }
        }

        if level > self.max_layer {
            self.max_layer = level;
            self.entry_point = Some(id);
        }

        self.nodes.insert(id, node);
    }

    pub fn search(&self, query: &[f32], k: usize) -> Vec<(u32, f32)> {
        if self.entry_point.is_none() {
            return Vec::new();
        }

        let ep = self.entry_point.unwrap();
        let mut current = ep;

        // Greedy descent from top to layer 1
        for l in (1..=self.max_layer).rev() {
            current = self.greedy_closest(current, query, l);
        }

        // Search layer 0
        let candidates = self.search_layer(current, query, EF_SEARCH.max(k), 0);

        candidates
            .into_iter()
            .filter(|c| self.nodes.get(&c.id).map(|n| !n.deleted).unwrap_or(false))
            .take(k)
            .map(|c| (c.id, c.dist))
            .collect()
    }

    pub fn delete(&mut self, id: u32) {
        if let Some(node) = self.nodes.get_mut(&id) {
            if !node.deleted {
                node.deleted = true;
                self.deleted_count += 1;
            }
        }
    }

    fn greedy_closest(&self, start: u32, query: &[f32], layer: usize) -> u32 {
        let mut current = start;
        let mut best_dist = self
            .nodes
            .get(&current)
            .map(|n| cosine_distance(query, &n.vector))
            .unwrap_or(f32::MAX);

        loop {
            let mut changed = false;
            // Avoid clone: collect neighbor IDs first, then iterate
            let neighbor_ids: Vec<u32> = self
                .nodes
                .get(&current)
                .and_then(|n| n.neighbors.get(layer))
                .map(|v| v.as_slice())
                .unwrap_or(&[])
                .to_vec();

            for nid in neighbor_ids {
                if let Some(neighbor) = self.nodes.get(&nid) {
                    let d = cosine_distance(query, &neighbor.vector);
                    if d < best_dist {
                        best_dist = d;
                        current = nid;
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        current
    }

    /// Search a single layer using beam search with ef candidates.
    /// Optimized: uses max-heap for results to avoid O(n log n) sort per insertion.
    fn search_layer(&self, start: u32, query: &[f32], ef: usize, layer: usize) -> Vec<Candidate> {
        let start_dist = self
            .nodes
            .get(&start)
            .map(|n| cosine_distance(query, &n.vector))
            .unwrap_or(f32::MAX);

        let mut visited = HashSet::new();
        visited.insert(start);

        // Min-heap for exploration frontier (closest candidates to explore next)
        let mut candidates = BinaryHeap::new();
        candidates.push(Candidate {
            id: start,
            dist: start_dist,
        });

        // Max-heap for results (farthest at top for efficient eviction)
        let mut results: BinaryHeap<MaxCandidate> = BinaryHeap::new();
        results.push(MaxCandidate {
            id: start,
            dist: start_dist,
        });

        while let Some(closest) = candidates.pop() {
            // If closest candidate is farther than the worst result, stop
            // O(1) peek instead of O(n) fold
            if results.len() >= ef {
                if let Some(worst) = results.peek() {
                    if closest.dist > worst.dist {
                        break;
                    }
                }
            }

            // Collect neighbor IDs to avoid borrow conflict
            let neighbor_ids: Vec<u32> = self
                .nodes
                .get(&closest.id)
                .and_then(|n| n.neighbors.get(layer))
                .map(|v| v.as_slice())
                .unwrap_or(&[])
                .to_vec();

            for nid in neighbor_ids {
                if visited.contains(&nid) {
                    continue;
                }
                visited.insert(nid);

                if let Some(neighbor) = self.nodes.get(&nid) {
                    let d = cosine_distance(query, &neighbor.vector);

                    // O(1) peek to get worst distance
                    let dominated =
                        results.len() >= ef && results.peek().map(|w| d >= w.dist).unwrap_or(false);

                    if !dominated {
                        candidates.push(Candidate { id: nid, dist: d });
                        results.push(MaxCandidate { id: nid, dist: d });

                        // Evict worst if over capacity — O(log ef)
                        if results.len() > ef {
                            results.pop();
                        }
                    }
                }
            }
        }

        // Convert max-heap to sorted vec (ascending by distance)
        let mut sorted: Vec<Candidate> = results
            .into_iter()
            .map(|mc| Candidate {
                id: mc.id,
                dist: mc.dist,
            })
            .collect();
        sorted.sort_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap());
        sorted
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        use crate::index::mmap_store;
        mmap_store::write(path, self)
    }

    pub fn load(path: &Path) -> Result<Self> {
        use crate::index::mmap_store;
        mmap_store::read(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn random_vec(dim: usize) -> Vec<f32> {
        (0..dim).map(|_| fastrand::f32() - 0.5).collect()
    }

    #[test]
    fn test_insert_and_search() {
        let dim = 32;
        let mut index = HnswIndex::new(dim);

        // Insert 100 vectors
        for i in 0..100 {
            index.insert(i, random_vec(dim));
        }
        assert_eq!(index.len(), 100);

        // Search should return results
        let query = random_vec(dim);
        let results = index.search(&query, 5);
        assert_eq!(results.len(), 5);

        // Results should be sorted by distance
        for w in results.windows(2) {
            assert!(w[0].1 <= w[1].1);
        }
    }

    #[test]
    fn test_delete() {
        let dim = 8;
        let mut index = HnswIndex::new(dim);
        index.insert(0, vec![1.0; dim]);
        index.insert(1, vec![0.5; dim]);
        assert_eq!(index.len(), 2);

        index.delete(0);
        assert_eq!(index.len(), 1);

        let results = index.search(&vec![1.0; dim], 10);
        assert!(!results.iter().any(|(id, _)| *id == 0));
    }

    #[test]
    fn test_cosine_finds_similar() {
        let dim = 8;
        let mut index = HnswIndex::new(dim);

        // Insert a known vector and something very similar
        let target = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let similar = vec![0.9, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let different = vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0];

        index.insert(0, target.clone());
        index.insert(1, similar);
        index.insert(2, different);

        let results = index.search(&target, 3);
        assert_eq!(results[0].0, 0); // exact match first
        assert_eq!(results[1].0, 1); // similar second
    }

    #[test]
    fn test_persistence_roundtrip() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("test.bin");
        let dim = 16;

        let mut index = HnswIndex::new(dim);
        for i in 0..50 {
            index.insert(i, random_vec(dim));
        }
        index.save(&path).unwrap();

        let loaded = HnswIndex::load(&path).unwrap();
        assert_eq!(loaded.len(), 50);
        assert_eq!(loaded.dim, dim);

        // Search should still work
        let query = random_vec(dim);
        let r1 = index.search(&query, 5);
        let r2 = loaded.search(&query, 5);
        assert_eq!(r1.len(), r2.len());
    }

    #[test]
    fn test_empty_index() {
        let index = HnswIndex::new(8);
        assert!(index.is_empty());
        let results = index.search(&[0.0; 8], 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_large_insert() {
        let dim = 64;
        let mut index = HnswIndex::new(dim);
        for i in 0..1000 {
            index.insert(i, random_vec(dim));
        }
        assert_eq!(index.len(), 1000);

        let query = random_vec(dim);
        let results = index.search(&query, 10);
        assert_eq!(results.len(), 10);
    }
}
