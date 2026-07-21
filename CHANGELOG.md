# Changelog

All notable changes to `strobengine` will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Features

- *(config)* Add TestConfig pyclass with Python default arguments (#5)
- *(metrics)* Add TestSummary pyclass and calculate_summary (#6)
- Add StrobEngine class with sync and async interfaces (#11)

### Miscellaneous Tasks

- Add Python environment and build artifacts to .gitignore
- Initialize strobengine hybrid workspace architecture
- *(cargo)* Fix formatting and wrap comment for abi3-py38 feature (#1)
- *(python)* Add __all__ to package init for explicit public API (#8)
- Add GitHub Actions workflow for Rust and Python checks (#10)

### Refactoring

- Clean up __init__.py public API exports (#14)
