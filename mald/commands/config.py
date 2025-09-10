"""
MALD config command - Configuration management
"""

import json
import logging
from pathlib import Path


logger = logging.getLogger(__name__)


def handle(args):
    """Handle the config command"""
    if not args.config_action:
        logger.error("No config action specified")
        return 1
    
    if args.config_action == 'get':
        return _get_config(args.key)
    elif args.config_action == 'set':
        return _set_config(args.key, args.value)
    else:
        logger.error(f"Unknown config action: {args.config_action}")
        return 1


def _get_config(key):
    """Get configuration value"""
    config = _load_config()
    
    if not config:
        logger.error("No configuration found. Run 'mald init' first.")
        return 1
    
    # Navigate nested keys with dot notation
    keys = key.split('.')
    value = config
    
    try:
        for k in keys:
            value = value[k]
        
        print(f"{key} = {value}")
        return 0
        
    except KeyError:
        logger.error(f"Configuration key '{key}' not found")
        return 1


def _set_config(key, value):
    """Set configuration value"""
    config = _load_config()
    
    if not config:
        logger.error("No configuration found. Run 'mald init' first.")
        return 1
    
    # Navigate nested keys with dot notation
    keys = key.split('.')
    target = config
    
    # Navigate to parent
    for k in keys[:-1]:
        if k not in target:
            target[k] = {}
        target = target[k]
    
    # Set the value
    final_key = keys[-1]
    
    # Try to parse as JSON for complex values
    try:
        parsed_value = json.loads(value)
        target[final_key] = parsed_value
    except json.JSONDecodeError:
        # Store as string if not valid JSON
        target[final_key] = value
    
    # Save config
    if _save_config(config):
        logger.info(f"Set {key} = {value}")
        return 0
    else:
        logger.error("Failed to save configuration")
        return 1


def _load_config():
    """Load MALD configuration"""
    mald_home = Path.home() / '.mald'
    config_file = mald_home / 'config' / 'config.json'
    
    if not config_file.exists():
        return None
    
    try:
        with open(config_file) as f:
            return json.load(f)
    except Exception as e:
        logger.error(f"Failed to load configuration: {e}")
        return None


def _save_config(config):
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