use super::MarkdownDocument;
use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub name: String,
    pub path: std::path::PathBuf,
    pub links: Vec<String>,
    pub tags: Vec<String>,
}

/// Graph data with Arc-wrapped collections for cheap cloning.
/// Cloning GraphData only increments reference counts, not deep copies.
#[derive(Debug, Clone)]
pub struct GraphData {
    pub nodes: Arc<Vec<GraphNode>>,
    pub edges: Arc<Vec<(String, String)>>,
}

impl GraphData {
    /// Create new GraphData with owned vectors (wraps in Arc).
    pub fn new(nodes: Vec<GraphNode>, edges: Vec<(String, String)>) -> Self {
        Self {
            nodes: Arc::new(nodes),
            edges: Arc::new(edges),
        }
    }

    /// Check if graph is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get node count.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get edge count.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

/// Parse all markdown files in a knowledge base directory.
pub fn parse_knowledge_base(kb_path: &Path) -> Result<Vec<MarkdownDocument>> {
    let files = crate::fs::find_files(kb_path, "md")?;
    let mut docs = Vec::new();
    for file in files {
        match MarkdownDocument::from_file(&file) {
            Ok(doc) => docs.push(doc),
            Err(e) => tracing::warn!("Failed to parse {}: {}", file.display(), e),
        }
    }
    Ok(docs)
}

/// Find files that have no incoming links from other files.
pub fn find_orphaned_files(docs: &[MarkdownDocument]) -> Vec<&MarkdownDocument> {
    let mut linked_targets: HashSet<String> = HashSet::new();
    for doc in docs {
        for link in doc.all_links() {
            linked_targets.insert(link.to_lowercase());
        }
    }

    docs.iter()
        .filter(|doc| {
            let name = doc
                .path
                .as_ref()
                .and_then(|p| p.file_stem())
                .map(|s| s.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            !linked_targets.contains(&name) && name != "index"
        })
        .collect()
}

/// Generate graph data from parsed documents.
/// Returns Arc-wrapped data for cheap cloning across panes/views.
pub fn generate_graph_data(docs: &[MarkdownDocument]) -> GraphData {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for doc in docs {
        let name = doc
            .path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| doc.title.clone());

        // Extract links once — avoids duplicate Vec allocation per document
        let links = doc.all_links();

        for link in &links {
            edges.push((name.clone(), link.to_string()));
        }

        nodes.push(GraphNode {
            name,
            path: doc.path.clone().unwrap_or_default(),
            links: links.into_iter().map(String::from).collect(),
            tags: doc.tags.clone(),
        });
    }

    GraphData::new(nodes, edges)
}

/// Find all documents that link to a given target.
pub fn find_backlinks<'a>(docs: &'a [MarkdownDocument], target: &str) -> Vec<&'a MarkdownDocument> {
    let target_lower = target.to_lowercase();
    docs.iter()
        .filter(|doc| {
            doc.all_links()
                .iter()
                .any(|l| l.to_lowercase() == target_lower)
        })
        .collect()
}

