# Changelog

All notable changes to `strobengine` will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-08-02

### Bug Fixes

- *(progress)* Downgrade expected HTTP errors from warn to debug
- *(cli)* Move logging flags to subcommands for natural syntax
- *(engine)* Resolve null options and attribute access bugs (#38)

### Documentation

- Add benchmark methodology, infrastructure setup, and project roadmap
- Add tool versions to benchmark methodology (#29)
- Document verbosity, progress bar, and CLI flags
- Update roadmap to reflect changes (#34)
- Add HTTP method, body, and header documentation
- Add code examples (#37)

### Features

- *(reporter)* Add metric descriptions to CLI summary output (#30)
- Add chaos testing engine (#32)
- Add indicatif dependency for progress bar rendering
- Add progress bar module and live render loop
- *(metrics)* Add atomic fields to LiveCounters for active workers and latency tracking
- *(worker)* Track active workers and update live request metrics
- *(config)* Support no_progress flag in TestConfig
- *(engine)* Spawn live progress bar during load test runs
- *(cli)* Add --no-progress option to suppress live progress bar
- Progress indicators (#33)
- Add HTTP request configuration
- Support custom HTTP requests
- *(cli)* Add HTTP request options
- Add support for HTTP methods, request bodies, and custom headers (#35)
- Implement graceful shutdown on SIGINT (#39)

### Miscellaneous Tasks

- *(release)* Prepare v0.1.0 changelog and tag
- Add PyPI publish workflow with tag-based triggering (#26)

### Performance

- Switch global allocator to mimalloc for reduced lock contention (#28)
- Optimize connection pooling with tcp_nodelay, pre-warming, and body consumption (#31)

### Refactoring

- *(engine)* Encapsulate request options into RequestOptions dataclass (#36)

### Testing

- *(config)* Simplify header setup in custom config test
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
