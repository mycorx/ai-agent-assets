# Contributing

Thanks for your interest in improving this marketplace. This is a small,
solo-maintained project, so the workflow is deliberately lightweight.

## Workflow

1. **Open an issue first** for anything non-trivial — a bug, a feature idea,
   or a behavior change. It saves you from building something that won't be
   merged. Typo fixes and other small changes can skip straight to a PR.
2. **Fork** the repo and branch off `main` (e.g. `fix/…`, `feat/…`, `docs/…`).
3. **Commit** using [Conventional Commits](https://www.conventionalcommits.org/)
   (`feat:`, `fix:`, `docs:`, `chore:`, …) with clear, present-tense messages.
4. **Open a PR** against `main`. Keep it focused — one logical change per PR —
   and describe what changed and why.

The maintainer reviews and merges all PRs.

## Adding a new plugin

1. Create `plugins/<name>/` with its own `.claude-plugin/plugin.json`
   (`name`, `version` starting at `0.1.0`, `description`, `author`, `license`)
   and the plugin's content (`skills/`, `commands/`, `hooks/`, …).
2. Add an entry to `.claude-plugin/marketplace.json`'s `plugins` array:
   `{"name": "<name>", "source": "./plugins/<name>", "description": "…"}`.
3. Add a row to the plugin table in `README.md`.
4. Run `python3 scripts/validate_marketplace.py` before pushing.

A plugin with its own release cadence can add a `CHANGELOG.md` under
`plugins/<name>/` once the root `CHANGELOG.md` gets too crowded to track it —
not needed while the marketplace is small.

## Validation

A GitHub Actions workflow (`.github/workflows/validate.yml`) runs on every PR
and on pushes to `main`. It runs `scripts/validate_marketplace.py`, which
checks that:

- `.claude-plugin/marketplace.json` parses as JSON and carries `name`,
  `owner`, and a non-empty `plugins` array.
- Each plugin's `source` resolves to a local directory with a
  `.claude-plugin/plugin.json` carrying `name`, `description`, and an
  `x.y.z` semver `version`.
- `commands/*.md` have a `description` and `skills/*/SKILL.md` have `name`
  and `description` in their frontmatter, with the skill directory name
  matching the frontmatter `name`.
- Relative links in `README.md` files resolve.

The script is stdlib + PyYAML only. Run it locally before pushing:

```
python3 -m pip install pyyaml
python3 scripts/validate_marketplace.py
```

## Ground rules

- Match the existing style and structure of the code you're touching.
- Update the relevant README and `CHANGELOG.md` when your change is user-facing.
- Be respectful and constructive — assume good faith on all sides.
