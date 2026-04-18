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
/// Uses iterator-based processing to minimize allocations.
pub fn chunk_document(content: &str) -> Vec<Chunk> {
    if content.is_empty() {
        return Vec::new();
    }

    // Collect line boundaries lazily - we need indices for slicing
    let line_offsets: Vec<usize> = std::iter::once(0)
        .chain(content.match_indices('\n').map(|(i, _)| i + 1))
        .collect();
    let line_count = line_offsets.len();

    if line_count == 0 {
        return vec![Chunk {
            content: content.to_string(),
            start_line: 1,
            end_line: 1,
        }];
    }

    // Find heading boundaries (H2 or H3) using byte offsets
    let mut boundaries: Vec<usize> = vec![0];
    for (line_idx, &offset) in line_offsets.iter().enumerate() {
        if line_idx == 0 {
            continue;
        }
        let line_end = line_offsets
            .get(line_idx + 1)
            .map(|&o| o - 1)
            .unwrap_or(content.len());
        let line = &content[offset..line_end];
        let trimmed = line.trim_start();
        if trimmed.starts_with("## ") || trimmed.starts_with("### ") {
            boundaries.push(line_idx);
        }
    }
    boundaries.push(line_count);

    let mut chunks = Vec::new();

    for window in boundaries.windows(2) {
        let start_line = window[0];
        let end_line = window[1];

        // Get byte range for this section
        let start_byte = line_offsets[start_line];
        let end_byte = line_offsets.get(end_line).copied().unwrap_or(content.len());
        let section = &content[start_byte..end_byte];

        if section.len() <= MAX_CHUNK_SIZE {
            chunks.push(Chunk {
                content: section.to_string(),
                start_line: start_line as u32 + 1,
                end_line: end_line as u32,
            });
        } else {
            // Split large sections by character limit with fixed line overlap
            let mut line_idx = start_line;
            while line_idx < end_line {
                let chunk_start_line = line_idx;
                let chunk_start_byte = line_offsets[line_idx];
                let mut chunk_end_byte = chunk_start_byte;

                while line_idx < end_line {
                    let next_byte = line_offsets
                        .get(line_idx + 1)
                        .copied()
                        .unwrap_or(content.len());
                    if next_byte - chunk_start_byte > MAX_CHUNK_SIZE && line_idx > chunk_start_line
                    {
                        break;
                    }
                    chunk_end_byte = next_byte;
                    line_idx += 1;
                }

                chunks.push(Chunk {
                    content: content[chunk_start_byte..chunk_end_byte].to_string(),
                    start_line: chunk_start_line as u32 + 1,
                    end_line: line_idx as u32,
                });

                // Overlap: back up by a fixed number of lines.
                // Ensure we always advance at least 1 line to prevent infinite loop.
                if line_idx < end_line {
                    let advanced = line_idx - chunk_start_line;
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
