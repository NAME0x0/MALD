use super::MarkdownDocument;
use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug)]
pub struct GraphNode {
    pub name: String,
    pub path: std::path::PathBuf,
    pub links: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<(String, String)>,
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

        for link in doc.all_links() {
            edges.push((name.clone(), link.to_string()));
        }

        nodes.push(GraphNode {
            name,
            path: doc.path.clone().unwrap_or_default(),
            links: doc.all_links().into_iter().map(String::from).collect(),
            tags: doc.tags.clone(),
        });
    }

    GraphData { nodes, edges }
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
