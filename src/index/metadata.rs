use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;

pub struct MetadataStore {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct ChunkRecord {
    pub id: u32,
    pub doc_path: String,
    pub content: String,
    pub start_line: u32,
    pub end_line: u32,
    pub vector_offset: u32,
}

#[derive(Debug, Clone)]
pub struct FtsResult {
    pub path: String,
    pub title: String,
    pub snippet: String,
    pub rank: f64,
}

impl MetadataStore {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open DB: {}", path.display()))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                title TEXT,
                content TEXT,
                content_hash TEXT,
                last_indexed TEXT
            );
            CREATE TABLE IF NOT EXISTS chunks (
                id INTEGER PRIMARY KEY,
                doc_id INTEGER NOT NULL,
                content TEXT NOT NULL,
                start_line INTEGER,
                end_line INTEGER,
                vector_offset INTEGER,
                FOREIGN KEY (doc_id) REFERENCES documents(id)
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
                title, content, content='documents', content_rowid='id'
            );
            CREATE TRIGGER IF NOT EXISTS documents_ai AFTER INSERT ON documents BEGIN
                INSERT INTO documents_fts(rowid, title, content) VALUES (new.id, new.title, new.content);
            END;
            CREATE TRIGGER IF NOT EXISTS documents_ad AFTER DELETE ON documents BEGIN
                INSERT INTO documents_fts(documents_fts, rowid, title, content) VALUES('delete', old.id, old.title, old.content);
            END;
            CREATE TRIGGER IF NOT EXISTS documents_au AFTER UPDATE ON documents BEGIN
                INSERT INTO documents_fts(documents_fts, rowid, title, content) VALUES('delete', old.id, old.title, old.content);
                INSERT INTO documents_fts(rowid, title, content) VALUES (new.id, new.title, new.content);
            END;",
        )
        .context("Failed to create tables")?;

        Ok(Self { conn })
    }

    pub fn upsert_document(
        &self,
        path: &str,
        title: &str,
        full_content: &str,
        content_hash: &str,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO documents (path, title, content, content_hash, last_indexed)
             VALUES (?1, ?2, ?3, ?4, datetime('now'))
             ON CONFLICT(path) DO UPDATE SET
                title = excluded.title,
                content = excluded.content,
                content_hash = excluded.content_hash,
                last_indexed = excluded.last_indexed",
            params![path, title, full_content, content_hash],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn insert_chunk(
        &self,
        doc_id: i64,
        content: &str,
        start_line: u32,
        end_line: u32,
        vector_offset: u32,
    ) -> Result<u32> {
        self.conn.execute(
            "INSERT INTO chunks (doc_id, content, start_line, end_line, vector_offset)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![doc_id, content, start_line, end_line, vector_offset],
        )?;
        Ok(self.conn.last_insert_rowid() as u32)
    }

    pub fn get_chunk(&self, id: u32) -> Result<Option<ChunkRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, d.path, c.content, c.start_line, c.end_line, c.vector_offset
             FROM chunks c JOIN documents d ON c.doc_id = d.id
             WHERE c.id = ?1",
        )?;

        let result = stmt
            .query_row(params![id], |row| {
                Ok(ChunkRecord {
                    id: row.get(0)?,
                    doc_path: row.get(1)?,
                    content: row.get(2)?,
                    start_line: row.get(3)?,
                    end_line: row.get(4)?,
                    vector_offset: row.get(5)?,
                })
            })
            .ok();

        Ok(result)
    }

    pub fn needs_reindex(&self, path: &str, content_hash: &str) -> Result<bool> {
        let mut stmt = self
            .conn
            .prepare("SELECT content_hash FROM documents WHERE path = ?1")?;

        let existing: Option<String> = stmt.query_row(params![path], |row| row.get(0)).ok();

        match existing {
            Some(hash) => Ok(hash != content_hash),
            None => Ok(true),
        }
    }

    pub fn delete_doc_chunks(&self, path: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM chunks WHERE doc_id IN (SELECT id FROM documents WHERE path = ?1)",
            params![path],
        )?;
        Ok(())
    }

    /// Full-text search using FTS5. Works without AI, instant results.
    pub fn fts_search(&self, query: &str, limit: usize) -> Result<Vec<FtsResult>> {
        let mut stmt = self.conn.prepare(
            "SELECT d.path, d.title, snippet(documents_fts, 1, '>>>', '<<<', '...', 64), rank
             FROM documents_fts
             JOIN documents d ON d.id = documents_fts.rowid
             WHERE documents_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )?;

        let results = stmt
            .query_map(params![query, limit as i64], |row| {
                Ok(FtsResult {
                    path: row.get(0)?,
                    title: row.get(1)?,
                    snippet: row.get(2)?,
                    rank: row.get(3)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    }

    /// Index a single document into FTS (no embeddings needed).
    pub fn index_document_fts(
        &self,
        path: &str,
        title: &str,
        content: &str,
        content_hash: &str,
    ) -> Result<()> {
        if !self.needs_reindex(path, content_hash)? {
            return Ok(());
        }
        self.upsert_document(path, title, content, content_hash)?;
        Ok(())
    }

    /// FTS search filtered by modification date.
    pub fn fts_search_since(
        &self,
        query: &str,
        since: &str,
        limit: usize,
    ) -> Result<Vec<FtsResult>> {
        let mut stmt = self.conn.prepare(
            "SELECT d.path, d.title, snippet(documents_fts, 1, '>>>', '<<<', '...', 64), rank
             FROM documents_fts
             JOIN documents d ON d.id = documents_fts.rowid
             WHERE documents_fts MATCH ?1 AND d.last_indexed >= ?3
             ORDER BY rank
             LIMIT ?2",
        )?;

        let results = stmt
            .query_map(params![query, limit as i64, since], |row| {
                Ok(FtsResult {
                    path: row.get(0)?,
                    title: row.get(1)?,
                    snippet: row.get(2)?,
                    rank: row.get(3)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    }

    pub fn document_count(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))?;
        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_crud() {
        let dir = TempDir::new().unwrap();
        let db = MetadataStore::open(&dir.path().join("test.db")).unwrap();

        let doc_id = db
            .upsert_document("/test.md", "Test", "Hello world content", "abc123")
            .unwrap();
        let chunk_id = db.insert_chunk(doc_id, "Hello world", 1, 5, 0).unwrap();

        let chunk = db.get_chunk(chunk_id).unwrap().unwrap();
        assert_eq!(chunk.content, "Hello world");
        assert_eq!(chunk.doc_path, "/test.md");
    }

    #[test]
    fn test_needs_reindex() {
        let dir = TempDir::new().unwrap();
        let db = MetadataStore::open(&dir.path().join("test.db")).unwrap();

        assert!(db.needs_reindex("/test.md", "hash1").unwrap());
        db.upsert_document("/test.md", "Test", "content", "hash1")
            .unwrap();
        assert!(!db.needs_reindex("/test.md", "hash1").unwrap());
        assert!(db.needs_reindex("/test.md", "hash2").unwrap());
    }

    #[test]
    fn test_fts_search() {
        let dir = TempDir::new().unwrap();
        let db = MetadataStore::open(&dir.path().join("test.db")).unwrap();

        db.upsert_document(
            "/notes/rust.md",
            "Rust Programming",
            "Rust is a systems programming language focused on safety and performance.",
            "h1",
        )
        .unwrap();
        db.upsert_document(
            "/notes/python.md",
            "Python Scripting",
            "Python is a high-level interpreted language used for scripting and web development.",
            "h2",
        )
        .unwrap();
        db.upsert_document(
            "/notes/go.md",
            "Go Language",
            "Go is a compiled language designed for concurrency and simplicity.",
            "h3",
        )
        .unwrap();

        let results = db.fts_search("programming", 10).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].path.contains("rust"));

        let results = db.fts_search("scripting", 10).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].path.contains("python"));
    }

    #[test]
    fn test_fts_no_results() {
        let dir = TempDir::new().unwrap();
        let db = MetadataStore::open(&dir.path().join("test.db")).unwrap();

        db.upsert_document("/a.md", "A", "content a", "h1")
            .unwrap();

        let results = db.fts_search("nonexistent", 10).unwrap();
        assert!(results.is_empty());
    }
}
