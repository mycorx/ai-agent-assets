# Changelog

All notable changes to this repo's plugins are documented here. Format loosely
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- Marketplace scaffold: `.claude-plugin/marketplace.json`, per-plugin
  manifests under `plugins/<name>/`, CI validation
  (`scripts/validate_marketplace.py`), and gitleaks scanning.
- `git-rewrite-email` plugin (migrated from the standalone
  `skills/git-rewrite-history-email` skill).
