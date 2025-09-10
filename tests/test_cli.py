"""
Test MALD CLI functionality
"""

import pytest
from pathlib import Path
from mald.cli import create_parser
from mald.commands import init, kb
from mald.utils import config_manager


def test_cli_parser():
    """Test CLI argument parser"""
    parser = create_parser()
    
    # Test version
    args = parser.parse_args(['--version'])
    assert args.version
    
    # Test init command
    args = parser.parse_args(['init'])
    assert args.command == 'init'
    assert args.force is False
    
    # Test init with force
    args = parser.parse_args(['init', '--force'])
    assert args.command == 'init'
    assert args.force is True
    
    # Test kb commands
    args = parser.parse_args(['kb', 'create', 'test-kb'])
    assert args.command == 'kb'
    assert args.kb_action == 'create'
    assert args.name == 'test-kb'


def test_init_command(temp_mald_home):
    """Test init command functionality"""
    # Mock args
    class MockArgs:
        force = False
    
    args = MockArgs()
    
    # Run init
    result = init.handle(args)
    assert result == 0
    
    # Check if directories were created
    assert (temp_mald_home / 'kb').exists()
    assert (temp_mald_home / 'config').exists()
    assert (temp_mald_home / 'sessions').exists()
    assert (temp_mald_home / 'cache').exists()
    assert (temp_mald_home / 'ai').exists()
    
    # Check if config was created
    config_file = temp_mald_home / 'config' / 'config.json'
    assert config_file.exists()
    
    # Check if default KB was created
    default_kb = temp_mald_home / 'kb' / 'default'
    assert default_kb.exists()
    assert (default_kb / 'index.md').exists()
    assert (default_kb / 'Welcome to MALD.md').exists()


def test_kb_create_command(temp_mald_home):
    """Test knowledge base creation"""
    # Initialize first
    class MockInitArgs:
        force = False
    
    init.handle(MockInitArgs())
    
    # Create KB
    class MockKBArgs:
        kb_action = 'create'
        name = 'test-project'
    
    args = MockKBArgs()
    result = kb.handle(args)
    assert result == 0
    
    # Check if KB was created
    kb_path = temp_mald_home / 'kb' / 'test-project'
    assert kb_path.exists()
    assert (kb_path / 'index.md').exists()
    assert (kb_path / 'templates').exists()


def test_config_manager(temp_mald_home):
    """Test configuration management"""
    # Initialize config
    config = config_manager.initialize_default_config(temp_mald_home)
    assert config is not None
    assert config['version'] == '0.1.0'
    assert 'mald_home' in config
    
    # Test loading config
    loaded_config = config_manager.load_config()
    assert loaded_config == config
    
    # Test getting config value
    value = config_manager.get_config_value('ai.default_model')
    assert value == 'llama3.2:1b'
    
    # Test setting config value
    success = config_manager.set_config_value('test.value', 'test_data')
    assert success is True
    
    # Verify the value was set
    value = config_manager.get_config_value('test.value')
    assert value == 'test_data'