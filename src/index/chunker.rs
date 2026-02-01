/// A chunk of document text with line range.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub content: String,
    pub start_line: u32,
    pub end_line: u32,
}

const MAX_CHUNK_SIZE: usize = 2000;
const OVERLAP_LINES: usize = 3;

/// Split document content into chunks by H2/H3 headings, with fallback to character limits.
pub fn chunk_document(content: &str) -> Vec<Chunk> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Vec::new();
    }

    // Find heading boundaries (H2 or H3)
    let mut boundaries: Vec<usize> = vec![0];
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if (trimmed.starts_with("## ") || trimmed.starts_with("### ")) && i > 0 {
            boundaries.push(i);
        }
    }
    boundaries.push(lines.len());

    let mut chunks = Vec::new();

    for window in boundaries.windows(2) {
        let start = window[0];
        let end = window[1];
        let section: String = lines[start..end].join("\n");

        if section.len() <= MAX_CHUNK_SIZE {
            chunks.push(Chunk {
                content: section,
                start_line: start as u32 + 1,
                end_line: end as u32,
            });
        } else {
            // Split large sections by character limit with fixed line overlap
            let mut line_idx = start;
            while line_idx < end {
                let mut chunk_text = String::new();
                let chunk_start = line_idx;

                while line_idx < end && chunk_text.len() < MAX_CHUNK_SIZE {
                    if !chunk_text.is_empty() {
                        chunk_text.push('\n');
                    }
                    chunk_text.push_str(lines[line_idx]);
                    line_idx += 1;
                }

                chunks.push(Chunk {
                    content: chunk_text,
                    start_line: chunk_start as u32 + 1,
                    end_line: line_idx as u32,
                });

                // Overlap: back up by a fixed number of lines.
                // Ensure we always advance at least 1 line to prevent infinite loop.
                if line_idx < end {
                    let advanced = line_idx - chunk_start;
                    let back = OVERLAP_LINES.min(advanced.saturating_sub(1));
                    line_idx -= back;
                }
            }
        }
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_by_headings() {
        let content =
            "# Title\n\nIntro text.\n\n## Section A\n\nContent A.\n\n## Section B\n\nContent B.\n";
        let chunks = chunk_document(content);
        assert!(chunks.len() >= 2);
    }

    #[test]
    fn test_empty_document() {
        let chunks = chunk_document("");
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_small_document_single_chunk() {
        let content = "# Small\n\nJust a small doc.\n";
        let chunks = chunk_document(content);
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn test_large_section_splits() {
        let line = "x".repeat(100);
        let mut content = String::from("# Title\n\n");
        for _ in 0..50 {
            content.push_str(&line);
            content.push('\n');
        }
        let chunks = chunk_document(&content);
        assert!(chunks.len() > 1);
    }

    #[test]
    fn test_overlap_never_infinite_loops() {
        // Regression: previous overlap logic could cause infinite loop
        // with very short lines where overlap > advance
        let mut content = String::from("# Title\n");
        for i in 0..200 {
            content.push_str(&format!("line {i}\n"));
        }
        let chunks = chunk_document(&content);
        assert!(!chunks.is_empty());
        // Verify the last chunk reaches the end
        let last = chunks.last().unwrap();
        assert!(last.end_line >= 200);
    }
}
