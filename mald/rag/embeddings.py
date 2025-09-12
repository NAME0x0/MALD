"""
Embedding engine for generating vector representations of text
"""

import logging
from pathlib import Path
from typing import List, Union, Optional

# Try to import numpy, fallback to built-in list if not available
try:
    import numpy as np
    HAS_NUMPY = True
except ImportError:
    HAS_NUMPY = False
    np = None

logger = logging.getLogger(__name__)


class EmbeddingEngine:
    """Handles text embedding generation using various backends"""
    
    def __init__(self, model_name: str = "all-MiniLM-L6-v2", backend: str = "fallback"):
        self.model_name = model_name
        self.backend = backend
        self.model = None
        self._load_model()
    
    def _load_model(self):
        """Load the embedding model"""
        try:
            if self.backend == "sentence-transformers":
                self._load_sentence_transformers()
            elif self.backend == "ollama":
                self._load_ollama()
            else:
                self._create_fallback_model()
        except Exception as e:
            logger.error(f"Failed to load embedding model: {e}")
            self._create_fallback_model()
    
    def _load_sentence_transformers(self):
        """Load sentence-transformers model"""
        try:
            from sentence_transformers import SentenceTransformer
            self.model = SentenceTransformer(self.model_name)
            logger.info(f"Loaded sentence-transformers model: {self.model_name}")
        except ImportError:
            logger.warning("sentence-transformers not available, falling back to simple embeddings")
            self._create_fallback_model()
        except Exception as e:
            logger.error(f"Failed to load sentence-transformers model: {e}")
            self._create_fallback_model()
    
    def _load_ollama(self):
        """Load Ollama embedding model"""
        try:
            import requests
            # Test Ollama connection
            response = requests.get("http://localhost:11434/api/tags", timeout=5)
            if response.status_code == 200:
                self.model = "ollama"
                logger.info("Connected to Ollama for embeddings")
            else:
                raise Exception("Ollama not responding")
        except Exception as e:
            logger.warning(f"Ollama not available for embeddings: {e}")
            self._create_fallback_model()
    
    def _create_fallback_model(self):
        """Create a simple fallback embedding model"""
        self.model = "fallback"
        logger.info("Using fallback embedding model (hash-based)")
    
    def embed(self, texts: Union[str, List[str]]) -> Union[List[List[float]], 'np.ndarray']:
        """Generate embeddings for text(s)"""
        if isinstance(texts, str):
            texts = [texts]
        
        if self.model is None or self.model == "fallback":
            return self._fallback_embed(texts)
        
        try:
            if hasattr(self.model, 'encode') and callable(self.model.encode):
                # sentence-transformers model
                if HAS_NUMPY:
                    embeddings = self.model.encode(texts, convert_to_numpy=True)
                else:
                    embeddings = self.model.encode(texts)
                    embeddings = embeddings.tolist() if hasattr(embeddings, 'tolist') else embeddings
                return embeddings
            elif self.model == "ollama":
                return self._ollama_embed(texts)
            else:
                return self._fallback_embed(texts)
        except Exception as e:
            logger.error(f"Embedding generation failed: {e}")
            return self._fallback_embed(texts)
    
    def _ollama_embed(self, texts: List[str]) -> Union[List[List[float]], 'np.ndarray']:
        """Generate embeddings using Ollama"""
        try:
            import requests
            embeddings = []
            
            for text in texts:
                response = requests.post(
                    "http://localhost:11434/api/embeddings",
                    json={
                        "model": "nomic-embed-text",
                        "prompt": text
                    },
                    timeout=30
                )
                
                if response.status_code == 200:
                    embedding = response.json().get("embedding", [])
                    embeddings.append(embedding)
                else:
                    # Fallback for failed requests
                    embeddings.append(self._simple_hash_embed(text))
            
            if HAS_NUMPY:
                return np.array(embeddings)
            else:
                return embeddings
        except Exception as e:
            logger.error(f"Ollama embedding failed: {e}")
            return self._fallback_embed(texts)
    
    def _fallback_embed(self, texts: List[str]) -> Union[List[List[float]], 'np.ndarray']:
        """Simple hash-based embeddings as fallback"""
        embeddings = [self._simple_hash_embed(text) for text in texts]
        if HAS_NUMPY:
            return np.array(embeddings)
        else:
            return embeddings
    
    def _simple_hash_embed(self, text: str, dim: int = 384) -> List[float]:
        """Create a simple hash-based embedding"""
        import hashlib
        
        # Use multiple hash functions to create a pseudo-random vector
        embedding = []
        for i in range(dim // 32):  # Create groups of 32 values
            hash_input = f"{text}_{i}".encode('utf-8')
            hash_obj = hashlib.md5(hash_input)
            hex_dig = hash_obj.hexdigest()
            
            # Convert hex to floats
            for j in range(0, len(hex_dig), 2):
                if len(embedding) >= dim:
                    break
                val = int(hex_dig[j:j+2], 16) / 255.0 - 0.5  # Normalize to [-0.5, 0.5]
                embedding.append(val)
            
            if len(embedding) >= dim:
                break
        
        # Pad or truncate to desired dimension
        while len(embedding) < dim:
            embedding.append(0.0)
        
        return embedding[:dim]
    
    def get_dimension(self) -> int:
        """Get the dimension of embeddings produced by this model"""
        if hasattr(self.model, 'get_sentence_embedding_dimension'):
            return self.model.get_sentence_embedding_dimension()
        elif self.backend == "ollama":
            return 768  # Standard dimension for nomic-embed-text
        else:
            return 384  # Fallback dimension