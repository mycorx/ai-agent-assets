# CLAUDE.md — ai-agent-assets

Project instructions for agentic coding in this repository. This repo is a
general collection of AI-agent assets, organized into three top-level
categories:

- `claude-plugins/` — Claude Code plugins, cataloged by the root
  `.claude-plugin/marketplace.json`
- `mcp-servers/` — standalone MCP servers, agent-agnostic
- `agent-skills/` — skills for agents other than Claude Code

Each category is self-contained with its own conventions — see that
category's own `README.md`. Only `claude-plugins/` has a shared manifest
format and CI validation; the other two are freeform.

## Distribution model (Claude Code plugins)

Unlike a single-plugin repo distributed through a separate external catalog,
this repo *is* the marketplace for its `claude-plugins/` category.
`/plugin marketplace add mycorx/ai-agent-assets` clones the whole repo and
resolves each plugin's `./claude-plugins/<name>` source directly — there is
**no `release` branch, no fast-forward step, and no version-bump gate before
something ships**. Merging a plugin change to `main` is what makes it
installable. Keep that in mind: `main` must always be in a state you're
comfortable users installing from.

## Adding or changing an asset

See [CONTRIBUTING.md](CONTRIBUTING.md) for the mechanical steps, split by
category. For Claude Code plugins specifically: each plugin is self-contained
under `claude-plugins/<name>/`, versioned independently in its own
`plugin.json`, and listed once in `.claude-plugin/marketplace.json`. Run
`python3 scripts/validate_marketplace.py` before pushing.

## MCP servers

`mcp-servers/<name>/` holds real, buildable servers — `open-meteo-mcp` is a
Rust binary, and is the worked example for anything added here. CI does not
build or test them, so their gates are yours to run from the server's own
directory before pushing:

```
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

No test may touch a live upstream API — HTTP is mocked (`wiremock`) so CI
never depends on a third party's availability or quota.

## Git & merge conventions

- **Merge strategy:** squash merge for pull requests by default.
- **Branch cleanup:** delete branches after merge.
- Merging is never unilateral: propose the merge and wait for the maintainer's
  OK. Never push directly to `main`.

## Secret scanning

[gitleaks](https://github.com/gitleaks/gitleaks) runs in CI on every push/PR
([`.github/workflows/gitleaks.yaml`](.github/workflows/gitleaks.yaml)); known
historical findings would be baselined in
[`.gitleaks-baseline.json`](.gitleaks-baseline.json) (currently empty — clean
history) so CI stays green on dead history while still catching anything new.
No local pre-commit hook — dev environments vary, so this is CI-only by design.
