use anyhow::Result;
use rand::Rng;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::path::Path;

const M: usize = 16; // max connections per layer
const M_MAX0: usize = 32; // max connections at layer 0
const EF_CONSTRUCTION: usize = 200;
const EF_SEARCH: usize = 50;
fn ml() -> f64 {
    1.0 / (M as f64).ln()
}

#[derive(Clone)]
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
}

#[derive(PartialEq)]
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
        // Min-heap: reverse ordering
        other
            .dist
            .partial_cmp(&self.dist)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        1.0
    } else {
        1.0 - (dot / denom)
    }
}

fn random_level(rng: &mut impl Rng) -> usize {
    let r: f64 = rng.gen();
    (-r.ln() * ml()).floor() as usize
}

impl HnswIndex {
    pub fn new(dim: usize) -> Self {
        Self {
            nodes: HashMap::new(),
            entry_point: None,
            max_layer: 0,
            dim,
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.values().filter(|n| !n.deleted).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn insert(&mut self, id: u32, vector: Vec<f32>) {
        assert_eq!(vector.len(), self.dim);
        let mut rng = rand::thread_rng();
        let level = random_level(&mut rng);

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
                            // Clone what we need to avoid borrow conflict
                            let nv = neighbor.vector.clone();
                            let neighbor_ids: Vec<u32> = neighbor.neighbors[l].clone();
                            let mut scored: Vec<(u32, f32)> = neighbor_ids
                                .iter()
                                .map(|&nid| {
                                    let d = self
                                        .nodes
                                        .get(&nid)
                                        .map(|n2| cosine_distance(&nv, &n2.vector))
                                        .unwrap_or(f32::MAX);
                                    (nid, d)
                                })
                                .collect();
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
            node.deleted = true;
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
            let neighbors = self
                .nodes
                .get(&current)
                .and_then(|n| n.neighbors.get(layer))
                .cloned()
                .unwrap_or_default();

            for &nid in &neighbors {
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

    fn search_layer(&self, start: u32, query: &[f32], ef: usize, layer: usize) -> Vec<Candidate> {
        let start_dist = self
            .nodes
            .get(&start)
            .map(|n| cosine_distance(query, &n.vector))
            .unwrap_or(f32::MAX);

        let mut visited = HashSet::new();
        visited.insert(start);

        let mut candidates = BinaryHeap::new();
        candidates.push(Candidate {
            id: start,
            dist: start_dist,
        });

        let mut results: Vec<Candidate> = vec![Candidate {
            id: start,
            dist: start_dist,
        }];

        while let Some(closest) = candidates.pop() {
            // If closest candidate is farther than the worst result, stop
            if results.len() >= ef {
                let worst = results
                    .iter()
                    .map(|c| c.dist)
                    .fold(f32::NEG_INFINITY, f32::max);
                if closest.dist > worst {
                    break;
                }
            }

            let neighbors = self
                .nodes
                .get(&closest.id)
                .and_then(|n| n.neighbors.get(layer))
                .cloned()
                .unwrap_or_default();

            for &nid in &neighbors {
                if visited.contains(&nid) {
                    continue;
                }
                visited.insert(nid);

                if let Some(neighbor) = self.nodes.get(&nid) {
                    let d = cosine_distance(query, &neighbor.vector);
                    let worst_result = results
                        .iter()
                        .map(|c| c.dist)
                        .fold(f32::NEG_INFINITY, f32::max);

                    if results.len() < ef || d < worst_result {
                        candidates.push(Candidate { id: nid, dist: d });
                        results.push(Candidate { id: nid, dist: d });
                        results.sort_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap());
                        if results.len() > ef {
                            results.truncate(ef);
                        }
                    }
                }
            }
        }

        results.sort_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap());
        results
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
        let mut rng = rand::thread_rng();
        (0..dim).map(|_| rng.gen::<f32>() - 0.5).collect()
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
        let results = index.search(&vec![0.0; 8], 5);
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
