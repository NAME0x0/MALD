use anyhow::Result;

use super::history::ChatSession;
use super::ollama::{ChatMessage, OllamaClient};
use crate::config::ConfigManager;
use crate::fs::mald_home;
use crate::index::hnsw::HnswIndex;
use crate::index::metadata::MetadataStore;

/// A source chunk with citation metadata.
#[derive(Clone)]
pub struct CitedSource {
    pub path: String,
    pub content: String,
    pub start_line: u32,
    pub end_line: u32,
}

/// Retrieve relevant sources for a query, returning chunks with citation info.
pub async fn retrieve_sources(
    client: &OllamaClient,
    config: &ConfigManager,
    query: &str,
    k: usize,
) -> Result<Vec<CitedSource>> {
    let index_dir = mald_home().join("index");
    let hnsw_path = index_dir.join("hnsw.bin");
    let meta_path = index_dir.join("metadata.db");

    let mut sources = Vec::new();

    // Try vector search first
    if hnsw_path.exists() && meta_path.exists() {
        let embedding_model = config
            .get_string("ai.embedding_model")
            .unwrap_or_else(|| "nomic-embed-text".into());

        let meta = MetadataStore::open(&meta_path)?;
        let index = HnswIndex::load(&hnsw_path)?;
        let query_vec = client.embeddings(&embedding_model, query).await?;
        let results = index.search(&query_vec, k);

        for (id, _score) in &results {
            if let Some(chunk) = meta.get_chunk(*id)? {
                sources.push(CitedSource {
                    path: chunk.doc_path,
                    content: chunk.content,
                    start_line: chunk.start_line,
                    end_line: chunk.end_line,
                });
            }
        }
    }

    // Fallback to FTS if no vector results
    if sources.is_empty() {
        let meta_path = index_dir.join("metadata.db");
        if meta_path.exists() {
            let meta = MetadataStore::open(&meta_path)?;
            let fts_results = meta.fts_search(query, k)?;
            for r in fts_results {
                sources.push(CitedSource {
                    path: r.path,
                    content: r.snippet.clone(),
                    start_line: 0,
                    end_line: 0,
                });
            }
        }
    }

    Ok(sources)
}

/// Build a cited context block from sources, numbering each source.
fn build_cited_context(sources: &[CitedSource]) -> String {
    if sources.is_empty() {
        return String::new();
    }
    let mut ctx = String::from("Sources:\n\n");
    for (i, src) in sources.iter().enumerate() {
        let loc = if src.start_line > 0 {
            format!("{}:L{}-{}", src.path, src.start_line, src.end_line)
        } else {
            src.path.clone()
        };
        ctx.push_str(&format!("[{}] {}\n{}\n\n", i + 1, loc, src.content));
    }
    ctx
}

/// Format citation references for display.
pub fn format_citations(sources: &[CitedSource]) -> String {
    if sources.is_empty() {
        return String::new();
    }
    let mut out = String::from("\n--- Sources ---\n");
    for (i, src) in sources.iter().enumerate() {
        let loc = if src.start_line > 0 {
            format!("{}:L{}-{}", src.path, src.start_line, src.end_line)
        } else {
            src.path.clone()
        };
        out.push_str(&format!("[{}] {}\n", i + 1, loc));
    }
    out
}

/// Chat with RAG, citations, and conversation history.
pub async fn chat_with_rag(
    client: &OllamaClient,
    config: &ConfigManager,
    query: &str,
    kb_name: &str,
    session: Option<&mut ChatSession>,
) -> Result<(String, Vec<CitedSource>)> {
    let model = config
        .get_string("ai.default_model")
        .unwrap_or_else(|| "llama3.2".into());

    let sources = retrieve_sources(client, config, query, 5).await?;
    let context = build_cited_context(&sources);

    let system_prompt = if context.is_empty() {
        format!(
            "You are a helpful assistant for the knowledge base '{kb_name}'. \
             Answer questions directly and concisely."
        )
    } else {
        format!(
            "You are a helpful assistant for the knowledge base '{kb_name}'. \
             Answer using ONLY the following sources. Cite sources using [1], [2], etc. \
             If the sources don't contain the answer, say so.\n\n{context}"
        )
    };

    let mut messages = vec![ChatMessage {
        role: "system".into(),
        content: system_prompt,
    }];

    // Include conversation history if available
    if let Some(sess) = &session {
        // Include last 10 messages for context window management
        let history_start = sess.messages.len().saturating_sub(10);
        for msg in &sess.messages[history_start..] {
            messages.push(msg.clone());
        }
    }

    messages.push(ChatMessage {
        role: "user".into(),
        content: query.to_string(),
    });

    let response = client.chat_with_history(&model, &messages).await?;

    Ok((response, sources))
}

