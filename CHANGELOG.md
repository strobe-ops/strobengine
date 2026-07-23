# Changelog

All notable changes to `strobengine` will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-07-23

### Bug Fixes

- *(tests)* Fix backward-compat tests to go through main()
- *(docs)* Correct print_summary import and add results display to examples
- *(docs)* Correct print_summary import and add results display to examples (#23)

### Documentation

- *(changelog)* Configure git-cliff and generate initial CHANGELOG.md (#16)
- Add logging flags and tracing crates to README

### Features

- *(config)* Add TestConfig pyclass with Python default arguments (#5)
- *(metrics)* Add TestSummary pyclass and calculate_summary (#6)
- Add StrobEngine class with sync and async interfaces (#11)
- Dynamic load profiling (#17)
- *(cli)* Add -V/--version flag with importlib.metadata
- *(rust)* Add tracing instrumentation and init_logging binding
- *(cli)* Add -v/-q/--log-file flags with stderr logging
- *(logging)* Unify system logging (#20)

### Miscellaneous Tasks

- Add Python environment and build artifacts to .gitignore
- Initialize strobengine hybrid workspace architecture
- *(cargo)* Fix formatting and wrap comment for abi3-py38 feature (#1)
- *(python)* Add __all__ to package init for explicit public API (#8)
- Add GitHub Actions workflow for Rust and Python checks (#10)
- Update minimum Python version to 3.13 for abi3-py313
- *(lint)* Add ruff rules and auto-fix pyupgrade suggestions
- Bump minimum python version to 3.13 and expand ruff rules (#21)
- Add TestPyPI publish workflow with OIDC trusted publishing
- Lower minimum Python version to 3.11 for broader compatibility
- Lower minimum Python version to 3.11 for broader compatibility (#24)
- Add testpypi environment to publish workflow
- Add testpypi environment to publish workflow (#25)
- Add PyPI publish workflow with tag-based triggering

### Refactoring

- Clean up __init__.py public API exports (#14)
- *(cli)* Migrate CLI from argparse to typer (#18)
