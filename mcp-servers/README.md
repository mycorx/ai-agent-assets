# MCP servers

Standalone [MCP](https://modelcontextprotocol.io) servers, usable by any
MCP-capable agent (Claude Code or otherwise) — not tied to the Claude Code
plugin format in [`../claude-plugins/`](../claude-plugins).

## Adding one

1. Create `mcp-servers/<name>/` with the server's source (or a pointer to
   where it lives, if it's not hosted in this repo) and its own `README.md`
   covering what it does and how to run/install it.
2. Add a row to the table in the root [`README.md`](../README.md).

No shared manifest format is imposed here — each server documents its own
install and run instructions. [`open-meteo-mcp`](open-meteo-mcp) is the
worked example: a Rust binary with a `.mcpb` manifest, a packaging script,
and mocked-HTTP tests. A server with a `Cargo.toml` is picked up
automatically by [`mcp-servers.yaml`](../.github/workflows/mcp-servers.yaml),
which runs `cargo fmt --check`, `cargo clippy -D warnings` and `cargo test`
for it; run those locally before pushing. A server in another language needs
its own CI job.
