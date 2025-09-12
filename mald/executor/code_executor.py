"""
Code executor for running code blocks in markdown files
"""

import logging
import subprocess
import tempfile
import os
from pathlib import Path
from typing import Dict, List, Optional, Tuple
import json

logger = logging.getLogger(__name__)


class CodeExecutor:
    """Executes code blocks with various language support"""
    
    def __init__(self, work_dir: Optional[Path] = None):
        self.work_dir = Path(work_dir) if work_dir else Path.cwd()
        self.supported_languages = {
            'python': self._execute_python,
            'bash': self._execute_bash,
            'sh': self._execute_bash,
            'javascript': self._execute_javascript,
            'js': self._execute_javascript,
            'node': self._execute_javascript,
            'sql': self._execute_sql,
        }
        
        # Persistent session data
        self.sessions = {}
    
    def execute_code(self, code: str, language: str, session_id: str = "default") -> Dict:
        """Execute code and return results"""
        if not language:
            language = 'text'
        
        language = language.lower()
        
        if language not in self.supported_languages:
            return {
                'success': False,
                'output': f"Unsupported language: {language}",
                'error': f"Language '{language}' is not supported",
                'execution_time': 0
            }
        
        try:
            import time
            start_time = time.time()
            
            result = self.supported_languages[language](code, session_id)
            
            execution_time = time.time() - start_time
            result['execution_time'] = execution_time
            
            return result
            
        except Exception as e:
            logger.error(f"Code execution failed: {e}")
            return {
                'success': False,
                'output': '',
                'error': str(e),
                'execution_time': 0
            }
    
    def _execute_python(self, code: str, session_id: str) -> Dict:
        """Execute Python code"""
        try:
            # Create or get session globals
            if session_id not in self.sessions:
                self.sessions[session_id] = {'globals': {}, 'locals': {}}
            
            session = self.sessions[session_id]
            
            # Capture stdout/stderr
            import io
            import sys
            
            old_stdout = sys.stdout
            old_stderr = sys.stderr
            stdout_capture = io.StringIO()
            stderr_capture = io.StringIO()
            
            try:
                sys.stdout = stdout_capture
                sys.stderr = stderr_capture
                
                # Execute code in session context
                exec(code, session['globals'], session['locals'])
                
                output = stdout_capture.getvalue()
                error = stderr_capture.getvalue()
                
                return {
                    'success': True,
                    'output': output,
                    'error': error if error else None
                }
                
            finally:
                sys.stdout = old_stdout
                sys.stderr = old_stderr
                
        except Exception as e:
            return {
                'success': False,
                'output': '',
                'error': str(e)
            }
    
    def _execute_bash(self, code: str, session_id: str) -> Dict:
        """Execute bash/shell code"""
        try:
            # Create temporary script file
            with tempfile.NamedTemporaryFile(mode='w', suffix='.sh', delete=False) as f:
                f.write(code)
                script_path = f.name
            
            try:
                # Execute script
                result = subprocess.run(
                    ['bash', script_path],
                    capture_output=True,
                    text=True,
                    cwd=self.work_dir,
                    timeout=30
                )
                
                return {
                    'success': result.returncode == 0,
                    'output': result.stdout,
                    'error': result.stderr if result.stderr else None
                }
                
            finally:
                # Clean up temporary file
                os.unlink(script_path)
                
        except subprocess.TimeoutExpired:
            return {
                'success': False,
                'output': '',
                'error': 'Code execution timed out (30 seconds)'
            }
        except Exception as e:
            return {
                'success': False,
                'output': '',
                'error': str(e)
            }
    
    def _execute_javascript(self, code: str, session_id: str) -> Dict:
        """Execute JavaScript/Node.js code"""
        if not self._command_exists('node'):
            return {
                'success': False,
                'output': '',
                'error': 'Node.js not found. Please install Node.js to execute JavaScript code.'
            }
        
        try:
            # Create temporary script file
            with tempfile.NamedTemporaryFile(mode='w', suffix='.js', delete=False) as f:
                f.write(code)
                script_path = f.name
            
            try:
                # Execute script
                result = subprocess.run(
                    ['node', script_path],
                    capture_output=True,
                    text=True,
                    cwd=self.work_dir,
                    timeout=30
                )
                
                return {
                    'success': result.returncode == 0,
                    'output': result.stdout,
                    'error': result.stderr if result.stderr else None
                }
                
            finally:
                # Clean up temporary file
                os.unlink(script_path)
                
        except subprocess.TimeoutExpired:
            return {
                'success': False,
                'output': '',
                'error': 'Code execution timed out (30 seconds)'
            }
        except Exception as e:
            return {
                'success': False,
                'output': '',
                'error': str(e)
            }
    
    def _execute_sql(self, code: str, session_id: str) -> Dict:
        """Execute SQL code (using sqlite for demo)"""
        try:
            import sqlite3
            
            # Get or create session database
            if session_id not in self.sessions:
                self.sessions[session_id] = {'db_path': tempfile.mktemp(suffix='.db')}
            
            db_path = self.sessions[session_id]['db_path']
            
            with sqlite3.connect(db_path) as conn:
                cursor = conn.cursor()
                
                # Execute SQL
                results = []
                for statement in code.split(';'):
                    statement = statement.strip()
                    if statement:
                        cursor.execute(statement)
                        
                        # Fetch results if it's a SELECT
                        if statement.upper().startswith('SELECT'):
                            rows = cursor.fetchall()
                            cols = [desc[0] for desc in cursor.description]
                            results.append({'columns': cols, 'rows': rows})
                
                conn.commit()
                
                # Format output
                output = ""
                for result in results:
                    if result['rows']:
                        output += f"Columns: {', '.join(result['columns'])}\n"
                        for row in result['rows']:
                            output += f"{row}\n"
                        output += "\n"
                
                return {
                    'success': True,
                    'output': output.strip(),
                    'error': None
                }
                
        except Exception as e:
            return {
                'success': False,
                'output': '',
                'error': str(e)
            }
    
    def _command_exists(self, command: str) -> bool:
        """Check if a command exists in PATH"""
        return subprocess.run(
            ['which', command], 
            capture_output=True, 
            text=True
        ).returncode == 0
    
    def clear_session(self, session_id: str):
        """Clear a code execution session"""
        if session_id in self.sessions:
            session = self.sessions[session_id]
            
            # Clean up any temporary files
            if 'db_path' in session:
                try:
                    os.unlink(session['db_path'])
                except:
                    pass
            
            del self.sessions[session_id]
    
    def list_sessions(self) -> List[str]:
        """List active sessions"""
        return list(self.sessions.keys())
    
    def get_supported_languages(self) -> List[str]:
        """Get list of supported languages"""
        return list(self.supported_languages.keys())