/// Simple content search across documents.
pub fn search_content<'a>(docs: &'a [MarkdownDocument], query: &str) -> Vec<&'a MarkdownDocument> {
    let query_lower = query.to_lowercase();
    docs.iter()
        .filter(|doc| {
            doc.title.to_lowercase().contains(&query_lower)
                || doc.content.to_lowercase().contains(&query_lower)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper: create a markdown file in the given directory.
    fn create_md_file(dir: &std::path::Path, name: &str, content: &str) {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    /// Helper: build a MarkdownDocument with a given path, title, content, and wikilinks.
    fn make_doc(name: &str, content: &str) -> MarkdownDocument {
        let mut doc = MarkdownDocument::parse(content);
        doc.path = Some(std::path::PathBuf::from(format!("/kb/{}.md", name)));
        if doc.title.is_empty() {
            doc.title = name.to_string();
        }
        doc
    }

    // ---- parse_knowledge_base tests ----

    #[test]
    fn test_parse_knowledge_base_with_files() {
        let dir = TempDir::new().unwrap();
        create_md_file(dir.path(), "note1.md", "# Note One\nHello world\n");
        create_md_file(dir.path(), "note2.md", "# Note Two\nFoo bar\n");

        let docs = parse_knowledge_base(dir.path()).unwrap();
        assert_eq!(docs.len(), 2);
        let titles: Vec<&str> = docs.iter().map(|d| d.title.as_str()).collect();
        assert!(titles.contains(&"Note One"));
        assert!(titles.contains(&"Note Two"));
    }

    #[test]
    fn test_parse_knowledge_base_empty_dir() {
        let dir = TempDir::new().unwrap();
        let docs = parse_knowledge_base(dir.path()).unwrap();
        assert!(docs.is_empty());
    }

    // ---- find_orphaned_files tests ----

    #[test]
    fn test_find_orphaned_files_has_orphans() {
        let docs = vec![
            make_doc("alpha", "# Alpha\nLinks to [[beta]]\n"),
            make_doc("beta", "# Beta\nNo outgoing links\n"),
            make_doc("gamma", "# Gamma\nAlone in the world\n"),
        ];

        let orphans = find_orphaned_files(&docs);
        let orphan_titles: Vec<&str> = orphans.iter().map(|d| d.title.as_str()).collect();
        // alpha is not linked to by anyone, so it's orphaned
        assert!(orphan_titles.contains(&"Alpha"));
        // gamma is not linked to by anyone, so it's orphaned
        assert!(orphan_titles.contains(&"Gamma"));
        // beta IS linked to by alpha, so it's NOT orphaned
        assert!(!orphan_titles.contains(&"Beta"));
    }

    #[test]
    fn test_find_orphaned_files_none_orphaned() {
        let docs = vec![
            make_doc("alpha", "# Alpha\nLinks to [[beta]]\n"),
            make_doc("beta", "# Beta\nLinks to [[alpha]]\n"),
        ];

        let orphans = find_orphaned_files(&docs);
        assert!(orphans.is_empty());
    }

    // ---- find_backlinks tests ----

    #[test]
    fn test_find_backlinks_has_backlinks() {
        let docs = vec![
            make_doc("alpha", "# Alpha\nLinks to [[beta]]\n"),
            make_doc("beta", "# Beta\nNo links\n"),
            make_doc("gamma", "# Gamma\nAlso links to [[beta]]\n"),
        ];

        let backlinks = find_backlinks(&docs, "beta");
        assert_eq!(backlinks.len(), 2);
        let titles: Vec<&str> = backlinks.iter().map(|d| d.title.as_str()).collect();
        assert!(titles.contains(&"Alpha"));
        assert!(titles.contains(&"Gamma"));
    }

    #[test]
    fn test_find_backlinks_no_backlinks() {
        let docs = vec![
            make_doc("alpha", "# Alpha\nLinks to [[beta]]\n"),
            make_doc("beta", "# Beta\nNo links\n"),
        ];

        let backlinks = find_backlinks(&docs, "alpha");
        assert!(backlinks.is_empty());
    }

    // ---- search_content tests ----

    #[test]
    fn test_search_content_matches_title() {
        let docs = vec![
            make_doc("rust", "# Rust Programming\nSome content\n"),
            make_doc("python", "# Python Guide\nOther content\n"),
        ];

        let results = search_content(&docs, "Rust");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Programming");
    }

    #[test]
    fn test_search_content_matches_body() {
        let docs = vec![
            make_doc("note", "# My Note\nThe quick brown fox jumps\n"),
            make_doc("other", "# Other\nNothing relevant here\n"),
        ];

        let results = search_content(&docs, "quick brown");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "My Note");
    }

    #[test]
    fn test_search_content_no_match() {
        let docs = vec![make_doc("note", "# My Note\nContent here\n")];

        let results = search_content(&docs, "nonexistent_query_xyz");
        assert!(results.is_empty());
    }

    // ---- generate_graph_data tests ----

    #[test]
    fn test_generate_graph_data_nodes_and_edges() {
        let docs = vec![
            make_doc("alpha", "# Alpha\nLinks to [[beta]] and [[gamma]]\n"),
            make_doc("beta", "# Beta\nLinks to [[alpha]]\n"),
            make_doc("gamma", "# Gamma\nNo links\n"),
        ];

        let graph = generate_graph_data(&docs);
        assert_eq!(graph.node_count(), 3);
        // alpha -> beta, alpha -> gamma, beta -> alpha = 3 edges
        assert_eq!(graph.edge_count(), 3);

        let node_names: Vec<&str> = graph.nodes.iter().map(|n| n.name.as_str()).collect();
        assert!(node_names.contains(&"alpha"));
        assert!(node_names.contains(&"beta"));
        assert!(node_names.contains(&"gamma"));
    }

    #[test]
    fn test_generate_graph_data_empty() {
        let docs: Vec<MarkdownDocument> = vec![];
        let graph = generate_graph_data(&docs);
        assert!(graph.is_empty());
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }
}
