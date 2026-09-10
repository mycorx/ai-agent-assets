# open-meteo-mcp

An MCP server that answers "what's the weather" — current conditions and a
short forecast, by place name or coordinates — from
[Open-Meteo](https://open-meteo.com).

Agent-agnostic and stdio-based, so any MCP client can use it, and shipped as
a self-contained binary with no interpreter to install.

## The free tier is non-commercial only

Open-Meteo's free API is for **non-commercial use** (600 requests/minute,
5,000/hour, 10,000/day; data under CC-BY 4.0). Running this server against
the default endpoints puts you on that tier.

If your use is commercial, set `OPEN_METEO_BASE_URL`,
`OPEN_METEO_GEOCODING_URL` and `OPEN_METEO_API_KEY` to your paid-tier
endpoints and key. The paid API is otherwise identical, so those three
variables are the whole of the change.

There is no hosted instance of this server, and there will not be: hosting
one would make the host the API consumer, at scale.

## The tool

One tool, `get_weather`:

| Argument | Type | Meaning |
| --- | --- | --- |
| `location` | string | Place name, e.g. "Melbourne" or "Melbourne, Australia". |
| `latitude` | number | Decimal degrees. Use with `longitude` instead of `location`. |
| `longitude` | number | Decimal degrees. Use with `latitude`. |
| `days` | integer | Days of forecast, 1–7. Default 3; out-of-range values are clamped and the answer says so. |

Coordinates win if both are given. A name alone is geocoded, and the answer
names the place it picked — "Melbourne, Victoria, Australia" — so a wrong
match is audible rather than silent. With no arguments and no configured
default, the tool returns an error result asking for a location rather than
guessing one.

The answer is short prose, written to be spoken:

```
Melbourne, Victoria, Australia — currently 14°C, overcast, wind 19 km/h.
Today 11-17°C, rain 4mm. Tomorrow 9-16°C, showers. Saturday 10-19°C, clear.
Data by Open-Meteo.com, CC-BY 4.0.
```

The attribution line is part of the result, not documentation: CC-BY travels
with the data into whatever the calling agent says next.

## Configuration

Environment variables, set on the server process:

| Variable | Default | Purpose |
| --- | --- | --- |
| `OPEN_METEO_DEFAULT_LOCATION` | *unset* | Fallback place name or `lat,lon`. Unset means the tool asks. |
| `OPEN_METEO_UNITS` | `metric` | `metric` or `imperial`. |
| `OPEN_METEO_BASE_URL` | public forecast API | Commercial-tier escape hatch. |
| `OPEN_METEO_GEOCODING_URL` | public geocoding API | As above. |
| `OPEN_METEO_API_KEY` | *unset* | Sent only when set; required by the paid tier. |

A configured **country** is not a location: "Australia" resolves to a
centroid in the desert and reports desert weather, confidently, to someone
in Melbourne. Set a city or coordinates, or leave it unset.

## Install

**From a bundle.** Download the `.mcpb` for your platform and install it
with any MCP client that accepts bundles.

**From source.**

```bash
cargo build --release
```

then point your client at `target/release/open-meteo-mcp`. For a client
using JSON config:

```json
{
  "mcpServers": {
    "open-meteo": {
      "command": "/path/to/open-meteo-mcp",
      "env": { "OPEN_METEO_DEFAULT_LOCATION": "Melbourne" }
    }
  }
}
```

## Building a bundle

```bash
./scripts/package-mcpb.sh
```

Writes `dist/open-meteo-mcp-<platform>.mcpb`. Each platform's bundle is
built on that platform.

## Development

```bash
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

No test touches the live Open-Meteo API — every HTTP test runs against a
mock server. Please keep it that way: CI should not depend on a third
party's availability, and hammering a free non-commercial endpoint from CI
is what their terms discourage.

## Licence

[Apache-2.0](../../LICENSE). Weather data is © Open-Meteo.com under
[CC-BY 4.0](https://creativecommons.org/licenses/by/4.0/).
