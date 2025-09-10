"""
MALD knowledge base command - Manage knowledge bases
"""

import logging
from pathlib import Path
from ..utils import filesystem, markdown_parser


logger = logging.getLogger(__name__)


def handle(args):
    """Handle the kb command"""
    if not args.kb_action:
        logger.error("No knowledge base action specified")
        return 1
    
    if args.kb_action == 'create':
        return _create_kb(args.name)
    elif args.kb_action == 'list':
        return _list_kbs()
    elif args.kb_action == 'open':
        return _open_kb(args.name)
    else:
        logger.error(f"Unknown knowledge base action: {args.kb_action}")
        return 1


def _create_kb(name):
    """Create a new knowledge base"""
    mald_home = Path.home() / '.mald'
    kb_path = mald_home / 'kb' / name
    
    if kb_path.exists():
        logger.error(f"Knowledge base '{name}' already exists")
        return 1
    
    try:
        kb_path.mkdir(parents=True)
        
        # Create index file
        index_file = kb_path / 'index.md'
        index_content = f"""# {name}

Welcome to your **{name}** knowledge base.

## Structure

- [[Daily Notes]] - Daily notes and thoughts
- [[Projects]] - Project documentation and planning
- [[References]] - External links and references
- [[Ideas]] - Random ideas and inspirations

## Tags

Use tags to organize your content:
- #daily
- #project
- #reference
- #idea
- #todo

## Getting Started

1. Create your first note: `[[My First Note]]`
2. Use executable code blocks for dynamic content
3. Link related concepts with double brackets
4. Tag everything for easy retrieval

---
*Knowledge base created: {name}*
"""
        
        index_file.write_text(index_content)
        
        # Create basic structure
        templates_dir = kb_path / 'templates'
        templates_dir.mkdir()
        
        daily_template = templates_dir / 'daily.md'
        daily_template.write_text("""# {{date}}

## Goals
- [ ] 

## Notes


## Ideas


## Links
- 

---
Tags: #daily
""")
        
        project_template = templates_dir / 'project.md'
        project_template.write_text("""# {{title}}

## Overview


## Goals
- [ ] 

## Resources
- 

## Progress


## Next Steps
- [ ] 

---
Tags: #project
""")
        
        logger.info(f"Created knowledge base: {name}")
        logger.info(f"Location: {kb_path}")
        logger.info(f"Open with: mald kb open {name}")
        
        return 0
        
    except Exception as e:
        logger.error(f"Failed to create knowledge base: {e}")
        return 1


def _list_kbs():
    """List all knowledge bases"""
    mald_home = Path.home() / '.mald'
    kb_dir = mald_home / 'kb'
    
    if not kb_dir.exists():
        logger.info("No knowledge bases found. Create one with 'mald kb create <name>'")
        return 0
    
    kbs = [d for d in kb_dir.iterdir() if d.is_dir()]
    
    if not kbs:
        logger.info("No knowledge bases found. Create one with 'mald kb create <name>'")
        return 0
    
    logger.info("Available knowledge bases:")
    for kb in sorted(kbs):
        index_file = kb / 'index.md'
        note_count = len(list(kb.glob('*.md')))
        
        status = "✓" if index_file.exists() else "!"
        logger.info(f"  {status} {kb.name} ({note_count} notes)")
    
    return 0


def _open_kb(name):
    """Open a knowledge base"""
    mald_home = Path.home() / '.mald'
    kb_path = mald_home / 'kb' / name
    
    if not kb_path.exists():
        logger.error(f"Knowledge base '{name}' not found")
        logger.info("Available knowledge bases:")
        _list_kbs()
        return 1
    
    index_file = kb_path / 'index.md'
    
    if index_file.exists():
        logger.info(f"Opening knowledge base: {name}")
        logger.info(f"Location: {kb_path}")
        
        # For now, just print the index content
        # In a full implementation, this would launch the editor
        try:
            content = index_file.read_text()
            print("\n" + "="*50)
            print(content)
            print("="*50 + "\n")
        except Exception as e:
            logger.error(f"Failed to read index file: {e}")
            return 1
    else:
        logger.warning(f"Knowledge base '{name}' exists but has no index file")
        logger.info(f"Location: {kb_path}")
    
    return 0