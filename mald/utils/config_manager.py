"""
MALD configuration manager
"""

import json
import logging
from pathlib import Path


logger = logging.getLogger(__name__)


def initialize_default_config(mald_home):
    """Initialize default MALD configuration"""
    config_dir = mald_home / 'config'
    config_dir.mkdir(parents=True, exist_ok=True)  # Ensure config dir exists
    config_file = config_dir / 'config.json'
    
    default_config = {
        "version": "0.1.0",
        "mald_home": str(mald_home),
        "editor": {
            "default": "nvim",
            "markdown_mode": True,
            "line_numbers": True,
            "word_wrap": True
        },
        "terminal": {
            "default_shell": "/bin/bash",
            "tmux_enabled": True,
            "tmux_config": str(config_dir / 'tmux.conf')
        },
        "pkm": {
            "default_kb": "default",
            "link_style": "wikilink",
            "auto_save": True,
            "backup_enabled": True
        },
        "ai": {
            "provider": "ollama",
            "default_model": "llama3.2:1b",
            "embedding_model": "nomic-embed-text",
            "rag_enabled": True,
            "privacy_mode": True
        },
        "security": {
            "luks_enabled": False,
            "backup_encryption": True,
            "secure_delete": True
        },
        "snapshots": {
            "enabled": True,
            "frequency": "daily",
            "retention_days": 30,
            "auto_cleanup": True
        },
        "iso": {
            "arch_variant": "minimal",
            "include_ai": True,
            "compression": "xz"
        },
        "wsl": {
            "enabled": False,
            "integration_mode": "hybrid"
        }
    }
    
    with open(config_file, 'w') as f:
        json.dump(default_config, f, indent=2)
    
    logger.info(f"Created default configuration: {config_file}")
    return default_config


def load_config():
    """Load MALD configuration"""
    mald_home = Path.home() / '.mald'
    config_file = mald_home / 'config' / 'config.json'
    
    if not config_file.exists():
        logger.warning("Configuration file not found, using defaults")
        return initialize_default_config(mald_home)
    
    try:
        with open(config_file) as f:
            return json.load(f)
    except Exception as e:
        logger.error(f"Failed to load configuration: {e}")
        return None


def save_config(config):
    """Save MALD configuration"""
    mald_home = Path.home() / '.mald'
    config_file = mald_home / 'config' / 'config.json'
    
    try:
        with open(config_file, 'w') as f:
            json.dump(config, f, indent=2)
        return True
    except Exception as e:
        logger.error(f"Failed to save configuration: {e}")
        return False


def get_config_value(key, default=None):
    """Get a configuration value by key"""
    config = load_config()
    if not config:
        return default
    
    keys = key.split('.')
    value = config
    
    try:
        for k in keys:
            value = value[k]
        return value
    except KeyError:
        return default


def set_config_value(key, value):
    """Set a configuration value by key"""
    config = load_config()
    if not config:
        return False
    
    keys = key.split('.')
    target = config
    
    # Navigate to parent
    for k in keys[:-1]:
        if k not in target:
            target[k] = {}
        target = target[k]
    
    # Set the value
    target[keys[-1]] = value
    
    return save_config(config)