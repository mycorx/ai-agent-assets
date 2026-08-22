# Skills for other agents

Skills written for AI agents other than Claude Code (which has its own
category at [`../claude-plugins/`](../claude-plugins)) — e.g. Cursor, Codex,
or any other agent with its own skill/instruction format.

## Adding one

1. Create `agent-skills/<agent>/<name>/` using that agent's own skill or
   instruction-file convention — this repo doesn't impose a shared format
   across agents, since each has its own.
2. Add a row to the table in the root [`README.md`](../README.md).
