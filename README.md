# ai-agent-assets

A Claude Code plugin marketplace — a growing collection of skills and plugins
for AI-agent-assisted development.

## Install

From within Claude Code:

```
/plugin marketplace add mycorx/ai-agent-assets
/plugin install git-rewrite-email@ai-agent-assets
```

Run `/plugin marketplace update` to pick up new plugin versions. Plugins are
resolved directly from this repo's `main` branch — there's no separate
release branch to fast-forward.

## Plugins

| Plugin | Description |
| --- | --- |
| [`git-rewrite-email`](plugins/git-rewrite-email) | Rewrite Git history to remove an unwanted email, auto-install `filter-repo` if missing, and optionally restore the `origin` remote. |

## Repo layout

```
.claude-plugin/marketplace.json   # catalogs every plugin below
plugins/<name>/
  .claude-plugin/plugin.json      # that plugin's own manifest
  skills/, commands/, hooks/…     # the plugin's actual content
```

Each plugin is self-contained under `plugins/<name>/` and versioned
independently via its own `plugin.json`. See [CONTRIBUTING.md](CONTRIBUTING.md)
for how to add a new one.

## Validation

`scripts/validate_marketplace.py` checks that `marketplace.json` and every
cataloged plugin's manifest, skill/command frontmatter, and README links are
well-formed. It runs in CI on every push and PR; run it locally with:

```
python3 scripts/validate_marketplace.py
```

## License

[Apache-2.0](LICENSE)
