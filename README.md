# ai-agent-assets

A growing collection of assets for AI-agent-assisted development: Claude Code
plugins and MCP servers today, with room for skills for other agents as they
show up.

## Claude Code plugins

Install from within Claude Code:

```
/plugin marketplace add mycorx/ai-agent-assets
/plugin install git-rewrite-email@ai-agent-assets
```

Run `/plugin marketplace update` to pick up new plugin versions. Plugins are
resolved directly from this repo's `main` branch via
[`.claude-plugin/marketplace.json`](.claude-plugin/marketplace.json) — there's
no separate release branch to fast-forward.

| Plugin | Description |
| --- | --- |
| [`git-rewrite-email`](claude-plugins/git-rewrite-email) | Rewrite Git history to remove an unwanted email, auto-install `filter-repo` if missing, and optionally restore the `origin` remote. |

## MCP servers

Standalone MCP servers, usable by any MCP-capable agent. See
[`mcp-servers/`](mcp-servers) for how a new one gets added.

| Server | Description |
| --- | --- |
| [`open-meteo-mcp`](mcp-servers/open-meteo-mcp) | Current conditions and a short forecast for a place, from Open-Meteo. Free tier is non-commercial only. |

## Skills for other agents

Nothing here yet. See [`agent-skills/`](agent-skills) for how a new one gets added.

## Repo layout

```
.claude-plugin/marketplace.json     # Claude Code marketplace catalog (fixed path, Claude Code requirement)
claude-plugins/<name>/
  .claude-plugin/plugin.json        # that plugin's own manifest
  skills/, commands/, hooks/…       # the plugin's actual content
mcp-servers/<name>/                 # standalone MCP servers, agent-agnostic
agent-skills/<agent>/<name>/        # skills for agents other than Claude Code
scripts/validate_marketplace.py     # the plugin-category validator CI runs
docs/superpowers/                   # design specs and implementation plans
.github/workflows/                  # validate + mcp-servers + gitleaks
```

Each category is self-contained and versioned independently. See
[CONTRIBUTING.md](CONTRIBUTING.md) for how to add to any of them.

## Validation

`scripts/validate_marketplace.py` checks that `marketplace.json` and every
cataloged Claude Code plugin's manifest, skill/command frontmatter, and README
links are well-formed. [`validate.yaml`](.github/workflows/validate.yaml) runs
it on every PR and on pushes to `main`; run it locally with:

```
python3 scripts/validate_marketplace.py
```

It covers the Claude Code plugin category only — `agent-skills/` has no shared
manifest format to check, and MCP servers have their own job:
[`mcp-servers.yaml`](.github/workflows/mcp-servers.yaml) runs `cargo fmt
--check`, `cargo clippy -D warnings` and `cargo test` for every server under
`mcp-servers/` with a `Cargo.toml`, on PRs and pushes to `main` that touch
that path. [`gitleaks.yaml`](.github/workflows/gitleaks.yaml) scans every push
and PR for secrets.

## License

[Apache-2.0](LICENSE)
