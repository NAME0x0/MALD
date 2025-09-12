"""
Parser for extracting executable code cells from markdown
"""

import re
import logging
from pathlib import Path
from typing import List, Dict, Tuple
from ..utils.markdown_parser import MarkdownDocument

logger = logging.getLogger(__name__)


class CellParser:
    """Parses markdown files to extract executable code cells"""
    
    def __init__(self):
        self.executable_markers = [
            'exec',
            'execute',
            'run',
            'eval'
        ]
    
    def parse_file(self, file_path: Path) -> List[Dict]:
        """Parse a markdown file and extract executable code cells"""
        try:
            doc = MarkdownDocument(file_path)
            return self.parse_content(doc.content)
        except Exception as e:
            logger.error(f"Failed to parse file {file_path}: {e}")
            return []
    
    def parse_content(self, content: str) -> List[Dict]:
        """Parse markdown content and extract executable code cells"""
        cells = []
        
        # Find all fenced code blocks
        pattern = r'```(\w+)(?:\s+(.+?))?\n(.*?)\n```'
        matches = re.finditer(pattern, content, re.DOTALL)
        
        for match in matches:
            language = match.group(1) or 'text'
            attributes = match.group(2) or ''
            code = match.group(3)
            
            # Check if this code block should be executable
            is_executable = self._is_executable(language, attributes)
            
            cell = {
                'language': language,
                'code': code,
                'attributes': attributes,
                'executable': is_executable,
                'start_pos': match.start(),
                'end_pos': match.end(),
                'cell_id': self._generate_cell_id(match.start(), language)
            }
            
            cells.append(cell)
        
        return cells
    
    def _is_executable(self, language: str, attributes: str) -> bool:
        """Determine if a code block should be executable"""
        # Check language
        executable_languages = [
            'python', 'py',
            'bash', 'sh', 'shell',
            'javascript', 'js', 'node',
            'sql', 'sqlite'
        ]
        
        if language.lower() in executable_languages:
            # Check for explicit disable markers
            if any(marker in attributes.lower() for marker in ['no-exec', 'no-run', 'static']):
                return False
            return True
        
        # Check for explicit enable markers
        if any(marker in attributes.lower() for marker in self.executable_markers):
            return True
        
        return False
    
    def _generate_cell_id(self, position: int, language: str) -> str:
        """Generate a unique cell ID"""
        return f"{language}_{position}"
    
    def extract_executable_cells(self, file_path: Path) -> List[Dict]:
        """Extract only executable code cells from a file"""
        all_cells = self.parse_file(file_path)
        return [cell for cell in all_cells if cell['executable']]
    
    def find_cell_by_id(self, file_path: Path, cell_id: str) -> Dict:
        """Find a specific cell by ID"""
        cells = self.parse_file(file_path)
        for cell in cells:
            if cell['cell_id'] == cell_id:
                return cell
        return None
    
    def update_cell_output(self, content: str, cell: Dict, output: str, error: str = None) -> str:
        """Update markdown content with cell execution output"""
        # Find the position after the code block
        end_pos = cell['end_pos']
        
        # Look for existing output block
        output_pattern = r'<!-- MALD_OUTPUT_START -->(.*?)<!-- MALD_OUTPUT_END -->'
        existing_output = re.search(output_pattern, content[end_pos:], re.DOTALL)
        
        # Create new output block
        output_block = self._create_output_block(output, error)
        
        if existing_output:
            # Replace existing output
            start_replace = end_pos + existing_output.start()
            end_replace = end_pos + existing_output.end()
            new_content = content[:start_replace] + output_block + content[end_replace:]
        else:
            # Insert new output block after the code block
            new_content = content[:end_pos] + '\n\n' + output_block + content[end_pos:]
        
        return new_content
    
    def _create_output_block(self, output: str, error: str = None) -> str:
        """Create a formatted output block"""
        block = "<!-- MALD_OUTPUT_START -->\n"
        
        if output:
            block += "**Output:**\n```\n"
            block += output.strip()
            block += "\n```\n"
        
        if error:
            block += "\n**Error:**\n```\n"
            block += error.strip()
            block += "\n```\n"
        
        if not output and not error:
            block += "*No output*\n"
        
        block += "<!-- MALD_OUTPUT_END -->"
        
        return block
    
    def remove_all_outputs(self, content: str) -> str:
        """Remove all execution outputs from markdown content"""
        output_pattern = r'\n\n<!-- MALD_OUTPUT_START -->.*?<!-- MALD_OUTPUT_END -->'
        return re.sub(output_pattern, '', content, flags=re.DOTALL)
    
    def get_cell_summary(self, file_path: Path) -> Dict:
        """Get summary of code cells in a file"""
        cells = self.parse_file(file_path)
        
        summary = {
            'total_cells': len(cells),
            'executable_cells': len([c for c in cells if c['executable']]),
            'languages': list(set(c['language'] for c in cells)),
            'executable_languages': list(set(c['language'] for c in cells if c['executable']))
        }
        
        return summary