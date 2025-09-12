"""
RAG Engine - Main interface for retrieval augmented generation
"""

import logging
from pathlib import Path
from typing import List, Dict, Optional
from ..utils.markdown_parser import MarkdownDocument, parse_knowledge_base
from .embeddings import EmbeddingEngine
from .vector_store import VectorStore

logger = logging.getLogger(__name__)


class RAGEngine:
    """Main RAG engine that combines retrieval and generation"""
    
    def __init__(self, kb_path: Path, index_path: Path):
        self.kb_path = Path(kb_path)
        self.index_path = Path(index_path)
        
        # Initialize components
        self.embedding_engine = EmbeddingEngine()
        self.vector_store = VectorStore(
            index_path=index_path,
            dimension=self.embedding_engine.get_dimension()
        )
        
        # Track indexed documents
        self.indexed_files = {}
        self._load_indexed_files()
    
    def index_knowledge_base(self, force_reindex: bool = False) -> int:
        """Index all documents in the knowledge base"""
        logger.info(f"Indexing knowledge base: {self.kb_path}")
        
        if force_reindex:
            self.vector_store.clear()
            self.indexed_files = {}
        
        # Parse all markdown documents
        documents = parse_knowledge_base(self.kb_path)
        new_documents = []
        texts_to_embed = []
        
        for rel_path, doc in documents.items():
            file_path = doc.path
            file_mtime = file_path.stat().st_mtime if file_path.exists() else 0
            
            # Check if document needs indexing
            if (rel_path not in self.indexed_files or 
                self.indexed_files[rel_path] < file_mtime):
                
                # Prepare document for indexing
                doc_metadata = {
                    'path': str(file_path),
                    'relative_path': rel_path,
                    'title': doc.title,
                    'tags': doc.tags,
                    'links': doc.links,
                    'modified': file_mtime
                }
                
                # Create text for embedding (title + content)
                embed_text = f"{doc.title}\n\n{doc.content}"
                
                new_documents.append(doc_metadata)
                texts_to_embed.append(embed_text)
                self.indexed_files[rel_path] = file_mtime
        
        if new_documents:
            logger.info(f"Indexing {len(new_documents)} new/updated documents")
            
            # Generate embeddings
            embeddings = self.embedding_engine.embed(texts_to_embed)
            
            # Add to vector store
            self.vector_store.add_documents(new_documents, embeddings)
            
            # Save everything
            self.vector_store.save_index()
            self._save_indexed_files()
        else:
            logger.info("No new documents to index")
        
        return len(new_documents)
    
    def search(self, query: str, k: int = 5, min_score: float = 0.1) -> List[Dict]:
        """Search for relevant documents"""
        if not query.strip():
            return []
        
        # Generate query embedding
        query_embedding = self.embedding_engine.embed([query])
        
        # Search vector store
        results = self.vector_store.search(query_embedding[0], k=k)
        
        # Filter by minimum score and format results
        filtered_results = []
        for doc_metadata, score in results:
            if score >= min_score:
                result = {
                    'document': doc_metadata,
                    'score': score,
                    'snippet': self._get_snippet(doc_metadata['path'], query)
                }
                filtered_results.append(result)
        
        return filtered_results
    
    def _get_snippet(self, file_path: str, query: str, context_chars: int = 200) -> str:
        """Get a relevant snippet from the document"""
        try:
            content = Path(file_path).read_text(encoding='utf-8')
            
            # Find query in content (case insensitive)
            query_lower = query.lower()
            content_lower = content.lower()
            
            # Find the best match position
            best_pos = content_lower.find(query_lower)
            if best_pos == -1:
                # If exact match not found, return beginning
                best_pos = 0
            
            # Extract context around the match
            start = max(0, best_pos - context_chars // 2)
            end = min(len(content), best_pos + len(query) + context_chars // 2)
            
            snippet = content[start:end].strip()
            
            # Add ellipsis if truncated
            if start > 0:
                snippet = "..." + snippet
            if end < len(content):
                snippet = snippet + "..."
            
            return snippet
            
        except Exception as e:
            logger.warning(f"Failed to get snippet from {file_path}: {e}")
            return ""
    
    def generate_response(self, query: str, context_docs: List[Dict] = None, 
                         model: str = "llama3.2:1b") -> str:
        """Generate a response using retrieved context"""
        if context_docs is None:
            context_docs = self.search(query, k=3)
        
        # Build context from retrieved documents
        context_text = ""
        for doc in context_docs:
            doc_info = doc['document']
            snippet = doc['snippet']
            context_text += f"Document: {doc_info['title']}\n{snippet}\n\n"
        
        # Build prompt
        prompt = self._build_rag_prompt(query, context_text)
        
        # Generate response using available AI backend
        return self._generate_with_llm(prompt, model)
    
    def _build_rag_prompt(self, query: str, context: str) -> str:
        """Build RAG prompt with context"""
        prompt = f"""You are a helpful assistant with access to a personal knowledge base. 
Use the provided context to answer the user's question. If the context doesn't contain 
relevant information, say so clearly.

Context from knowledge base:
{context}

User question: {query}

Answer based on the context provided:"""
        
        return prompt
    
    def _generate_with_llm(self, prompt: str, model: str) -> str:
        """Generate response using available LLM"""
        try:
            # Try Ollama first
            response = self._generate_with_ollama(prompt, model)
            if response:
                return response
        except Exception as e:
            logger.warning(f"Ollama generation failed: {e}")
        
        # Fallback response
        return ("I can help you search your knowledge base, but no AI model is currently "
                "available for generating responses. The retrieved context might contain "
                "the information you're looking for.")
    
    def _generate_with_ollama(self, prompt: str, model: str) -> Optional[str]:
        """Generate response using Ollama"""
        try:
            import requests
            
            response = requests.post(
                "http://localhost:11434/api/generate",
                json={
                    "model": model,
                    "prompt": prompt,
                    "stream": False
                },
                timeout=60
            )
            
            if response.status_code == 200:
                result = response.json()
                return result.get("response", "")
            else:
                logger.warning(f"Ollama request failed: {response.status_code}")
                return None
                
        except Exception as e:
            logger.warning(f"Ollama generation error: {e}")
            return None
    
    def _load_indexed_files(self):
        """Load the record of indexed files"""
        try:
            index_file = self.index_path / "indexed_files.json"
            if index_file.exists():
                import json
                with open(index_file) as f:
                    self.indexed_files = json.load(f)
        except Exception as e:
            logger.warning(f"Failed to load indexed files record: {e}")
            self.indexed_files = {}
    
    def _save_indexed_files(self):
        """Save the record of indexed files"""
        try:
            import json
            index_file = self.index_path / "indexed_files.json"
            with open(index_file, 'w') as f:
                json.dump(self.indexed_files, f, indent=2)
        except Exception as e:
            logger.warning(f"Failed to save indexed files record: {e}")
    
    def get_stats(self) -> Dict:
        """Get RAG engine statistics"""
        return {
            'kb_path': str(self.kb_path),
            'indexed_files': len(self.indexed_files),
            'vector_store': self.vector_store.get_stats(),
            'embedding_model': self.embedding_engine.model_name,
            'embedding_backend': self.embedding_engine.backend
        }