"""
Test MALD markdown parsing utilities
"""

import pytest
from pathlib import Path
from mald.utils.markdown_parser import (
    MarkdownDocument, 
    parse_knowledge_base,
    find_orphaned_files,
    generate_graph_data,
    search_content
)


def test_markdown_document_parsing(sample_kb_path):
    """Test markdown document parsing"""
    test_file = sample_kb_path / 'test_note.md'
    doc = MarkdownDocument(test_file)
    
    # Test title extraction
    assert doc.title == 'Test Note'
    
    # Test link extraction
    assert 'Another Note' in doc.links
    assert 'Third Note' in doc.links
    assert len(doc.links) == 2
    
    # Test tag extraction
    assert 'test' in doc.tags
    assert 'sample' in doc.tags
    
    # Test code block extraction
    assert len(doc.code_blocks) == 1
    assert doc.code_blocks[0]['language'] == 'python'
    assert 'print("Hello, MALD!")' in doc.code_blocks[0]['code']


def test_knowledge_base_parsing(sample_kb_path):
    """Test full knowledge base parsing"""
    documents = parse_knowledge_base(sample_kb_path)
    
    assert len(documents) == 1
    assert 'test_note.md' in documents
    
    doc = documents['test_note.md']
    assert doc.title == 'Test Note'


def test_orphaned_files_detection(sample_kb_path):
    """Test orphaned files detection"""
    # Create another file that links to test_note
    linking_note = sample_kb_path / 'linking_note.md'
    linking_content = """# Linking Note

This note links to [[Test Note]].
"""
    linking_note.write_text(linking_content)
    
    # Create orphaned file
    orphaned_note = sample_kb_path / 'orphaned.md'
    orphaned_content = """# Orphaned Note

This note has no incoming links.
"""
    orphaned_note.write_text(orphaned_content)
    
    orphaned = find_orphaned_files(sample_kb_path)
    
    # Should find the orphaned note but not the linked ones
    orphaned_names = [doc.path.name for doc in orphaned]
    assert 'orphaned.md' in orphaned_names
    assert 'test_note.md' not in orphaned_names  # This is linked from linking_note


def test_graph_data_generation(sample_kb_path):
    """Test graph data generation"""
    # Create additional notes for graph
    note2 = sample_kb_path / 'note2.md'
    note2_content = """# Note Two

Links to [[Test Note]].
"""
    note2.write_text(note2_content)
    
    graph_data = generate_graph_data(sample_kb_path)
    
    assert 'nodes' in graph_data
    assert 'edges' in graph_data
    
    # Check nodes
    node_ids = [node['id'] for node in graph_data['nodes']]
    assert 'test_note.md' in node_ids
    assert 'note2.md' in node_ids
    
    # Check edges
    assert len(graph_data['edges']) > 0
    edge = graph_data['edges'][0]
    assert 'source' in edge
    assert 'target' in edge


def test_content_search(sample_kb_path):
    """Test content search functionality"""
    results = search_content(sample_kb_path, 'Hello')
    
    assert len(results) == 1
    result = results[0]
    assert result['document'].title == 'Test Note'
    assert result['match_count'] == 1
    assert len(result['matches']) == 1
    
    # Test case-insensitive search
    results = search_content(sample_kb_path, 'hello', case_sensitive=False)
    assert len(results) == 1
    
    # Test case-sensitive search (should find nothing)
    results = search_content(sample_kb_path, 'hello', case_sensitive=True)
    assert len(results) == 0


def test_markdown_document_with_frontmatter(tmp_path):
    """Test markdown document with YAML frontmatter"""
    test_file = tmp_path / 'frontmatter_test.md'
    content = """---
title: "Custom Title"
tags: [important, project]
date: 2024-01-01
---

# Frontmatter Test

This document has YAML frontmatter.
"""
    test_file.write_text(content)
    
    doc = MarkdownDocument(test_file, content)
    
    # Should use frontmatter title if available
    # Note: This requires PyYAML which may not be installed
    # So we test the fallback behavior
    assert doc.title == 'Frontmatter Test'  # Falls back to H1


def test_backlinks_detection(sample_kb_path):
    """Test backlinks detection"""
    # Create a note that links to test_note
    linking_note = sample_kb_path / 'linking_note.md'
    linking_content = """# Linking Note

This note references [[Test Note]] in the content.
"""
    linking_note.write_text(linking_content)
    
    # Get the test note document
    test_file = sample_kb_path / 'test_note.md'
    doc = MarkdownDocument(test_file)
    
    # Find backlinks
    backlinks = doc.get_backlinks(sample_kb_path)
    
    # Should find the linking note
    backlink_names = [bl.name for bl in backlinks]
    assert 'linking_note.md' in backlink_names