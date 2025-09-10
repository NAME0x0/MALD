"""
MALD AI command - AI and RAG management
"""

import logging
import subprocess
from pathlib import Path


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
    
    # Create index directory
    index_dir.mkdir(parents=True, exist_ok=True)
    
    # Find all markdown files
    md_files = list(kb_path.glob('**/*.md'))
    
    if not md_files:
        logger.warning(f"No markdown files found in {kb_name}")
        return 0
    
    logger.info(f"Found {len(md_files)} markdown files to index")
    
    # For now, just create a simple index file
    # In a full implementation, this would use proper vector embeddings
    index_file = index_dir / 'files.txt'
    
    with open(index_file, 'w') as f:
        for md_file in md_files:
            rel_path = md_file.relative_to(kb_path)
            f.write(f"{rel_path}\n")
    
    logger.info(f"Index created: {index_file}")
    logger.info("Note: Full vector indexing requires additional implementation")
    
    return 0


def _command_exists(command):
    """Check if a command exists in PATH"""
    return subprocess.run(
        ['which', command], 
        capture_output=True, 
        text=True
    ).returncode == 0