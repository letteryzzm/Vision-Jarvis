# AGENTS.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

Vision-Jarvis (视觉驱动的AI秘书) is a vision-driven AI secretary project.

**Status**: Early development - project structure not yet established.

## Development Setup

This is a Python-based project. Check for dependency management files when they are added:
- `requirements.txt`, `pyproject.toml`, `poetry.lock`, or `Pipfile` for dependency installation
- Look for virtual environment setup instructions in README.md

## Project Structure

The codebase structure has not yet been established. When analyzing the architecture:
- Check for main application entry points (e.g., `main.py`, `app.py`, `__main__.py`)
- Identify if this follows a standard Python package structure with `src/` or module directories
- Look for configuration files that indicate framework usage (Django, Flask, FastAPI, etc.)

## Common Commands

⚠️ Build, test, and lint commands have not yet been defined. Check:
- `Makefile` for defined tasks
- `pyproject.toml` for tool configurations (pytest, ruff, mypy, etc.)
- README.md for documented commands
- Scripts in a `scripts/` directory

## Architecture Notes

The architecture will be documented here once the codebase develops. Key areas to document:
- Core AI/vision processing pipeline
- Integration with vision models or APIs
- Data flow between components
- Configuration management approach
