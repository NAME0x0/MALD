"""
MALD filesystem utilities
"""

import os
import shutil
import logging
from pathlib import Path


logger = logging.getLogger(__name__)


def ensure_directory(path):
    """Ensure directory exists, create if necessary"""
    path = Path(path)
    path.mkdir(parents=True, exist_ok=True)
    return path


def safe_copy(src, dst):
    """Safely copy file or directory"""
    try:
        src = Path(src)
        dst = Path(dst)
        
        if src.is_file():
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src, dst)
        elif src.is_dir():
            shutil.copytree(src, dst, dirs_exist_ok=True)
        
        return True
    except Exception as e:
        logger.error(f"Failed to copy {src} to {dst}: {e}")
        return False


def safe_move(src, dst):
    """Safely move file or directory"""
    try:
        src = Path(src)
        dst = Path(dst)
        
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.move(src, dst)
        return True
    except Exception as e:
        logger.error(f"Failed to move {src} to {dst}: {e}")
        return False


def safe_delete(path, secure=False):
    """Safely delete file or directory"""
    try:
        path = Path(path)
        
        if not path.exists():
            return True
        
        if secure and path.is_file():
            # Secure delete by overwriting with random data
            _secure_delete_file(path)
        
        if path.is_file():
            path.unlink()
        elif path.is_dir():
            shutil.rmtree(path)
        
        return True
    except Exception as e:
        logger.error(f"Failed to delete {path}: {e}")
        return False


def _secure_delete_file(file_path):
    """Securely delete a file by overwriting with random data"""
    try:
        file_size = file_path.stat().st_size
        
        with open(file_path, 'r+b') as f:
            # Overwrite with random data multiple times
            for _ in range(3):
                f.seek(0)
                f.write(os.urandom(file_size))
                f.flush()
                os.fsync(f.fileno())
        
    except Exception as e:
        logger.warning(f"Secure delete failed for {file_path}: {e}")


def find_files(directory, pattern="*", recursive=True):
    """Find files matching pattern"""
    directory = Path(directory)
    
    if recursive:
        return list(directory.rglob(pattern))
    else:
        return list(directory.glob(pattern))


def get_file_info(path):
    """Get file information"""
    path = Path(path)
    
    if not path.exists():
        return None
    
    stat = path.stat()
    
    return {
        'path': str(path),
        'name': path.name,
        'size': stat.st_size,
        'modified': stat.st_mtime,
        'created': stat.st_ctime,
        'is_file': path.is_file(),
        'is_dir': path.is_dir(),
        'permissions': oct(stat.st_mode)[-3:]
    }


def create_backup(source, backup_dir, compress=True):
    """Create backup of file or directory"""
    try:
        source = Path(source)
        backup_dir = Path(backup_dir)
        
        ensure_directory(backup_dir)
        
        timestamp = str(int(time.time()))
        backup_name = f"{source.name}_{timestamp}"
        
        if compress and source.is_dir():
            backup_path = backup_dir / f"{backup_name}.tar.gz"
            _create_compressed_backup(source, backup_path)
        else:
            backup_path = backup_dir / backup_name
            safe_copy(source, backup_path)
        
        return backup_path
        
    except Exception as e:
        logger.error(f"Failed to create backup: {e}")
        return None


def _create_compressed_backup(source, backup_path):
    """Create compressed backup using tar"""
    import tarfile
    
    with tarfile.open(backup_path, 'w:gz') as tar:
        tar.add(source, arcname=source.name)


def cleanup_old_backups(backup_dir, retention_days=30):
    """Cleanup old backups"""
    import time
    
    backup_dir = Path(backup_dir)
    if not backup_dir.exists():
        return
    
    cutoff_time = time.time() - (retention_days * 24 * 60 * 60)
    
    for backup_file in backup_dir.iterdir():
        if backup_file.stat().st_mtime < cutoff_time:
            safe_delete(backup_file)
            logger.info(f"Cleaned up old backup: {backup_file}")


# Import time module at the top level to avoid NameError
import time