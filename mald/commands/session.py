"""
MALD session command - Start MALD interactive session
"""

import os
import subprocess
import logging
from pathlib import Path


logger = logging.getLogger(__name__)


def handle(args):
    """Handle the session command"""
    logger.info("Starting MALD interactive session...")
    
    mald_home = Path.home() / '.mald'
    
    if not mald_home.exists():
        logger.error("MALD environment not initialized. Run 'mald init' first.")
        return 1
    
    try:
        # Prepare session environment
        session_name = "mald-session"
        tmux_config = mald_home / 'config' / 'tmux.conf'
        
        # Set environment variables
        env = os.environ.copy()
        env['MALD_HOME'] = str(mald_home)
        env['MALD_SESSION'] = session_name
        
        if args.kb:
            kb_path = mald_home / 'kb' / args.kb
            if not kb_path.exists():
                logger.error(f"Knowledge base '{args.kb}' not found")
                return 1
            env['MALD_KB'] = str(kb_path)
            env['MALD_KB_NAME'] = args.kb
        
        if args.tmux:
            return _start_tmux_session(session_name, tmux_config, env)
        else:
            return _start_shell_session(env)
            
    except Exception as e:
        logger.error(f"Failed to start session: {e}")
        return 1


def _start_tmux_session(session_name, tmux_config, env):
    """Start a tmux session"""
    logger.info(f"Starting tmux session: {session_name}")
    
    # Check if tmux is available
    if not _command_exists('tmux'):
        logger.error("tmux not found. Install tmux or run without --tmux flag.")
        return 1
    
    # Kill existing session if it exists
    subprocess.run(['tmux', 'kill-session', '-t', session_name], 
                  capture_output=True, env=env)
    
    # Start new session
    cmd = [
        'tmux', 'new-session', '-d', '-s', session_name,
        '-c', env.get('MALD_HOME', str(Path.home() / '.mald'))
    ]
    
    if tmux_config.exists():
        cmd.extend(['-f', str(tmux_config)])
    
    result = subprocess.run(cmd, env=env)
    
    if result.returncode == 0:
        # Setup session windows
        _setup_tmux_windows(session_name, env)
        
        # Attach to session
        subprocess.run(['tmux', 'attach-session', '-t', session_name], env=env)
        return 0
    else:
        logger.error("Failed to start tmux session")
        return 1


def _setup_tmux_windows(session_name, env):
    """Setup tmux windows for MALD workflow"""
    
    # Window 1: Editor (neovim)
    subprocess.run([
        'tmux', 'new-window', '-t', session_name, '-n', 'editor',
        'nvim'
    ], env=env)
    
    # Window 2: Terminal
    subprocess.run([
        'tmux', 'new-window', '-t', session_name, '-n', 'terminal'
    ], env=env)
    
    # Window 3: AI (if available)
    subprocess.run([
        'tmux', 'new-window', '-t', session_name, '-n', 'ai',
        'echo "AI workspace - run mald ai chat to start"'
    ], env=env)
    
    # Select first window
    subprocess.run([
        'tmux', 'select-window', '-t', f'{session_name}:1'
    ], env=env)


def _start_shell_session(env):
    """Start a shell session with MALD environment"""
    logger.info("Starting MALD shell session...")
    
    # Print welcome message
    print("\n" + "="*60)
    print("  Welcome to MALD - Markdown Archive Linux Distribution")
    print("="*60)
    
    if 'MALD_KB_NAME' in env:
        print(f"  Knowledge Base: {env['MALD_KB_NAME']}")
    
    print("  Commands:")
    print("    mald kb list      - List knowledge bases")
    print("    mald kb open <kb> - Open knowledge base")
    print("    mald ai chat      - Start AI chat")
    print("    mald session --tmux - Start tmux session")
    print("="*60 + "\n")
    
    # Start shell with MALD environment
    shell = env.get('SHELL', '/bin/bash')
    
    try:
        subprocess.run([shell], env=env)
        return 0
    except KeyboardInterrupt:
        print("\nMALD session ended.")
        return 0


def _command_exists(command):
    """Check if a command exists in PATH"""
    return subprocess.run(
        ['which', command], 
        capture_output=True, 
        text=True
    ).returncode == 0