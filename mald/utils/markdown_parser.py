"""
MALD markdown parser utilities
"""

import re
import logging
from pathlib import Path
from typing import List, Dict, Set


logger = logging.getLogger(__name__)


class MarkdownDocument:
    """Represents a markdown document with MALD features"""
    
    def __init__(self, path: Path, content: str = None):
        self.path = path
        self.content = content or self._load_content()
        self.title = self._extract_title()
        self.links = self._extract_links()
        self.tags = self._extract_tags()
        self.code_blocks = self._extract_code_blocks()
        self.metadata = self._extract_metadata()
    
    def _load_content(self):
        """Load content from file"""
        try:
            return self.path.read_text(encoding='utf-8')
        except Exception as e:
            logger.error(f"Failed to load {self.path}: {e}")
            return ""
    
    def _extract_title(self):
        """Extract document title"""
        # Try H1 heading first
        h1_match = re.search(r'^#\s+(.+)$', self.content, re.MULTILINE)
        if h1_match:
            return h1_match.group(1).strip()
        
        # Fall back to filename
        return self.path.stem
    
    def _extract_links(self):
        """Extract wikilinks and markdown links"""
        links = set()
        
        # Wikilinks [[link]]
        wikilinks = re.findall(r'\[\[([^\]]+)\]\]', self.content)
        for link in wikilinks:
            # Handle [[link|display]] format
            actual_link = link.split('|')[0].strip()
            links.add(actual_link)
        
        # Markdown links [text](url)
        md_links = re.findall(r'\[([^\]]+)\]\(([^)]+)\)', self.content)
        for text, url in md_links:
            # Only include local markdown files
            if url.endswith('.md') and not url.startswith('http'):
                links.add(url)
        
        return list(links)
    
    def _extract_tags(self):
        """Extract hashtags"""
        tags = re.findall(r'#([a-zA-Z0-9_-]+)', self.content)
        return list(set(tags))  # Remove duplicates
    
    def _extract_code_blocks(self):
        """Extract code blocks with language and content"""
        code_blocks = []
        
        # Find fenced code blocks
        pattern = r'```(\w+)?\n(.*?)\n```'
        matches = re.finditer(pattern, self.content, re.DOTALL)
        
        for match in matches:
            language = match.group(1) or 'text'
            code = match.group(2).strip()
            
            code_blocks.append({
                'language': language,
                'code': code,
                'start': match.start(),
                'end': match.end()
            })
        
        return code_blocks
    
    def _extract_metadata(self):
        """Extract YAML frontmatter metadata"""
        metadata = {}
        
        # Check for YAML frontmatter
        frontmatter_match = re.match(r'^---\n(.*?)\n---\n', self.content, re.DOTALL)
        if frontmatter_match:
            try:
                import yaml
                metadata = yaml.safe_load(frontmatter_match.group(1))
            except ImportError:
                logger.warning("PyYAML not available, skipping frontmatter parsing")
            except Exception as e:
                logger.warning(f"Failed to parse frontmatter: {e}")
        
        return metadata or {}
    
    def get_backlinks(self, kb_path: Path):
        """Find documents that link to this one"""
        backlinks = []
        target_name = self.path.stem
        
        for md_file in kb_path.glob('**/*.md'):
            if md_file == self.path:
                continue
            
            try:
                content = md_file.read_text(encoding='utf-8')
                
                # Check for wikilinks to this document
                wikilink_pattern = rf'\[\[{re.escape(target_name)}(\|[^\]]+)?\]\]'
                if re.search(wikilink_pattern, content):
                    backlinks.append(md_file)
                
                # Check for markdown links
                md_link_pattern = rf'\]\({re.escape(self.path.name)}\)'
                if re.search(md_link_pattern, content):
                    backlinks.append(md_file)
            
            except Exception as e:
                logger.warning(f"Failed to check backlinks in {md_file}: {e}")
        
        return backlinks


def parse_knowledge_base(kb_path: Path):
    """Parse all markdown files in a knowledge base"""
    documents = {}
    
    for md_file in kb_path.glob('**/*.md'):
        try:
            doc = MarkdownDocument(md_file)
            documents[str(md_file.relative_to(kb_path))] = doc
        except Exception as e:
            logger.error(f"Failed to parse {md_file}: {e}")
    
    return documents


def find_orphaned_files(kb_path: Path):
    """Find markdown files with no incoming links"""
    documents = parse_knowledge_base(kb_path)
    linked_files = set()
    
    for doc in documents.values():
        for link in doc.links:
            # Convert link to potential file path
            link_path = link if link.endswith('.md') else f"{link}.md"
            linked_files.add(link_path)
    
    orphaned = []
    for rel_path, doc in documents.items():
        if rel_path not in linked_files and rel_path != 'index.md':
            orphaned.append(doc)
    
    return orphaned


def generate_graph_data(kb_path: Path):
    """Generate graph data for visualization"""
    documents = parse_knowledge_base(kb_path)
    
    nodes = []
    edges = []
    
    for rel_path, doc in documents.items():
        # Create node
        nodes.append({
            'id': rel_path,
            'title': doc.title,
            'tags': doc.tags,
            'path': str(doc.path)
        })
        
        # Create edges for links
        for link in doc.links:
            target_path = link if link.endswith('.md') else f"{link}.md"
            if target_path in documents:
                edges.append({
                    'source': rel_path,
                    'target': target_path,
                    'type': 'link'
                })
    
    return {'nodes': nodes, 'edges': edges}


def search_content(kb_path: Path, query: str, case_sensitive: bool = False):
    """Search for content across all documents"""
    results = []
    documents = parse_knowledge_base(kb_path)
    
    flags = 0 if case_sensitive else re.IGNORECASE
    
    for rel_path, doc in documents.items():
        matches = []
        
        for match in re.finditer(re.escape(query), doc.content, flags):
            # Get context around the match
            start = max(0, match.start() - 50)
            end = min(len(doc.content), match.end() + 50)
            context = doc.content[start:end]
            
            matches.append({
                'position': match.start(),
                'context': context.strip()
            })
        
        if matches:
            results.append({
                'document': doc,
                'matches': matches,
                'match_count': len(matches)
            })
    
    return sorted(results, key=lambda x: x['match_count'], reverse=True)