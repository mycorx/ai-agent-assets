<!-- Keep PRs focused: one logical change each. -->

## What & why

<!-- What does this change, and what problem does it solve? Link any related issue. -->

## Checklist

- [ ] Commits follow [Conventional Commits](https://www.conventionalcommits.org/)
- [ ] `README.md` updated if the change is user-facing
- [ ] Claude Code plugin change: `python3 scripts/validate_marketplace.py` passes
- [ ] MCP server change: `cargo test`, `cargo fmt --check` and
      `cargo clippy --all-targets -- -D warnings` pass in that server's directory
