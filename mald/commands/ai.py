"""
MALD AI command - AI and RAG management
"""

import logging
import subprocess
from pathlib import Path
from ..rag.rag_engine import RAGEngine


logger = logging.getLogger(__name__)


def handle(args):
    """Handle the ai command"""
    if not args.ai_action:
        logger.error("No AI action specified")
        return 1
    
    if args.ai_action == 'setup':
        return _setup_ai()
    elif args.ai_action == 'chat':
        return _start_chat()
    elif args.ai_action == 'index':
        return _index_kb(args.kb)
    elif args.ai_action == 'search':
        return _search_kb(args.kb, args.query)
    elif args.ai_action == 'ask':
        return _ask_kb(args.kb, args.query)
    else:
        logger.error(f"Unknown AI action: {args.ai_action}")
        return 1


def _setup_ai():
    """Setup local AI models"""
    logger.info("Setting up local AI models...")
    
    mald_home = Path.home() / '.mald'
    ai_dir = mald_home / 'ai'
    models_dir = ai_dir / 'models'
    
    if not ai_dir.exists():
        logger.error("MALD environment not initialized. Run 'mald init' first.")
        return 1
    
    # Check for Ollama
    if _command_exists('ollama'):
        logger.info("Ollama found, setting up models...")
        _setup_ollama()
    else:
        logger.warning("Ollama not found. Installing Ollama...")
        _install_ollama()
    
    # Setup llama.cpp if available
    if _command_exists('llama-cpp-python'):
        logger.info("llama.cpp found")
    else:
        logger.info("llama.cpp not found. Consider installing for additional model support.")
    
    logger.info("AI setup completed")
    return 0


def _setup_ollama():
    """Setup Ollama models"""
    try:
        # Pull a small, fast model for local use
        models_to_pull = [
            'llama3.2:1b',  # Small, fast model
            'nomic-embed-text',  # For embeddings/RAG
        ]
        
        for model in models_to_pull:
            logger.info(f"Pulling model: {model}")
            result = subprocess.run(['ollama', 'pull', model], capture_output=True, text=True)
            if result.returncode == 0:
                logger.info(f"Successfully pulled {model}")
            else:
                logger.warning(f"Failed to pull {model}: {result.stderr}")
        
    except Exception as e:
        logger.error(f"Failed to setup Ollama models: {e}")


def _install_ollama():
    """Install Ollama"""
    try:
        logger.info("Installing Ollama...")
        # Install script from ollama.ai
        install_cmd = 'curl -fsSL https://ollama.ai/install.sh | sh'
        result = subprocess.run(install_cmd, shell=True, capture_output=True, text=True)
        
        if result.returncode == 0:
            logger.info("Ollama installed successfully")
            _setup_ollama()
        else:
            logger.error(f"Failed to install Ollama: {result.stderr}")
            
    except Exception as e:
        logger.error(f"Failed to install Ollama: {e}")


def _start_chat():
    """Start AI chat session"""
    logger.info("Starting AI chat session...")
    
    if not _command_exists('ollama'):
        logger.error("Ollama not found. Run 'mald ai setup' first.")
        return 1
    
    try:
        # Check if Ollama service is running
        result = subprocess.run(['ollama', 'list'], capture_output=True, text=True)
        if result.returncode != 0:
            logger.info("Starting Ollama service...")
            subprocess.run(['ollama', 'serve'], capture_output=True)
        
        # Start chat with default model
        logger.info("Starting chat with llama3.2:1b...")
        print("\n" + "="*50)
        print("  MALD AI Chat Session")
        print("  Type 'exit' or Ctrl+C to quit")
        print("="*50 + "\n")
        
        subprocess.run(['ollama', 'run', 'llama3.2:1b'])
        return 0
        
    except KeyboardInterrupt:
        print("\nChat session ended.")
        return 0
    except Exception as e:
        logger.error(f"Failed to start chat: {e}")
        return 1


