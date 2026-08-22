# CLAUDE.md — ai-agent-assets

Project instructions for agentic coding in this repository. This repo is a
**Claude Code plugin marketplace**: a `.claude-plugin/marketplace.json` at the
root cataloging plugins that live locally under `plugins/<name>/`, each with
its own `.claude-plugin/plugin.json`.

## Distribution model

Unlike a single-plugin repo distributed through a separate external catalog,
this repo *is* the marketplace. `/plugin marketplace add mycorx/ai-agent-assets`
clones the whole repo and resolves each plugin's `./plugins/<name>` source
directly — there is **no `release` branch, no fast-forward step, and no
version-bump gate before something ships**. Merging a plugin change to `main`
is what makes it installable. Keep that in mind: `main` must always be in a
state you're comfortable users installing from.

## Adding or changing a plugin

See [CONTRIBUTING.md](CONTRIBUTING.md) for the mechanical steps. In short:
each plugin is self-contained under `plugins/<name>/`, versioned independently
in its own `plugin.json`, and listed once in `.claude-plugin/marketplace.json`.
Run `python3 scripts/validate_marketplace.py` before pushing.

## Git & merge conventions

- **Merge strategy:** squash merge for pull requests by default.
- **Branch cleanup:** delete branches after merge.
- Merging is never unilateral: propose the merge and wait for the maintainer's
  OK. Never push directly to `main`.

## Secret scanning

[gitleaks](https://github.com/gitleaks/gitleaks) runs in CI on every push/PR
([`.github/workflows/gitleaks.yml`](.github/workflows/gitleaks.yml)); known
historical findings would be baselined in
[`.gitleaks-baseline.json`](.gitleaks-baseline.json) (currently empty — clean
history) so CI stays green on dead history while still catching anything new.
No local pre-commit hook — dev environments vary, so this is CI-only by design.
