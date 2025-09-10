# MALD Makefile
.PHONY: help install clean test format lint build iso dev

# Default target
help:
	@echo "MALD - Markdown Archive Linux Distribution"
	@echo "Available targets:"
	@echo "  install    Install MALD CLI"
	@echo "  clean      Clean build artifacts"
	@echo "  test       Run tests"
	@echo "  format     Format code with black"
	@echo "  lint       Run linting checks"
	@echo "  build      Build Python package"
	@echo "  iso        Build bootable ISO"
	@echo "  dev        Setup development environment"

# Install MALD locally
install:
	./scripts/install.sh

# Clean build artifacts
clean:
	rm -rf build/
	rm -rf dist/
	rm -rf *.egg-info/
	find . -type d -name __pycache__ -delete
	find . -type f -name "*.pyc" -delete
	rm -rf iso/output/
	rm -rf iso/work/

# Run tests
test:
	python3 -m pytest tests/ -v

# Format code
format:
	python3 -m black mald/ tests/

# Run linting
lint:
	python3 -m flake8 mald/ tests/
	python3 -m mypy mald/

# Build Python package
build:
	python3 -m build

# Build ISO (requires Arch Linux)
iso:
	@echo "Building MALD ISO..."
	@if [ ! -f /etc/arch-release ]; then \
		echo "Error: ISO building requires Arch Linux"; \
		exit 1; \
	fi
	cd iso && ./build.sh

# Setup development environment
dev:
	python3 -m venv venv
	./venv/bin/pip install -e ".[dev]"
	@echo "Development environment created. Activate with:"
	@echo "source venv/bin/activate"

# Quick test of CLI
quick-test:
	python3 -m mald --version
	python3 -m mald --help

# Initialize test environment
init-test:
	python3 -m mald init --force
	python3 -m mald kb create test-kb
	python3 -m mald kb list

# Package for distribution
package: clean build
	@echo "Package created in dist/"