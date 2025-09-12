"""
MALD execute command - Execute code blocks in markdown files
"""

import logging
from pathlib import Path
from ..executor import CodeExecutor, CellParser

logger = logging.getLogger(__name__)


def handle(args):
    """Handle the execute command"""
    if not args.exec_action:
        logger.error("No execute action specified")
        return 1
    
    if args.exec_action == 'file':
        return _execute_file(args.file, args.session_id)
    elif args.exec_action == 'cell':
        return _execute_cell(args.file, args.cell_id, args.session_id)
    elif args.exec_action == 'list':
        return _list_cells(args.file)
    elif args.exec_action == 'clear':
        return _clear_session(args.session_id)
    else:
        logger.error(f"Unknown execute action: {args.exec_action}")
        return 1


def _execute_file(file_path_str, session_id="default"):
    """Execute all executable code cells in a file"""
    file_path = Path(file_path_str)
    
    if not file_path.exists():
        logger.error(f"File not found: {file_path}")
        return 1
    
    try:
        # Parse cells
        parser = CellParser()
        executor = CodeExecutor(work_dir=file_path.parent)
        
        cells = parser.extract_executable_cells(file_path)
        
        if not cells:
            logger.info("No executable code cells found in file")
            return 0
        
        logger.info(f"Found {len(cells)} executable code cells")
        
        # Execute cells
        content = file_path.read_text(encoding='utf-8')
        updated_content = content
        
        for i, cell in enumerate(cells, 1):
            print(f"\n--- Executing Cell {i}/{len(cells)} ({cell['language']}) ---")
            print(f"Cell ID: {cell['cell_id']}")
            
            # Execute code
            result = executor.execute_code(
                cell['code'], 
                cell['language'], 
                session_id
            )
            
            print(f"Execution time: {result['execution_time']:.3f}s")
            
            if result['success']:
                print("✓ Success")
                if result['output']:
                    print("Output:")
                    print(result['output'])
            else:
                print("✗ Error")
                if result['error']:
                    print(f"Error: {result['error']}")
            
            # Update content with output
            updated_content = parser.update_cell_output(
                updated_content, 
                cell, 
                result['output'] or '', 
                result['error']
            )
        
        # Write updated content back to file
        file_path.write_text(updated_content, encoding='utf-8')
        logger.info(f"Updated file with execution results: {file_path}")
        
        return 0
        
    except Exception as e:
        logger.error(f"Failed to execute file: {e}")
        return 1


def _execute_cell(file_path_str, cell_id, session_id="default"):
    """Execute a specific code cell"""
    file_path = Path(file_path_str)
    
    if not file_path.exists():
        logger.error(f"File not found: {file_path}")
        return 1
    
    try:
        # Parse cells
        parser = CellParser()
        executor = CodeExecutor(work_dir=file_path.parent)
        
        cell = parser.find_cell_by_id(file_path, cell_id)
        
        if not cell:
            logger.error(f"Cell not found: {cell_id}")
            return 1
        
        if not cell['executable']:
            logger.error(f"Cell {cell_id} is not executable")
            return 1
        
        print(f"Executing cell {cell_id} ({cell['language']})...")
        
        # Execute code
        result = executor.execute_code(
            cell['code'], 
            cell['language'], 
            session_id
        )
        
        print(f"Execution time: {result['execution_time']:.3f}s")
        
        if result['success']:
            print("✓ Success")
            if result['output']:
                print("Output:")
                print(result['output'])
        else:
            print("✗ Error")
            if result['error']:
                print(f"Error: {result['error']}")
        
        # Update file with output
        content = file_path.read_text(encoding='utf-8')
        updated_content = parser.update_cell_output(
            content, 
            cell, 
            result['output'] or '', 
            result['error']
        )
        
        file_path.write_text(updated_content, encoding='utf-8')
        logger.info(f"Updated file with execution results: {file_path}")
        
        return 0
        
    except Exception as e:
        logger.error(f"Failed to execute cell: {e}")
        return 1


def _list_cells(file_path_str):
    """List all code cells in a file"""
    file_path = Path(file_path_str)
    
    if not file_path.exists():
        logger.error(f"File not found: {file_path}")
        return 1
    
    try:
        parser = CellParser()
        cells = parser.parse_file(file_path)
        summary = parser.get_cell_summary(file_path)
        
        print(f"\nCode cells in {file_path.name}:")
        print("=" * 50)
        print(f"Total cells: {summary['total_cells']}")
        print(f"Executable cells: {summary['executable_cells']}")
        print(f"Languages: {', '.join(summary['languages'])}")
        
        if cells:
            print("\nCell details:")
            for i, cell in enumerate(cells, 1):
                status = "✓ Executable" if cell['executable'] else "○ Static"
                print(f"  {i}. {cell['cell_id']} ({cell['language']}) - {status}")
                if cell['attributes']:
                    print(f"     Attributes: {cell['attributes']}")
        
        return 0
        
    except Exception as e:
        logger.error(f"Failed to list cells: {e}")
        return 1


def _clear_session(session_id):
    """Clear a code execution session"""
    try:
        executor = CodeExecutor()
        
        if session_id == "all":
            sessions = executor.list_sessions()
            for sid in sessions:
                executor.clear_session(sid)
            logger.info(f"Cleared {len(sessions)} sessions")
        else:
            executor.clear_session(session_id)
            logger.info(f"Cleared session: {session_id}")
        
        return 0
        
    except Exception as e:
        logger.error(f"Failed to clear session: {e}")
        return 1