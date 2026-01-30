"""
MALD config command - Configuration management
"""

import json
import logging

from ..utils import config_manager

logger = logging.getLogger(__name__)


def handle(args):
    """Handle the config command"""
    if not args.config_action:
        logger.error("No config action specified")
        return 1

    if args.config_action == "get":
        return _get_config(args.key)
    elif args.config_action == "set":
        return _set_config(args.key, args.value)
    else:
        logger.error(f"Unknown config action: {args.config_action}")
        return 1


def _get_config(key):
    """Get configuration value"""
    value = config_manager.get_config_value(key)

    if value is None:
        logger.error(f"Configuration key '{key}' not found")
        return 1

    print(f"{key} = {value}")
    return 0


def _set_config(key, value):
    """Set configuration value"""
    # Try to parse as JSON for complex values
    try:
        parsed_value = json.loads(value)
    except json.JSONDecodeError:
        parsed_value = value

    if config_manager.set_config_value(key, parsed_value):
        logger.info(f"Set {key} = {value}")
        return 0
    else:
        logger.error("Failed to save configuration")
        return 1
