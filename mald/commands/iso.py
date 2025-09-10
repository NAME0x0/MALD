"""
MALD ISO command - Build and manage MALD ISO
"""

import logging
import subprocess
from pathlib import Path


logger = logging.getLogger(__name__)


def handle(args):
    """Handle the iso command"""
    if not args.iso_action:
        logger.error("No ISO action specified")
        return 1
    
    if args.iso_action == 'build':
        return _build_iso(args)
    else:
        logger.error(f"Unknown ISO action: {args.iso_action}")
        return 1


def _build_iso(args):
    """Build MALD ISO"""
    logger.info("Building MALD ISO...")
    
    # Get the project root
    project_root = Path(__file__).parent.parent.parent
    iso_dir = project_root / 'iso'
    
    if not iso_dir.exists():
        logger.error("ISO build directory not found")
        return 1
    
    build_script = iso_dir / 'build.sh'
    
    if not build_script.exists():
        logger.error("ISO build script not found")
        logger.info("Creating basic build script...")
        _create_build_script(build_script)
    
    # Set output directory
    output_dir = Path(args.output) if args.output else iso_dir / 'output'
    output_dir.mkdir(parents=True, exist_ok=True)
    
    try:
        # Run the build script
        env = {
            'MALD_OUTPUT_DIR': str(output_dir),
            'MALD_ISO_DIR': str(iso_dir),
            **dict(subprocess.os.environ)
        }
        
        result = subprocess.run(
            ['bash', str(build_script)],
            cwd=iso_dir,
            env=env,
            capture_output=False
        )
        
        if result.returncode == 0:
            logger.info(f"ISO build completed successfully")
            logger.info(f"Output directory: {output_dir}")
            return 0
        else:
            logger.error("ISO build failed")
            return 1
            
    except Exception as e:
        logger.error(f"Failed to build ISO: {e}")
        return 1


def _create_build_script(build_script):
    """Create a basic ISO build script"""
    content = """#!/bin/bash
# MALD ISO Build Script
set -e

echo "Building MALD ISO..."
echo "This is a placeholder build script."
echo "Output directory: ${MALD_OUTPUT_DIR}"
echo "ISO directory: ${MALD_ISO_DIR}"

# TODO: Implement actual ISO build process
echo "ISO build script needs to be implemented with:"
echo "- Arch Linux base system"
echo "- MALD packages and configurations"
echo "- s6 init system"
echo "- LUKS encryption setup"
echo "- btrfs filesystem"
echo "- tmux + neovim configuration"
echo "- AI tools (Ollama, llama.cpp)"

echo "Build script placeholder completed."
"""
    
    build_script.write_text(content)
    build_script.chmod(0o755)
    logger.info(f"Created build script: {build_script}")