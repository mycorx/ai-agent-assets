# Changelog

All notable changes to this repo's plugins are documented here. Format loosely
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Changed

- Split the repo into three top-level categories — `claude-plugins/` (renamed
  from `plugins/`), `mcp-servers/`, and `agent-skills/` — so it can hold MCP
  servers and skills for other agents alongside Claude Code plugins, not just
  Claude Code plugins.

### Added

- Marketplace scaffold: `.claude-plugin/marketplace.json`, per-plugin
  manifests under `plugins/<name>/`, CI validation
  (`scripts/validate_marketplace.py`), and gitleaks scanning.
- `git-rewrite-email` plugin (migrated from the standalone
  `skills/git-rewrite-history-email` skill).
