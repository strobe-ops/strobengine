.PHONY: check fix

# Shortcut for manual pre-commit check
check:
	pre-commit run --hook-stage pre-push --all-files

# Quick command to auto-fix formatting before pushing
fix:
	cargo fmt
	uv run ruff check --fix .
	uv run ruff format .
