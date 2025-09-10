#!/usr/bin/env python3
"""
MALD CLI - Main command-line interface for MALD system
"""

import sys
import argparse
import logging
from pathlib import Path

from . import __version__
from .commands import init, kb, session, iso, ai, config


def setup_logging(verbose: bool = False):
    """Setup logging configuration"""
    level = logging.DEBUG if verbose else logging.INFO
    logging.basicConfig(
        level=level,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )


def create_parser():
    """Create the argument parser"""
    parser = argparse.ArgumentParser(
        prog='mald',
        description='MALD - Markdown Archive Linux Distribution CLI'
    )
    
    parser.add_argument(
        '--version', 
        action='version', 
        version=f'mald {__version__}'
    )
    
    parser.add_argument(
        '-v', '--verbose',
        action='store_true',
        help='Enable verbose logging'
    )
    
    # Subcommands
    subparsers = parser.add_subparsers(dest='command', help='Available commands')
    
    # Init command
    init_parser = subparsers.add_parser('init', help='Initialize MALD environment')
    init_parser.add_argument('--force', action='store_true', help='Force re-initialization')
    
    # Knowledge Base commands
    kb_parser = subparsers.add_parser('kb', help='Knowledge base management')
    kb_subparsers = kb_parser.add_subparsers(dest='kb_action')
    
    kb_create = kb_subparsers.add_parser('create', help='Create new knowledge base')
    kb_create.add_argument('name', help='Knowledge base name')
    
    kb_list = kb_subparsers.add_parser('list', help='List knowledge bases')
    
    kb_open = kb_subparsers.add_parser('open', help='Open knowledge base')
    kb_open.add_argument('name', help='Knowledge base name')
    
    # Session command
    session_parser = subparsers.add_parser('session', help='Start MALD interactive session')
    session_parser.add_argument('--kb', help='Knowledge base to open')
    session_parser.add_argument('--tmux', action='store_true', help='Start in tmux session')
    
    # ISO command
    iso_parser = subparsers.add_parser('iso', help='ISO build and management')
    iso_subparsers = iso_parser.add_subparsers(dest='iso_action')
    
    iso_build = iso_subparsers.add_parser('build', help='Build MALD ISO')
    iso_build.add_argument('--output', '-o', help='Output directory')
    
    # AI command
    ai_parser = subparsers.add_parser('ai', help='AI and RAG management')
    ai_subparsers = ai_parser.add_subparsers(dest='ai_action')
    
    ai_setup = ai_subparsers.add_parser('setup', help='Setup local AI models')
    ai_chat = ai_subparsers.add_parser('chat', help='Start AI chat session')
    ai_index = ai_subparsers.add_parser('index', help='Index knowledge base for RAG')
    ai_index.add_argument('kb', help='Knowledge base to index')
    
    # Config command
    config_parser = subparsers.add_parser('config', help='Configuration management')
    config_subparsers = config_parser.add_subparsers(dest='config_action')
    
    config_get = config_subparsers.add_parser('get', help='Get configuration value')
    config_get.add_argument('key', help='Configuration key')
    
    config_set = config_subparsers.add_parser('set', help='Set configuration value')
    config_set.add_argument('key', help='Configuration key')
    config_set.add_argument('value', help='Configuration value')
    
    return parser


def main():
    """Main CLI entry point"""
    parser = create_parser()
    args = parser.parse_args()
    
    setup_logging(args.verbose)
    logger = logging.getLogger(__name__)
    
    if not args.command:
        parser.print_help()
        return 1
    
    try:
        # Route to appropriate command handler
        if args.command == 'init':
            return init.handle(args)
        elif args.command == 'kb':
            return kb.handle(args)
        elif args.command == 'session':
            return session.handle(args)
        elif args.command == 'iso':
            return iso.handle(args)
        elif args.command == 'ai':
            return ai.handle(args)
        elif args.command == 'config':
            return config.handle(args)
        else:
            logger.error(f"Unknown command: {args.command}")
            return 1
            
    except KeyboardInterrupt:
        logger.info("Operation cancelled by user")
        return 1
    except Exception as e:
        logger.error(f"Error: {e}")
        if args.verbose:
            logger.exception("Full traceback:")
        return 1


if __name__ == '__main__':
    sys.exit(main())