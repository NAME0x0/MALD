"""
Vector store for storing and retrieving document embeddings
"""

import json
import logging
from pathlib import Path
from typing import List, Dict, Tuple, Union

# Try to import numpy, fallback to built-in list if not available
try:
    import numpy as np
    HAS_NUMPY = True
except ImportError:
    HAS_NUMPY = False
    np = None

logger = logging.getLogger(__name__)


class VectorStore:
    """Simple vector store using numpy fallback"""
    
    def __init__(self, index_path: Path, dimension: int = 384):
        self.index_path = Path(index_path)
        self.dimension = dimension
        self.documents = {}  # Maps vector IDs to document metadata
        self.next_id = 0
        
        if HAS_NUMPY:
            self.vectors = np.empty((0, self.dimension), dtype=np.float32)
        else:
            self.vectors = []  # List of lists fallback
        
        self._load_index()
        logger.info("Using fallback vector storage (no numpy)")
    
    def add_documents(self, documents: List[Dict], embeddings: Union[List[List[float]], 'np.ndarray']):
        """Add documents with their embeddings to the store"""
        if HAS_NUMPY and hasattr(embeddings, 'shape'):
            if embeddings.shape[1] != self.dimension:
                raise ValueError(f"Embedding dimension {embeddings.shape[1]} doesn't match expected {self.dimension}")
        elif isinstance(embeddings, list) and embeddings:
            if len(embeddings[0]) != self.dimension:
                raise ValueError(f"Embedding dimension {len(embeddings[0])} doesn't match expected {self.dimension}")
        
        num_docs = len(documents)
        start_id = self.next_id
        
        # Normalize embeddings for cosine similarity
        normalized_embeddings = self._normalize_vectors(embeddings)
        
        # Add to vector store
        if HAS_NUMPY:
            if isinstance(normalized_embeddings, list):
                normalized_embeddings = np.array(normalized_embeddings, dtype=np.float32)
            self.vectors = np.vstack([self.vectors, normalized_embeddings.astype(np.float32)])
        else:
            if not isinstance(normalized_embeddings, list):
                normalized_embeddings = normalized_embeddings.tolist()
            self.vectors.extend(normalized_embeddings)
        
        # Store document metadata
        for i, doc in enumerate(documents):
            doc_id = start_id + i
            self.documents[doc_id] = doc
        
        self.next_id += num_docs
        logger.info(f"Added {num_docs} documents to vector store")
    
    def search(self, query_embedding: Union[List[float], 'np.ndarray'], k: int = 5) -> List[Tuple[Dict, float]]:
        """Search for similar documents"""
        if len(self.documents) == 0:
            return []
        
        # Convert to normalized form
        if isinstance(query_embedding, list):
            query_embedding = [query_embedding]
        elif HAS_NUMPY and hasattr(query_embedding, 'reshape'):
            query_embedding = query_embedding.reshape(1, -1)
        else:
            query_embedding = [query_embedding]
            
        normalized_query = self._normalize_vectors(query_embedding)
        
        if HAS_NUMPY and isinstance(self.vectors, np.ndarray):
            # Numpy computation
            if isinstance(normalized_query, list):
                normalized_query = np.array(normalized_query)
            similarities = np.dot(self.vectors, normalized_query.T).flatten()
            
            # Get top k indices
            top_indices = np.argsort(similarities)[-k:][::-1]
            
            results = []
            for idx in top_indices:
                if idx in self.documents:
                    results.append((self.documents[idx], float(similarities[idx])))
        else:
            # Pure Python fallback
            if not isinstance(normalized_query, list):
                if HAS_NUMPY and hasattr(normalized_query, 'tolist'):
                    normalized_query = normalized_query.tolist()
                else:
                    normalized_query = list(normalized_query)
            query_vec = normalized_query[0]
            
            similarities = []
            for i, doc_vec in enumerate(self.vectors):
                similarity = sum(a * b for a, b in zip(doc_vec, query_vec))
                similarities.append((i, similarity))
            
            # Sort by similarity and get top k
            similarities.sort(key=lambda x: x[1], reverse=True)
            
            results = []
            for i, (idx, sim) in enumerate(similarities[:k]):
                if idx in self.documents:
                    results.append((self.documents[idx], sim))
        
        return results
    
    def _normalize_vectors(self, vectors: Union[List[List[float]], 'np.ndarray']) -> Union[List[List[float]], 'np.ndarray']:
        """Normalize vectors for cosine similarity"""
        if HAS_NUMPY and hasattr(vectors, 'shape'):
            # Numpy computation
            norms = np.linalg.norm(vectors, axis=1, keepdims=True)
            norms = np.where(norms == 0, 1, norms)  # Avoid division by zero
            return vectors / norms
        else:
            # Pure Python fallback
            if HAS_NUMPY and hasattr(vectors, 'tolist'):
                vectors = vectors.tolist()
            elif not isinstance(vectors, list):
                vectors = list(vectors)
            
            normalized = []
            for vec in vectors:
                norm = sum(x * x for x in vec) ** 0.5
                if norm == 0:
                    norm = 1
                normalized.append([x / norm for x in vec])
            
            return normalized
    
    def save_index(self):
        """Save the index to disk"""
        try:
            self.index_path.parent.mkdir(parents=True, exist_ok=True)
            
            # Save vectors
            if HAS_NUMPY and isinstance(self.vectors, np.ndarray):
                np.save(self.index_path / "vectors.npy", self.vectors)
            else:
                # Save as JSON for pure Python fallback
                with open(self.index_path / "vectors.json", 'w') as f:
                    json.dump(self.vectors, f)
            
            # Save document metadata
            with open(self.index_path / "documents.json", 'w') as f:
                json.dump(self.documents, f, indent=2)
            
            # Save store metadata
            metadata = {
                'dimension': self.dimension,
                'next_id': self.next_id,
                'num_documents': len(self.documents),
                'has_numpy': HAS_NUMPY
            }
            
            with open(self.index_path / "metadata.json", 'w') as f:
                json.dump(metadata, f, indent=2)
            
            logger.info(f"Saved vector store to {self.index_path}")
            
        except Exception as e:
            logger.error(f"Failed to save vector store: {e}")
    
    def _load_index(self):
        """Load the index from disk"""
        try:
            metadata_file = self.index_path / "metadata.json"
            if not metadata_file.exists():
                return
            
            with open(metadata_file) as f:
                metadata = json.load(f)
            
            self.next_id = metadata.get('next_id', 0)
            self.dimension = metadata.get('dimension', self.dimension)
            stored_has_numpy = metadata.get('has_numpy', True)
            
            # Load documents
            docs_file = self.index_path / "documents.json"
            if docs_file.exists():
                with open(docs_file) as f:
                    # Convert string keys back to integers
                    docs_data = json.load(f)
                    self.documents = {int(k): v for k, v in docs_data.items()}
            
            # Load vectors
            if HAS_NUMPY and (self.index_path / "vectors.npy").exists():
                self.vectors = np.load(self.index_path / "vectors.npy")
            elif (self.index_path / "vectors.json").exists():
                with open(self.index_path / "vectors.json") as f:
                    vectors_data = json.load(f)
                    if HAS_NUMPY:
                        self.vectors = np.array(vectors_data, dtype=np.float32)
                    else:
                        self.vectors = vectors_data
            
            logger.info(f"Loaded vector store from {self.index_path} ({len(self.documents)} documents)")
            
        except Exception as e:
            logger.warning(f"Failed to load vector store: {e}")
    
    def clear(self):
        """Clear all stored vectors and documents"""
        if HAS_NUMPY:
            self.vectors = np.empty((0, self.dimension), dtype=np.float32)
        else:
            self.vectors = []
        
        self.documents = {}
        self.next_id = 0
        logger.info("Cleared vector store")
    
    def get_stats(self) -> Dict:
        """Get statistics about the vector store"""
        return {
            'num_documents': len(self.documents),
            'dimension': self.dimension,
            'backend': 'numpy' if HAS_NUMPY else 'python',
            'size_mb': self._estimate_size_mb()
        }
    
    def _estimate_size_mb(self) -> float:
        """Estimate storage size in MB"""
        vector_size = len(self.documents) * self.dimension * 4  # 4 bytes per float32
        metadata_size = len(json.dumps(self.documents).encode('utf-8'))
        return (vector_size + metadata_size) / (1024 * 1024)