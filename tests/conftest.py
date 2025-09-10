"""
Test configuration for MALD
"""

import os
import tempfile
import pytest
from pathlib import Path

@pytest.fixture
def temp_mald_home():
    """Create temporary MALD home directory for testing"""
    with tempfile.TemporaryDirectory() as temp_dir:
        mald_home = Path(temp_dir) / '.mald'
        mald_home.mkdir()
        
        # Set environment variable
        old_home = os.environ.get('HOME')
        os.environ['HOME'] = str(Path(temp_dir))
        
        yield mald_home
        
        # Restore environment
        if old_home:
            os.environ['HOME'] = old_home
        else:
            os.environ.pop('HOME', None)

@pytest.fixture
def sample_kb_path(temp_mald_home):
    """Create sample knowledge base for testing"""
    kb_path = temp_mald_home / 'kb' / 'test'
    kb_path.mkdir(parents=True)
    
    # Create sample markdown file
    sample_note = kb_path / 'test_note.md'
    sample_content = """# Test Note

This is a test note with [[links]] and #tags.

## Code Block

```python
print("Hello, MALD!")
```

## Links
- [[Another Note]]
- [[Third Note]]

Tags: #test #sample
"""
    sample_note.write_text(sample_content)
    
    return kb_path