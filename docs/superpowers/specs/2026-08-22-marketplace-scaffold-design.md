# ai-agent-assets: marketplace scaffold design

> **SUPERSEDED — historical record only.** This is what was designed on
> 2026-08-22, not how the repo works now. It shipped, then the repo outgrew
> it. Read the current docs instead: [`README.md`](../../../README.md),
> [`CONTRIBUTING.md`](../../../CONTRIBUTING.md), [`CLAUDE.md`](../../../CLAUDE.md).
>
> What diverged since:
>
> - **`plugins/` is `claude-plugins/`.** The repo now holds three sibling
>   categories — `claude-plugins/`, `mcp-servers/`, `agent-skills/` — where
>   this spec assumed a single plugin directory. Every `plugins/<name>` path
>   and `./plugins/<name>` source below is wrong by that rename.
> - **Workflows are `.yaml`, not `.yml`** — `validate.yaml`, `gitleaks.yaml`.
> - **`dependabot.yml` was never added.** Dependency updates run through
>   Renovate ([`.github/renovate.json5`](../../../.github/renovate.json5)).
> - **`CHANGELOG.md` was never created**, so the CHANGELOG section below
>   describes a file that does not exist. A plugin's `version` in its own
>   `plugin.json` is the only release signal; merging to `main` ships it.
> - **The marketplace `description` changed** — it catalogs plugins only,
>   not "skills and plugins".
>
> Still accurate: the distribution model (no release branch, sources resolved
> off `main`), the `source`-must-be-a-git-marketplace constraint, and what
> `scripts/validate_marketplace.py` checks.


## Goal

Turn `ai-agent-assets` from a loose folder of skills into a working Claude
Code plugin marketplace: a `.claude-plugin/marketplace.json` at the repo root
cataloging one or more plugins that live locally in the same repo, each under
`plugins/<name>/` with its own `.claude-plugin/plugin.json`.

Modeled on `plan-staged-rollout`'s scaffolding (CI, gitleaks, validation
script, CONTRIBUTING/README conventions) but *not* its distribution model.
`plan-staged-rollout` is a single plugin distributed through a separate
external catalog repo, so it needs a `main`/`release` branch split with a
fast-forward-only release workflow to avoid installing half-finished work.
Here the plugins live inside the marketplace repo itself, so
`/plugin marketplace add mycorx/ai-agent-assets` clones the whole repo and
resolves `./plugins/<name>` sources straight off `main` — no release branch,
no fast-forward workflow, no version-bump-gates-shipping risk.

## Layout

```
ai-agent-assets/
├── .claude-plugin/
│   └── marketplace.json
├── plugins/
│   └── git-rewrite-email/
│       ├── .claude-plugin/
│       │   └── plugin.json
│       └── skills/
│           └── git-rewrite-email/
│               └── SKILL.md
├── .github/
│   ├── workflows/
│   │   ├── validate.yml
│   │   └── gitleaks.yml
│   ├── PULL_REQUEST_TEMPLATE.md
│   ├── ISSUE_TEMPLATE/*.yml
│   ├── CODEOWNERS
│   └── dependabot.yml
├── scripts/
│   └── validate_marketplace.py
├── .gitattributes
├── .gitleaks.toml
├── .gitleaks-baseline.json
├── README.md
├── CONTRIBUTING.md
├── CLAUDE.md
├── CHANGELOG.md
└── LICENSE
```

## marketplace.json

```json
{
  "name": "ai-agent-assets",
  "description": "Collection of Claude Code skills and plugins.",
  "owner": { "name": "mycorx" },
  "plugins": [
    {
      "name": "git-rewrite-email",
      "source": "./plugins/git-rewrite-email",
      "description": "Rewrite Git history to remove an unwanted email, auto-install filter-repo if missing, and optionally restore the origin remote."
    }
  ]
}
```

`source` is a bare relative-path string resolved against the marketplace
root (the directory containing `.claude-plugin/`) — confirmed against current
Claude Code docs. This only resolves correctly when the marketplace is added
as a git source (`owner/repo` or a git URL), not as a direct URL to the JSON
file, so the README's install instructions must use the repo-shorthand form.

## Existing skill cleanup

`skills/git-rewrite-history-email/SKILL.md` has `name: git-rewrite-email` in
its frontmatter — the directory name and the frontmatter name disagree.
Frontmatter is what Claude keys off, so the directory is renamed to match:
`plugins/git-rewrite-email/skills/git-rewrite-email/SKILL.md`. This also
becomes the marketplace's first plugin entry with its own
`.claude-plugin/plugin.json` (name/version/description/author/license,
version starts at `0.1.0`).

## CI / validation

- `gitleaks.yml` ported unchanged (same tool, same purpose, repo-agnostic).
- `validate_plugin.py` → `scripts/validate_marketplace.py`: parses
  `marketplace.json`, resolves each `./plugins/<name>` source, and for each
  one runs the same checks the old script ran for the single manifest —
  `plugin.json` has `name`/`version`(semver)/`description`; `SKILL.md` /
  `commands/*.md` frontmatter has required fields; README relative links
  resolve — just looped over N plugin directories instead of asserting one
  root manifest.
- `validate.yml` workflow unchanged apart from invoking the renamed script.

## CHANGELOG

One root `CHANGELOG.md` with an `[Unreleased]` section, sufficient while
there's a single plugin. `CONTRIBUTING.md` notes a plugin can graduate to its
own `CHANGELOG.md` under `plugins/<name>/` if the collection grows and
changes need per-plugin granularity — not built now (YAGNI).

## Out of scope

- No release/main branch split, no release-prepare/release-publish workflows.
- No second plugin — the scaffold ships with the one existing skill migrated
  in; future skills/plugins follow the same `plugins/<name>/` pattern
  documented in CONTRIBUTING.md.