def _index_kb(kb_name):
    """Index knowledge base for RAG"""
    logger.info(f"Indexing knowledge base: {kb_name}")
    
    mald_home = Path.home() / '.mald'
    kb_path = mald_home / 'kb' / kb_name
    index_dir = mald_home / 'ai' / 'index' / kb_name
    
    if not kb_path.exists():
        logger.error(f"Knowledge base '{kb_name}' not found")
        return 1
    
    try:
        # Initialize RAG engine
        rag_engine = RAGEngine(kb_path, index_dir)
        
        # Index the knowledge base
        indexed_count = rag_engine.index_knowledge_base(force_reindex=False)
        
        # Get stats
        stats = rag_engine.get_stats()
        
        logger.info(f"Indexing completed!")
        logger.info(f"Indexed documents: {indexed_count}")
        logger.info(f"Total documents in index: {stats['indexed_files']}")
        logger.info(f"Vector store stats: {stats['vector_store']}")
        
        return 0
        
    except Exception as e:
        logger.error(f"Failed to index knowledge base: {e}")
        return 1


def _search_kb(kb_name, query):
    """Search knowledge base using RAG"""
    if not query:
        logger.error("No search query provided")
        return 1
    
    mald_home = Path.home() / '.mald'
    kb_path = mald_home / 'kb' / kb_name
    index_dir = mald_home / 'ai' / 'index' / kb_name
    
    if not kb_path.exists():
        logger.error(f"Knowledge base '{kb_name}' not found")
        return 1
    
    try:
        # Initialize RAG engine
        rag_engine = RAGEngine(kb_path, index_dir)
        
        # Search for relevant documents
        results = rag_engine.search(query, k=5, min_score=0.1)
        
        if not results:
            logger.info("No relevant documents found")
            return 0
        
        print(f"\nSearch results for '{query}' in {kb_name}:")
        print("=" * 60)
        
        for i, result in enumerate(results, 1):
            doc = result['document']
            score = result['score']
            snippet = result['snippet']
            
            print(f"\n{i}. {doc['title']} (Score: {score:.3f})")
            print(f"   Path: {doc['relative_path']}")
            if doc['tags']:
                print(f"   Tags: {', '.join(doc['tags'])}")
            print(f"   Snippet: {snippet}")
        
        return 0
        
    except Exception as e:
        logger.error(f"Failed to search knowledge base: {e}")
        return 1


def _ask_kb(kb_name, query):
    """Ask a question to the knowledge base using RAG"""
    if not query:
        logger.error("No question provided")
        return 1
    
    mald_home = Path.home() / '.mald'
    kb_path = mald_home / 'kb' / kb_name
    index_dir = mald_home / 'ai' / 'index' / kb_name
    
    if not kb_path.exists():
        logger.error(f"Knowledge base '{kb_name}' not found")
        return 1
    
    try:
        # Initialize RAG engine
        rag_engine = RAGEngine(kb_path, index_dir)
        
        # Search for relevant context
        context_docs = rag_engine.search(query, k=3, min_score=0.1)
        
        if not context_docs:
            logger.info("No relevant documents found to answer the question")
            return 0
        
        print(f"\nQuestion: {query}")
        print("=" * 60)
        
        # Show retrieved context
        print("\nRelevant documents found:")
        for i, result in enumerate(context_docs, 1):
            doc = result['document']
            score = result['score']
            print(f"  {i}. {doc['title']} (Score: {score:.3f})")
        
        print("\nGenerating response...")
        
        # Generate response using RAG
        response = rag_engine.generate_response(query, context_docs)
        
        print(f"\nAnswer:")
        print("-" * 40)
        print(response)
        print()
        
        return 0
        
    except Exception as e:
        logger.error(f"Failed to process question: {e}")
        return 1


def _command_exists(command):
    """Check if a command exists in PATH"""
    return subprocess.run(
        ['which', command], 
        capture_output=True, 
        text=True
    ).returncode == 0