# open-meteo-mcp: a standalone weather server

## Goal

An MCP server that answers "what's the weather" — current conditions
and a short forecast, by place name or coordinates — from
[Open-Meteo](https://open-meteo.com). Agent-agnostic and stdio-based,
so any MCP client can use it, and shipped as a self-contained binary so
it also satisfies the strictest local-bundle validation in use (see
[Packaging](#packaging)).

It exists because a voice assistant cannot answer live-data questions
from a model, and the alternatives are worse: a second LLM has no more
access to today's weather than the first, and a web search returns
prose scraped off a page that may be about the wrong suburb and hours
stale, where a weather API returns structured current data in one call.

## Why this is a separate repo, not part of UIA

This is the load-bearing decision, so it is recorded rather than
assumed.

Open-Meteo's free tier is **non-commercial only** (600 req/min, 5,000/hr,
10,000/day; data under CC-BY 4.0). That restriction governs *consumption
of the API*, not distribution of code that can call it. Publishing a
client makes no requests. The person who installs and runs it is the API
consumer, and they choose their own tier.

Shipping this inside UIA would collapse that distinction in a way that
matters: every UIA user would land on the non-commercial tier by
default, so a commercial user would be in breach without being told.
Kept separate, the choice to install is the same moment as the choice to
accept the terms.

Two consequences that follow directly:

- **MycorX must not host a public instance.** Doing so would make
  MycorX the API consumer, at scale, commercially. This server is
  install-and-run-locally; there is no hosted offering.
- **The README must state the non-commercial default plainly**, next to
  the configuration that switches to a paid tier. Not a legal
  requirement — a matter of not setting users up to fail silently.

The licence split falls out of the same reasoning: this repo is Apache
2.0, which will get the server far wider use than UIA's PolyForm dual
licence would, and makes "not part of UIA" structurally true rather
than merely asserted.

## Non-goals

- **Bundling with, or hosting for, UIA.** See above.
- **Routes, places, directions.** A sibling server built on
  OpenRouteService could cover those on a free key; it is tracked in
  UIA's backlog and is not this server.
- **Open-Meteo's other endpoints** — historical archive, marine, air
  quality, flood, climate projections, ensemble. All real, none asked
  for. Each is a new tool the model must choose between.
- **Caching.** Weather is asked because it is current, and Open-Meteo's
  terms restrict what may be done with the data.
- **Severe-weather alerts or anything push-shaped.** This server
  answers questions; it does not originate messages.
- **A geocoding tool of its own.** Geocoding happens *inside*
  `get_weather` when it is given a name. Exposing it separately would
  add a second routing decision to serve a case nobody asked for.

## The tool

One tool, `get_weather`:

```json
{
  "type": "object",
  "properties": {
    "location":  { "type": "string",  "description": "Place name, e.g. \"Melbourne\" or \"Melbourne, Australia\"." },
    "latitude":  { "type": "number",  "description": "Decimal degrees. Use with longitude instead of location." },
    "longitude": { "type": "number",  "description": "Decimal degrees. Use with latitude instead of location." },
    "days":      { "type": "integer", "description": "Days of forecast, 1-7. Default 3." }
  }
}
```

Resolution order, in full, because ambiguity here becomes a wrong
forecast rather than an error:

1. `latitude` **and** `longitude` present — used, and `location` is
   ignored if also given. Coordinates are unambiguous; a name is not.
2. `location` alone — geocoded (below).
3. Neither — the configured default location, if one is set.
4. Nothing set — an **error result**, not a guess. See
   [The default is unset on purpose](#the-default-is-unset-on-purpose).

One of `latitude`/`longitude` without the other is an error result
naming the missing one. `days` outside 1–7 is clamped rather than
rejected, and the clamp is stated in the result, since the model
guessing `10` should still get a useful answer.

### Geocoding

`https://geocoding-api.open-meteo.com/v1/search` — same provider, no
API key for non-commercial use, and it returns latitude, longitude,
country, admin region **and IANA timezone** in one call, which is
exactly the set needed to both fetch and label a forecast.

A name that matches several places resolves to the first result, and
**the result says which place it picked** — "Melbourne, Victoria,
Australia" — so a user who meant Melbourne, Florida hears that it went
wrong instead of silently receiving the wrong forecast. A name that
matches nothing is an error result quoting the input.

## Output: prose, capped, written to be spoken

The consumer is a speech-to-speech model that must *say* this. Raw JSON
invites it to read field names aloud, and 24 hourly rows produce a
rambling answer that costs audio tokens and tells the user less than
two sentences would. So the result is short prose:

```
Melbourne, Victoria, Australia — currently 14°C, overcast, wind 19 km/h.
Today 11-17°C, rain 4mm. Tomorrow 9-16°C, showers. Sunday 10-19°C, clear.
Data by Open-Meteo.com, CC-BY 4.0.
```

Hourly data is not returned at all by default. The attribution line is
part of the result rather than documentation: CC-BY travels with the
data into whatever the end user's agent says next, and the only place
this server can guarantee it appears is the payload it hands over.

## The default is unset on purpose

The location fallback chain ends here, and the last tier ships **empty**.

A configured country is not a location. "Australia" resolves to a
centroid somewhere near the middle of the continent and reports desert
conditions, confidently, to someone in Melbourne — a silent wrong
answer, which is worse than an honest question. So with nothing
configured, `get_weather` with no arguments returns:

```
No default location is configured. Ask the user which city or place
they want the weather for, then call get_weather again with it.
```

That is an **error result** (`is_error: true`), never a transport
error. The tool ran correctly; the model simply lacks an argument, and
an error-flagged result is the thing it can read and act on. A
`ToolError` would surface to the user as a spoken failure instead.

Callers that *do* want a default set it explicitly, to a place name or
coordinates, and get exactly what they asked for.

## Configuration

Environment variables, because that is the seam that exists: an
`.mcpb` manifest carries an `env` block, and MCP clients generally
allow env on a stdio server. No config file.

| Variable | Default | Purpose |
| --- | --- | --- |
| `OPEN_METEO_DEFAULT_LOCATION` | *unset* | Fallback place name or `lat,lon`. Unset means ask. |
| `OPEN_METEO_UNITS` | `metric` | `metric` or `imperial`. |
| `OPEN_METEO_BASE_URL` | public forecast API | Commercial-tier escape hatch. |
| `OPEN_METEO_GEOCODING_URL` | public geocoding API | As above. |
| `OPEN_METEO_API_KEY` | *unset* | Sent only when set; required by the paid tier. |

Units are configuration rather than a tool argument on purpose. A
user's preferred units are a property of the user, not of the question,
and making it an argument invites the model to pick — which it will
sometimes get wrong for no benefit. The commercial tier is
API-identical to the free one, so the three URL/key variables are the
whole of what switching to it requires.

## Implementation

Rust, `rmcp` 3.2.0 (latest stable) with the server and stdio-transport
features, producing one statically-linked binary.

Rust is chosen for a specific reason rather than taste: UIA's bundle
validation requires the launched command to be a real executable that
shipped *inside* the bundle, with no interpreter resolved from the host
machine. A Node or Python server can satisfy that only by vendoring an
entire runtime. Rust emits a single file that passes as-is. That the
same binary works for every other MCP client is a bonus, not the
driver.

Note UIA itself pins `rmcp` 3.1.3, one minor behind. These are separate
repos with separate lockfiles and need not match; the newer pin here is
simply the current stable at the time of writing.

## Testing

TDD — each case written as a failing test first.

**Resolution**

- `latitude` + `longitude` are used, and `location` is ignored when all
  three are given.
- `location` alone is geocoded and the coordinates reach the forecast
  call.
- Neither, with a default configured, uses the default.
- Neither, with no default, returns `is_error: true` and text naming
  what the model should ask for.
- `latitude` without `longitude` is an error result naming the missing
  field.
- `days` of `0` and `99` clamp to 1 and 7, and the result says so.

**Geocoding**

- A multi-match name picks the first and the output names the place
  chosen, including its admin region and country.
- A no-match name is an error result quoting the input.

**Output**

- Every successful result carries the CC-BY attribution line.
- No hourly rows appear in the output.
- `OPEN_METEO_UNITS=imperial` changes the units in both the request and
  the rendered text.

**Failure**

- A non-200 response, a timeout, and malformed JSON each produce an
  error result rather than a panic or a `ToolError`.
- `OPEN_METEO_API_KEY` is sent when set and absent when not.

Every test runs against a mocked HTTP server. **No test touches the
live Open-Meteo API** — CI must not depend on a third party's
availability, and hammering a free non-commercial endpoint from CI is
precisely the behaviour their terms discourage.

## Packaging

Two artifacts, from one build:

- **A plain binary** per platform, for any MCP client configured with a
  command path.
- **A `.mcpb` bundle** per platform, for clients that install bundles:

```json
{
  "name": "open-meteo-mcp",
  "version": "0.1.0",
  "server": {
    "type": "binary",
    "entry_point": "bin/open-meteo-mcp",
    "mcp_config": {
      "command": "${__dirname}/bin/open-meteo-mcp",
      "platform_overrides": {
        "win32": { "command": "${__dirname}/bin/open-meteo-mcp.exe" }
      }
    }
  }
}
```

`server.type` is `"binary"`, the command lives inside the bundle, it
has no script extension and no shebang — the four checks UIA applies,
satisfied by construction rather than by exemption.

## Repository placement

```
mcp-servers/open-meteo-mcp/
├── README.md          # what it does, how to install, the non-commercial default
├── Cargo.toml
├── src/
└── manifest.json      # .mcpb template
```

Per [`mcp-servers/README.md`](../../../mcp-servers/README.md), adding a
server also means adding a row to the table in the root `README.md` —
which currently reads "Nothing here yet." This would be the category's
first entry.

## A prerequisite, handled separately

`ai-agent-assets` had **no `.gitignore`**, which was survivable while
every tracked file was markdown, yaml or a Python script — and is not,
once a Rust crate under `mcp-servers/` starts producing `target/`.

Fixed on its own branch (`chore/add-gitignore`) rather than here, so
this spec stays a spec and the ignore rules can land ahead of any code.
That branch also covers Python bytecode, `node_modules/`, local
worktrees, editor and agent state, and a secrets block in front of
gitleaks. Two choices in it matter to this server: `target/` is
unanchored, because there is no Cargo workspace and each server builds
its own, and `Cargo.lock` is deliberately **not** ignored, since this
crate produces a binary and the committed lockfile is what makes its
build reproducible.