/// Summarize one or more notes.
pub async fn summarize(
    client: &OllamaClient,
    config: &ConfigManager,
    note_contents: &[(String, String)], // (title, content) pairs
) -> Result<String> {
    let model = config
        .get_string("ai.default_model")
        .unwrap_or_else(|| "llama3.2".into());

    let mut context = String::new();
    for (i, (title, content)) in note_contents.iter().enumerate() {
        context.push_str(&format!(
            "--- Document {} ---\nTitle: {}\n{}\n\n",
            i + 1,
            title,
            content
        ));
    }

    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: "Summarize the following documents concisely. Include key points, \
                     main arguments, and important details. Use bullet points for clarity."
                .into(),
        },
        ChatMessage {
            role: "user".into(),
            content: context,
        },
    ];

    client.chat_with_history(&model, &messages).await
}

/// Generate a quiz from note contents.
pub async fn quiz(
    client: &OllamaClient,
    config: &ConfigManager,
    note_contents: &[(String, String)],
    count: usize,
) -> Result<String> {
    let model = config
        .get_string("ai.default_model")
        .unwrap_or_else(|| "llama3.2".into());

    let mut context = String::new();
    for (title, content) in note_contents {
        context.push_str(&format!("--- {title} ---\n{content}\n\n"));
    }

    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: format!(
                "Generate exactly {count} quiz questions based on the following documents. \
                 For each question:\n\
                 1. Write the question\n\
                 2. Provide 4 multiple choice options (A-D)\n\
                 3. Mark the correct answer\n\
                 4. Give a brief explanation\n\
                 Format clearly with numbered questions."
            ),
        },
        ChatMessage {
            role: "user".into(),
            content: context,
        },
    ];

    client.chat_with_history(&model, &messages).await
}

/// Generate a briefing/digest from recent notes.
pub async fn briefing(
    client: &OllamaClient,
    config: &ConfigManager,
    note_contents: &[(String, String)],
) -> Result<String> {
    let model = config
        .get_string("ai.default_model")
        .unwrap_or_else(|| "llama3.2".into());

    let mut context = String::new();
    for (title, content) in note_contents {
        context.push_str(&format!("--- {title} ---\n{content}\n\n"));
    }

    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: "Generate a daily briefing from these notes. Structure it as:\n\
                     1. **Key Updates** — what's new or changed\n\
                     2. **Action Items** — tasks and deadlines extracted from notes\n\
                     3. **Connections** — themes or links between different notes\n\
                     4. **Gaps** — what's missing or needs follow-up\n\
                     Be concise and actionable."
                .into(),
        },
        ChatMessage {
            role: "user".into(),
            content: context,
        },
    ];

    client.chat_with_history(&model, &messages).await
}

/// Compare multiple notes and find commonalities/differences.
pub async fn compare(
    client: &OllamaClient,
    config: &ConfigManager,
    note_contents: &[(String, String)],
) -> Result<String> {
    let model = config
        .get_string("ai.default_model")
        .unwrap_or_else(|| "llama3.2".into());

    let mut context = String::new();
    for (i, (title, content)) in note_contents.iter().enumerate() {
        context.push_str(&format!(
            "--- Document {} ---\nTitle: {}\n{}\n\n",
            i + 1,
            title,
            content
        ));
    }

    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: "Compare and contrast these documents. Identify:\n\
                     1. **Common Themes** — shared ideas across documents\n\
                     2. **Contradictions** — conflicting information\n\
                     3. **Unique Points** — ideas only in one document\n\
                     4. **Synthesis** — what conclusions can be drawn from all documents together\n\
                     Reference documents by their titles."
                .into(),
        },
        ChatMessage {
            role: "user".into(),
            content: context,
        },
    ];

    client.chat_with_history(&model, &messages).await
}

/// Extract a timeline from notes.
pub async fn timeline(
    client: &OllamaClient,
    config: &ConfigManager,
    note_contents: &[(String, String)],
) -> Result<String> {
    let model = config
        .get_string("ai.default_model")
        .unwrap_or_else(|| "llama3.2".into());

    let mut context = String::new();
    for (title, content) in note_contents {
        context.push_str(&format!("--- {title} ---\n{content}\n\n"));
    }

    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: "Extract a chronological timeline from these documents. \
                     List events/milestones in order with dates (if available) or relative ordering. \
                     Format: `YYYY-MM-DD | Event description (source: document title)`\n\
                     If no dates are found, order by logical sequence."
                .into(),
        },
        ChatMessage {
            role: "user".into(),
            content: context,
        },
    ];

    client.chat_with_history(&model, &messages).await
}

/// Explain a note in simpler terms.
pub async fn explain(
    client: &OllamaClient,
    config: &ConfigManager,
    title: &str,
    content: &str,
) -> Result<String> {
    let model = config
        .get_string("ai.default_model")
        .unwrap_or_else(|| "llama3.2".into());

    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: "Explain the following document in plain, simple language. \
                     Assume the reader has no background knowledge. \
                     Use analogies where helpful. Break complex ideas into small steps."
                .into(),
        },
        ChatMessage {
            role: "user".into(),
            content: format!("Title: {title}\n\n{content}"),
        },
    ];

    client.chat_with_history(&model, &messages).await
}
