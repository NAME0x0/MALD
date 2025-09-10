# MALD Personal Knowledge Management Guide

MALD provides an Obsidian-style PKM system with additional features like executable code blocks and AI integration.

## Core Concepts

### Knowledge Bases
Knowledge bases are collections of interconnected markdown documents. Each knowledge base is a separate namespace for organizing information.

```bash
# Create new knowledge base
mald kb create project-alpha

# List all knowledge bases
mald kb list

# Open knowledge base
mald kb open project-alpha
```

### Linking System

#### Wikilinks
Use double brackets for internal links:
```markdown
[[Document Title]]           # Link to document
[[Document Title|Display]]   # Link with custom display text
```

#### Tags
Use hashtags for categorization:
```markdown
#project #important #draft
```

#### Markdown Links
Standard markdown links work for external resources:
```markdown
[External Link](https://example.com)
[Local File](./file.md)
```

### Document Structure

#### Frontmatter (Optional)
```yaml
---
title: "Document Title"
tags: [project, important]
created: 2024-01-01
modified: 2024-01-01
---
```

#### Headers
Use ATX-style headers for structure:
```markdown
# Main Title (H1)
## Section (H2)
### Subsection (H3)
```

## Executable Code Blocks

MALD supports Jupyter-style executable code blocks within markdown:

### Python Example
```python
# Execute with Ctrl+Enter (in editor) or through MALD session
import datetime
import os

print(f"Current time: {datetime.datetime.now()}")
print(f"MALD home: {os.getenv('MALD_HOME', '~/.mald')}")

# Variables persist between executions
data = {"notes": 42, "projects": 7}
print(f"Stats: {data}")
```

### Bash Example
```bash
# System information
uname -a
df -h ~/.mald
ls -la ~/.mald/kb/
```

### SQL Example (with database)
```sql
-- Query knowledge base metadata
SELECT 
    name,
    COUNT(*) as note_count,
    MAX(modified) as last_modified
FROM notes 
GROUP BY kb_name;
```

## Knowledge Base Organization

### Recommended Structure
```
kb/
├── index.md              # Main entry point
├── daily/                # Daily notes
│   ├── 2024-01-01.md
│   └── 2024-01-02.md
├── projects/             # Project documentation
│   ├── project-alpha.md
│   └── project-beta.md
├── areas/                # Areas of responsibility
│   ├── health.md
│   └── learning.md
├── resources/            # Reference materials
│   ├── books.md
│   └── articles.md
└── archive/              # Completed/archived items
    └── old-project.md
```

### Templates
Create templates for common note types:

#### Daily Note Template
```markdown
# {{date}}

## Goals
- [ ] 

## Notes


## Ideas


## Links
- [[Yesterday]] | [[Tomorrow]]

---
Tags: #daily
```

#### Project Template
```markdown
# {{title}}

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
```

## Search and Discovery

### Full-Text Search
```bash
# Search across all knowledge bases
mald search "machine learning"

# Search specific knowledge base
mald search "python" --kb project-alpha

# Case-sensitive search
mald search "API" --case-sensitive
```

### Graph Navigation
MALD builds a knowledge graph of all documents and links:

```bash
# Generate graph data
mald graph generate --kb project-alpha

# Find orphaned documents (no incoming links)
mald graph orphans --kb project-alpha

# Find most connected documents
mald graph hubs --kb project-alpha
```

### Backlinks
Find all documents that link to current document:

```bash
# Find backlinks for a document
mald backlinks "Document Title" --kb project-alpha
```

## AI Integration

### RAG (Retrieval Augmented Generation)
Use local AI to enhance your knowledge base:

```bash
# Index knowledge base for AI search
mald ai index project-alpha

# Query with AI assistance
mald ai ask "What are the main themes in my notes about Python?"

# Chat with knowledge base context
mald ai chat --kb project-alpha
```

### Content Generation
```bash
# Generate summary of knowledge base
mald ai summarize --kb project-alpha

# Generate content suggestions
mald ai suggest --topic "machine learning" --kb project-alpha

# Expand on existing notes
mald ai expand "Brief Note Title" --kb project-alpha
```

## Workflows

### Daily Workflow
```bash
# Morning routine
mald session --kb daily
# - Review yesterday's notes
# - Create today's note
# - Plan tasks

# Throughout the day
# - Capture thoughts in daily note
# - Link to relevant project notes
# - Execute code for quick calculations

# Evening routine
# - Review and organize notes
# - Archive completed tasks
# - Plan tomorrow
```

### Project Workflow
```bash
# Project setup
mald kb create project-name
# - Create project overview
# - Set up resource links
# - Define goals and milestones

# Project work
mald session --kb project-name
# - Update project notes
# - Link related resources
# - Track progress

# Project completion
# - Create project summary
# - Archive to archive/
# - Extract lessons learned
```

### Research Workflow
```bash
# Research collection
# - Create research topic note
# - Collect sources with links
# - Tag with research area

# Analysis
# - Use AI to summarize sources
# - Create synthesis notes
# - Link related concepts

# Knowledge integration
# - Connect to existing knowledge
# - Update relevant project notes
# - Share insights with team
```

## Best Practices

### Naming Conventions
- Use descriptive, unique titles
- Avoid special characters in filenames
- Use consistent date formats (YYYY-MM-DD)
- Prefix with category when helpful

### Linking Strategy
- Link liberally - connections create value
- Use consistent naming for common concepts
- Create hub pages for major topics
- Link both directions when relevant

### Tagging Strategy
- Use hierarchical tags (#project/alpha)
- Create tag ontology document
- Review and clean tags regularly
- Use tags for cross-cutting concerns

### Content Organization
- Keep atomic notes (one concept per note)
- Use headers for internal structure
- Include context in note titles
- Regular review and refactoring

### Backup and Sync
```bash
# Create backup
mald backup create full-backup

# Export knowledge base
mald export --kb project-alpha --format markdown

# Sync with git (recommended)
cd ~/.mald/kb/project-alpha
git init
git add .
git commit -m "Initial knowledge base"
git remote add origin <repository-url>
git push -u origin main
```

## Advanced Features

### Custom Scripts
Create custom automation scripts:

```python
# ~/.mald/scripts/daily_summary.py
import os
from pathlib import Path
from mald.utils import markdown_parser

def generate_daily_summary():
    kb_path = Path.home() / '.mald' / 'kb' / 'daily'
    documents = markdown_parser.parse_knowledge_base(kb_path)
    
    # Generate summary logic
    pass

if __name__ == '__main__':
    generate_daily_summary()
```

### Integration with External Tools
- Export to Anki for spaced repetition
- Sync with external note-taking apps
- Generate reports and dashboards
- Integration with calendar and task systems

### Performance Tips
- Use knowledge base per major topic
- Regular cleanup of orphaned files
- Optimize large knowledge bases with archiving
- Use indexes for large document collections

## Troubleshooting

### Common Issues
- **Broken links**: Use `mald graph check` to find broken links
- **Slow search**: Rebuild search index with `mald index rebuild`
- **Large files**: Split large documents into smaller notes
- **Sync conflicts**: Use git-based versioning for collaboration

### Maintenance
```bash
# Health check
mald doctor --kb project-alpha

# Rebuild indexes
mald index rebuild --kb project-alpha

# Cleanup orphaned files
mald cleanup --kb project-alpha

# Validate links
mald graph validate --kb project-alpha
```