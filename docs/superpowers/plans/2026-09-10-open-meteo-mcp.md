# open-meteo-mcp Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship `open-meteo-mcp` — a standalone, stdio MCP server with one tool, `get_weather`, that answers current conditions and a short forecast from Open-Meteo as one short paragraph a speech model can say aloud.

**Architecture:** Four pure modules and one thin MCP surface. `config.rs` reads env through an injected lookup; `location.rs` decides *which place* a call is about, with no I/O; `open_meteo.rs` makes the two HTTP calls; `render.rs` turns a forecast into prose. `server.rs` is the only file that knows about MCP, and it composes the other four. Every seam that a test needs to control — the two endpoint URLs, the env lookup — is a value on `Config`, so no test touches the process environment or the network.

**Tech Stack:** Rust 2024, `rmcp` 3.2.0 (`server`, `macros`, `schemars`, `transport-io`), `reqwest` 0.13.5, `tokio` 1.53.1, `chrono` 0.4.45, `wiremock` 0.6.5.

**Spec:** [`../specs/2026-09-10-open-meteo-mcp-design.md`](../specs/2026-09-10-open-meteo-mcp-design.md)

## Everything below was verified against a real compile

This plan was not written from memory. A throwaway crate with these exact
dependencies was built, tested, `cargo fmt --check`ed and
`cargo clippy --all-targets -D warnings`ed before the plan was written:
**24 tests passing, fmt clean, clippy clean.** The live Open-Meteo endpoints
were called by hand, once each, to capture the real response shapes.

Six things were wrong on the first attempt. They are the reason this section
exists — **do not "correct" the code below back to what looks more familiar:**

| What looks right | What actually compiles | Where it bites |
| --- | --- | --- |
| `reqwest` feature `rustls-tls` | `rustls` | reqwest 0.13 renamed it; the old name fails dependency resolution, not compilation |
| `.query(&params)` works out of the box | needs the `query` feature | reqwest 0.13 gates `RequestBuilder::query` behind it; the error is `no method named 'query'` |
| `ContentBlock` has a `.raw` field wrapping `RawContent` | `ContentBlock` **is** the enum: `ContentBlock::Text(TextContent)` | rmcp 3.x flattened the `Annotated` wrapper. Match on the block directly |
| `Weekday::to_string()` gives `"Saturday"` | it gives `"Sat"` | a speech model reads the abbreviation aloud. `weekday_name()` spells it out |
| geocoding returns `"results": []` on no match | it **omits `results` entirely** | `Option<Vec<Place>>`, not `Vec<Place>` — a `Vec` with `#[serde(default)]` would silently work, but the field is genuinely absent and the type should say so |
| `80 \| 81 \| 82` is fine | clippy demands `80..=82` | `manual_range_patterns` is on by default under `-D warnings` |

Two more verified facts worth keeping:

- Imperial wind units come back from the API labelled **`"mp/h"`**, not
  `"mph"`. The rendered text uses `"mph"` deliberately — it is what a person
  says — and never echoes the API's unit strings.
- A bad request returns HTTP 400 with `{"error":true,"reason":"..."}`. The
  status check catches it before the body is parsed, so the reason string is
  never depended on.

## Global Constraints

- **TDD is non-negotiable.** Write the failing test, run it, watch it fail
  for the right reason, then implement. Never the reverse.
- **Every task commits to `docs/open-meteo-mcp-design`.** One branch per
  repo, one PR per repo. Do not create a branch per task.
- **No test touches the live Open-Meteo API.** Every HTTP test runs against
  `wiremock`. CI must not depend on a third party's availability, and
  hammering a free non-commercial endpoint from CI is what their terms
  discourage.
- **No test touches process environment.** `std::env::set_var` is `unsafe`
  in edition 2024 and Rust runs tests in parallel threads inside one
  process, so a test that sets a variable corrupts its neighbours.
  `Config::from_vars()` takes the lookup as a closure; `Config::from_env()`
  is the one-line wrapper over `std::env::var`.
- **Errors the caller should see are `CallToolResult::error(...)`, never
  `Err(ErrorData)`.** rmcp's own documentation draws this line: a protocol
  error is rendered opaquely by the client ("Tool result missing due to
  internal error") and the message never reaches the user, whereas a
  tool-level error result is content the model can read and act on. Every
  failure in this server — no default location, half a coordinate pair, an
  unknown place name, a 503, malformed JSON — is a tool-level error result.
  `Err(ErrorData)` is not used anywhere in this crate.
- **Pin exact latest-stable versions; never a beta, RC or preview.** The
  versions in Task 1's `Cargo.toml` were the current stable releases on
  2026-09-10, confirmed against the crates.io API.
- **`Cargo.lock` is committed.** This crate produces a binary; the lockfile
  is what makes its build reproducible. `.gitignore` on `chore/add-gitignore`
  already excludes `target/` (unanchored — there is no Cargo workspace) and
  deliberately does not exclude `Cargo.lock`.
- `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` must
  both pass before every commit.
- Apache-2.0. This repo has **no copyright-header script** — unlike UIA, do
  not add per-file licence headers.

## Model Assignment

Current-generation Claude only. Never pin a 3.x or 4.x model; there is no
Haiku 5 — Haiku is still 4.5.

| Work | Model | ID | Why |
| --- | --- | --- | --- |
| Tasks 1–4 (implementation) | Sonnet 5 | `claude-sonnet-5` | Every test and implementation body is written out below and verified to compile. This is execution, not design. |
| Task 5 (packaging) | Sonnet 5 | `claude-sonnet-5` | Mechanical, and the four checks it must satisfy are quoted verbatim from UIA's validator. |
| Task 6 (README) | Sonnet 5 | `claude-sonnet-5` | Prose against a stated requirement, with the wording that matters supplied. |
| Review between tasks | Opus 5 | `claude-opus-5` | Reviewing against the spec's intent, not just whether tests pass. |

For subagent dispatch the `model` parameter takes the short name: `sonnet`
for every task, `opus` for every review.

**Escalate to Opus 5 mid-task** if the compiler disagrees with this plan's
code, or a test fails for a reason the plan does not predict. That means the
plan is wrong, and correcting a plan is design work — stop and report rather
than improvising a fix.

## Ledger mapping

The work ledger tracks these as rows B3–B8. One task per row, in order:

| Ledger row | Task |
| --- | --- |
| B3 Scaffold the crate | Task 1 |
| B4 `get_weather` + resolution order | Task 2 |
| B5 Geocoding + spoken output | Task 3 |
| B6 Config seams | Task 4 |
| B7 `.mcpb` packaging | Task 5 |
| B8 README + root table row | Task 6 |

Row **B9** (UIA home-location seam) is *not* in this plan. It lands in the
`user-interface-agents` repo, against a different codebase, on that repo's
branch — it needs its own plan written against UIA's settings and session
code, and folding it in here would break the one-branch-per-repo rule this
plan is built on.

## File Structure

```
mcp-servers/open-meteo-mcp/
├── Cargo.toml          # Task 1
├── Cargo.lock          # committed, Task 1
├── README.md           # Task 6
├── manifest.json       # .mcpb template, Task 5
└── src/
    ├── main.rs         # stdio wiring and module declarations — Task 1
    ├── config.rs       # env → Config, through an injected lookup — Task 1, extended Task 4
    ├── server.rs       # the MCP surface: WeatherServer, get_weather — Task 1, extended 2/3/4
    ├── location.rs     # which place a call is about; pure, no I/O — Task 2
    ├── open_meteo.rs   # the two HTTP calls and their response types — Task 2, extended 3/4
    └── render.rs       # forecast → one speakable paragraph — Task 3, extended Task 4
```

Tests are inline `#[cfg(test)] mod tests` in the module they cover. The
crate is a binary, so there is no `tests/` directory: integration tests
there could not import these modules.

---

### Task 1: Scaffold the crate and its configuration

**Files:**
- Create: `mcp-servers/open-meteo-mcp/Cargo.toml`
- Create: `mcp-servers/open-meteo-mcp/src/main.rs`
- Create: `mcp-servers/open-meteo-mcp/src/config.rs`
- Create: `mcp-servers/open-meteo-mcp/src/server.rs`
- Test: inline in `config.rs` and `server.rs`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `config::Config { default_location: Option<String>, forecast_url: String, geocoding_url: String }`
  - `config::Config::from_env() -> Config` and `config::Config::from_vars(get: impl Fn(&str) -> Option<String>) -> Config`
  - `config::DEFAULT_FORECAST_URL`, `config::DEFAULT_GEOCODING_URL`
  - `server::WeatherServer::new(config: Config) -> WeatherServer`
  - `server::GetWeatherArgs { location: Option<String>, latitude: Option<f64>, longitude: Option<f64>, days: Option<i64> }`
  - `server::ASK_USER: &str`
  - `server::WeatherServer::get_weather(&self, Parameters(args): Parameters<GetWeatherArgs>) -> Result<CallToolResult, ErrorData>`

The deliverable is a **real, runnable MCP server**: it speaks stdio, it
lists one tool, and with nothing configured it honestly says it needs a
location. It cannot fetch weather yet. That is the point — the tier-3
behaviour from the spec ("the default is unset on purpose") is the first
thing that works, rather than the last thing bolted on.

- [ ] **Step 1: Create the crate manifest**

Create `mcp-servers/open-meteo-mcp/Cargo.toml`. Every version here is the
latest stable as of 2026-09-10 and every feature list was resolved by a real
`cargo build` — see the table above for the two that are not what they look
like.

```toml
[package]
name = "open-meteo-mcp"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
license = "Apache-2.0"
description = "An MCP server answering current conditions and short forecasts from Open-Meteo."

[dependencies]
rmcp = { version = "3.2.0", features = ["server", "macros", "schemars", "transport-io"] }
tokio = { version = "1.53.1", features = ["macros", "rt-multi-thread", "io-std"] }
serde = { version = "1.0.229", features = ["derive"] }
serde_json = "1.0.151"
schemars = "1.2.2"
# `rustls`, not `rustls-tls` — reqwest 0.13 renamed it. `query` is what
# gates `RequestBuilder::query`; without it the call does not exist.
reqwest = { version = "0.13.5", features = ["json", "query", "rustls"], default-features = false }
anyhow = "1.0.104"
chrono = { version = "0.4.45", default-features = false, features = ["std", "alloc"] }
tracing = "0.1.44"
tracing-subscriber = { version = "0.3.23", features = ["env-filter"] }

[dev-dependencies]
wiremock = "0.6.5"
```

- [ ] **Step 2: Write the failing config tests**

Create `mcp-servers/open-meteo-mcp/src/config.rs` containing **only** this
test module for now:

```rust
//! Environment configuration. Parsed through an injected lookup so tests
//! never touch process env: `std::env::set_var` is `unsafe` in edition 2024
//! and Rust runs tests in parallel threads within one process.

#[cfg(test)]
mod tests {
    use super::*;

    fn empty(_: &str) -> Option<String> {
        None
    }

    #[test]
    fn defaults_are_the_public_endpoints_and_no_default_location() {
        let c = Config::from_vars(empty);
        assert_eq!(c.forecast_url, DEFAULT_FORECAST_URL);
        assert_eq!(c.geocoding_url, DEFAULT_GEOCODING_URL);
        assert!(c.default_location.is_none());
    }

    #[test]
    fn the_urls_are_overridable_because_that_is_the_commercial_tier_seam() {
        let c = Config::from_vars(|k| {
            (k == "OPEN_METEO_BASE_URL").then(|| "https://customer-api.example/v1/forecast".to_string())
        });
        assert_eq!(c.forecast_url, "https://customer-api.example/v1/forecast");
        assert_eq!(c.geocoding_url, DEFAULT_GEOCODING_URL);
    }

    #[test]
    fn a_blank_default_location_is_treated_as_unset() {
        let c = Config::from_vars(|k| {
            (k == "OPEN_METEO_DEFAULT_LOCATION").then(|| "   ".to_string())
        });
        assert!(c.default_location.is_none());
    }
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cd mcp-servers/open-meteo-mcp && cargo test`
Expected: FAIL — `cannot find value 'DEFAULT_FORECAST_URL' in this scope`,
`failed to resolve: use of undeclared type 'Config'`. The crate has no
`main.rs` yet either, so the build fails before the tests run. That is the
expected first failure.

- [ ] **Step 4: Implement the configuration**

Insert above the test module in `src/config.rs`:

```rust
pub const DEFAULT_FORECAST_URL: &str = "https://api.open-meteo.com/v1/forecast";
pub const DEFAULT_GEOCODING_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";

#[derive(Debug, Clone)]
pub struct Config {
    pub default_location: Option<String>,
    pub forecast_url: String,
    pub geocoding_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self::from_vars(|k| std::env::var(k).ok())
    }

    /// The lookup is a parameter, not `std::env`, so tests can vary it
    /// without mutating shared process state.
    pub fn from_vars(get: impl Fn(&str) -> Option<String>) -> Self {
        Config {
            default_location: get("OPEN_METEO_DEFAULT_LOCATION").filter(|v| !v.trim().is_empty()),
            forecast_url: get("OPEN_METEO_BASE_URL")
                .unwrap_or_else(|| DEFAULT_FORECAST_URL.to_string()),
            geocoding_url: get("OPEN_METEO_GEOCODING_URL")
                .unwrap_or_else(|| DEFAULT_GEOCODING_URL.to_string()),
        }
    }
}
```

- [ ] **Step 5: Write the failing server tests**

Create `mcp-servers/open-meteo-mcp/src/server.rs` containing **only** this
test module for now:

```rust
//! The MCP surface: one tool, `get_weather`.

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::ContentBlock;

    fn args() -> GetWeatherArgs {
        GetWeatherArgs {
            location: None,
            latitude: None,
            longitude: None,
            days: None,
        }
    }

    /// rmcp 3.x has no `Annotated`/`.raw` wrapper — `ContentBlock` is the
    /// enum itself.
    fn text_of(result: &CallToolResult) -> String {
        result
            .content
            .iter()
            .filter_map(|b| match b {
                ContentBlock::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .collect()
    }

    fn server(default_location: Option<&str>) -> WeatherServer {
        WeatherServer::new(Config::from_vars(|k| match k {
            "OPEN_METEO_DEFAULT_LOCATION" => default_location.map(str::to_string),
            _ => None,
        }))
    }

    #[test]
    fn the_router_lists_exactly_one_tool_named_get_weather() {
        let tools = WeatherServer::tool_router().list_all();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "get_weather");
    }

    #[tokio::test]
    async fn no_arguments_and_no_default_is_an_error_result_telling_the_model_to_ask() {
        let result = server(None).get_weather(Parameters(args())).await.unwrap();
        assert_eq!(result.is_error, Some(true));
        assert_eq!(text_of(&result), ASK_USER);
    }
}
```

- [ ] **Step 6: Run the tests to verify they fail**

Run: `cd mcp-servers/open-meteo-mcp && cargo test`
Expected: FAIL — `cannot find type 'GetWeatherArgs' in this scope` and
friends.

- [ ] **Step 7: Implement the server surface**

Insert above the test module in `src/server.rs`. `#[tool_router]` generates
`WeatherServer::tool_router()`; `#[tool_handler]` fills in `call_tool`,
`list_tools` and `get_info` from it, so no `ServerHandler` method is
written by hand.

```rust
use crate::config::Config;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{ErrorData, ServerHandler, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

/// What the tool says when it has nothing to go on. Written at the model,
/// not the user: it is an instruction to ask, not a sentence to read aloud.
pub const ASK_USER: &str = "No default location is configured. Ask the user which city or place \
they want the weather for, then call get_weather again with it.";

/// Doc comments on these fields become the tool's JSON Schema descriptions.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetWeatherArgs {
    /// Place name, e.g. "Melbourne" or "Melbourne, Australia".
    pub location: Option<String>,
    /// Decimal degrees. Use with longitude instead of location.
    pub latitude: Option<f64>,
    /// Decimal degrees. Use with latitude instead of location.
    pub longitude: Option<f64>,
    /// Days of forecast, 1-7. Default 3.
    pub days: Option<i64>,
}

#[derive(Clone)]
pub struct WeatherServer {
    config: Config,
}

impl WeatherServer {
    pub fn new(config: Config) -> Self {
        WeatherServer { config }
    }
}

#[tool_router]
impl WeatherServer {
    #[tool(description = "Current conditions and a short forecast for a place. \
Give a place name, or a latitude/longitude pair.")]
    pub async fn get_weather(
        &self,
        Parameters(args): Parameters<GetWeatherArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        // Task 2 replaces this with the full resolution order. Until then the
        // only honest answer is the one the spec's tier 3 asks for.
        let _ = (&args, &self.config);
        Ok(error_text(ASK_USER))
    }
}

/// Tool-level error, never `Err(ErrorData)`: MCP clients render protocol
/// errors opaquely, so an `Err` here would reach the user as a spoken
/// failure with none of this text in it.
fn error_text(text: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(text)])
}

#[tool_handler(
    name = "open-meteo-mcp",
    instructions = "Answers questions about current weather and short forecasts. \
Weather is live data; do not answer from memory."
)]
impl ServerHandler for WeatherServer {}
```

- [ ] **Step 8: Wire up stdio**

Create `mcp-servers/open-meteo-mcp/src/main.rs`:

```rust
mod config;
mod server;

use rmcp::ServiceExt;
use rmcp::transport::stdio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // stderr, never stdout: stdout is the MCP transport, and one stray
    // line on it corrupts the JSON-RPC stream.
    tracing_subscriber::fmt().with_writer(std::io::stderr).init();

    let service = server::WeatherServer::new(config::Config::from_env())
        .serve(stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}
```

- [ ] **Step 9: Run the tests to verify they pass**

Run: `cd mcp-servers/open-meteo-mcp && cargo test`
Expected: PASS — 5 tests (3 in `config`, 2 in `server`).

- [ ] **Step 10: Run the gates**

Run: `cargo fmt --check && cargo clippy --all-targets -- -D warnings`
Expected: both clean, no output from clippy but the `Finished` line.

- [ ] **Step 11: Commit**

```bash
git add mcp-servers/open-meteo-mcp/
git commit -m "feat(open-meteo-mcp): scaffold the crate, config and MCP surface"
```

---

### Task 2: Resolution order, the days clamp, and a forecast by coordinates

**Files:**
- Create: `mcp-servers/open-meteo-mcp/src/location.rs`
- Create: `mcp-servers/open-meteo-mcp/src/open_meteo.rs`
- Create: `mcp-servers/open-meteo-mcp/src/render.rs`
- Modify: `mcp-servers/open-meteo-mcp/src/main.rs` (three new `mod` lines)
- Modify: `mcp-servers/open-meteo-mcp/src/server.rs` (`get_weather` body, imports, one new test)
- Test: inline in `location.rs`, `open_meteo.rs`, `render.rs`, `server.rs`

**Interfaces:**
- Consumes: `config::Config` (Task 1), `server::error_text`, `server::ASK_USER` (Task 1).
- Produces:
  - `location::Resolution` — `Coords { latitude: f64, longitude: f64 } | Name(String) | Ask | MissingHalf(&'static str)`
  - `location::Clamped { days: u8, note: Option<String> }`
  - `location::clamp_days(days: Option<i64>) -> Clamped`
  - `location::resolve(location: Option<&str>, latitude: Option<f64>, longitude: Option<f64>, config: &Config) -> Resolution`
  - `open_meteo::{Current, Daily, Forecast, WeatherError}`
  - `open_meteo::REQUEST_TIMEOUT: std::time::Duration`
  - `open_meteo::Client::new(config: Config) -> Client`
  - `open_meteo::Client::forecast(&self, latitude: f64, longitude: f64, days: u8, timezone: Option<&str>) -> Result<Forecast, WeatherError>`
  - `render::describe_code(code: i64) -> &'static str`, `render::ATTRIBUTION`
  - `render::render(place: &str, forecast: &Forecast, clamp_note: Option<&str>) -> String`

**Deliberately incomplete, and the next task finishes it:** `Resolution::Name`
is produced here but not acted on — `get_weather` answers a place name with
an error result saying so, and Task 3 replaces that one match arm and its one
test. Geocoding is a second HTTP call with its own failure modes, and gating
it separately is what lets a reviewer reject the resolution rules without
also rejecting the geocoder.

Likewise `render()` renders **current conditions only** here. Task 3 appends
the daily lines. The tests below assert `starts_with`/`ends_with` rather than
whole-string equality precisely so Task 3 extends them instead of deleting
them.

- [ ] **Step 1: Write the failing resolution tests**

Create `mcp-servers/open-meteo-mcp/src/location.rs`:

```rust
//! Argument resolution: which place a `get_weather` call is actually about.
//! Pure — no I/O, no clock, no network — so every rule here is cheap to test.

use crate::config::Config;

pub const MIN_DAYS: u8 = 1;
pub const MAX_DAYS: u8 = 7;
pub const DEFAULT_DAYS: u8 = 3;

#[cfg(test)]
mod tests {
    use super::*;

    fn config(default_location: Option<&str>) -> Config {
        Config::from_vars(|k| match k {
            "OPEN_METEO_DEFAULT_LOCATION" => default_location.map(str::to_string),
            _ => None,
        })
    }

    #[test]
    fn coordinates_win_and_a_location_given_alongside_them_is_ignored() {
        let r = resolve(
            Some("Melbourne"),
            Some(-37.814),
            Some(144.9633),
            &config(None),
        );
        assert_eq!(
            r,
            Resolution::Coords {
                latitude: -37.814,
                longitude: 144.9633
            }
        );
    }

    #[test]
    fn a_name_alone_is_geocoded() {
        assert_eq!(
            resolve(Some("Melbourne"), None, None, &config(None)),
            Resolution::Name("Melbourne".into())
        );
    }

    #[test]
    fn nothing_with_a_default_configured_uses_the_default() {
        assert_eq!(
            resolve(None, None, None, &config(Some("Hobart"))),
            Resolution::Name("Hobart".into())
        );
    }

    #[test]
    fn nothing_with_no_default_asks_rather_than_guessing() {
        assert_eq!(resolve(None, None, None, &config(None)), Resolution::Ask);
    }

    #[test]
    fn half_a_coordinate_pair_names_the_half_that_is_missing() {
        assert_eq!(
            resolve(None, Some(-37.8), None, &config(None)),
            Resolution::MissingHalf("longitude")
        );
        assert_eq!(
            resolve(None, None, Some(144.9), &config(None)),
            Resolution::MissingHalf("latitude")
        );
    }

    #[test]
    fn days_clamp_at_both_ends_and_say_so() {
        assert_eq!(
            clamp_days(None),
            Clamped {
                days: 3,
                note: None
            }
        );
        let low = clamp_days(Some(0));
        assert_eq!(low.days, 1);
        assert!(low.note.unwrap().contains("at least 1"));
        let high = clamp_days(Some(99));
        assert_eq!(high.days, 7);
        assert!(high.note.unwrap().contains("at most 7"));
        assert_eq!(
            clamp_days(Some(5)),
            Clamped {
                days: 5,
                note: None
            }
        );
    }
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cd mcp-servers/open-meteo-mcp && cargo test location`
Expected: FAIL — `cannot find function 'resolve' in this scope`,
`cannot find type 'Resolution' in this scope`. (The build fails, so no test
runs; that is the failure.)

- [ ] **Step 3: Implement resolution**

Insert above the test module in `src/location.rs`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Resolution {
    Coords { latitude: f64, longitude: f64 },
    Name(String),
    Ask,
    MissingHalf(&'static str),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Clamped {
    pub days: u8,
    /// Set only when the request was actually changed. The result says so
    /// out loud: a model that asked for 10 days should hear why it got 7,
    /// rather than silently believing it got what it asked for.
    pub note: Option<String>,
}

pub fn clamp_days(days: Option<i64>) -> Clamped {
    let Some(asked) = days else {
        return Clamped {
            days: DEFAULT_DAYS,
            note: None,
        };
    };
    let clamped = asked.clamp(MIN_DAYS as i64, MAX_DAYS as i64) as u8;
    let note = (i64::from(clamped) != asked)
        .then(|| format!("Asked for {asked} days; this server returns at most {MAX_DAYS} and at least {MIN_DAYS}, so {clamped} are shown."));
    Clamped {
        days: clamped,
        note,
    }
}

/// The spec's resolution order, in one place: coordinates, then a name,
/// then the configured default, then ask.
pub fn resolve(
    location: Option<&str>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    config: &Config,
) -> Resolution {
    match (latitude, longitude) {
        (Some(latitude), Some(longitude)) => Resolution::Coords {
            latitude,
            longitude,
        },
        (Some(_), None) => Resolution::MissingHalf("longitude"),
        (None, Some(_)) => Resolution::MissingHalf("latitude"),
        (None, None) => match location.map(str::trim).filter(|s| !s.is_empty()) {
            Some(name) => Resolution::Name(name.to_string()),
            None => match config.default_location.as_deref() {
                Some(default) => Resolution::Name(default.trim().to_string()),
                None => Resolution::Ask,
            },
        },
    }
}
```

- [ ] **Step 4: Run them to verify they pass**

Run: `cd mcp-servers/open-meteo-mcp && cargo test location`
Expected: PASS — 6 tests.

- [ ] **Step 5: Write the failing forecast-client tests**

Create `mcp-servers/open-meteo-mcp/src/open_meteo.rs`:

```rust
//! The HTTP calls to Open-Meteo, and the shapes they answer with.
//!
//! Every field name here was checked against a live response on 2026-09-10.

use crate::config::Config;
use serde::Deserialize;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::Duration;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Points the client at the mock server. No test in this crate is ever
    /// allowed to reach the real api.open-meteo.com.
    fn config(server: &MockServer) -> Config {
        let base = server.uri();
        Config::from_vars(move |k| match k {
            "OPEN_METEO_BASE_URL" => Some(format!("{base}/v1/forecast")),
            "OPEN_METEO_GEOCODING_URL" => Some(format!("{base}/v1/search")),
            _ => None,
        })
    }

    fn melbourne_forecast() -> serde_json::Value {
        json!({
            "current": {"temperature_2m": 14.3, "weather_code": 3, "wind_speed_10m": 19.4},
            "daily": {
                "time": ["2026-09-10", "2026-09-11", "2026-09-12"],
                "weather_code": [61, 80, 0],
                "temperature_2m_max": [17.0, 16.0, 19.0],
                "temperature_2m_min": [11.0, 9.0, 10.0],
                "precipitation_sum": [4.0, 0.0, 0.0]
            }
        })
    }

    #[tokio::test]
    async fn a_forecast_is_parsed_and_no_hourly_block_is_ever_requested() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/forecast"))
            .respond_with(ResponseTemplate::new(200).set_body_json(melbourne_forecast()))
            .mount(&server)
            .await;

        let forecast = Client::new(config(&server))
            .forecast(-37.814, 144.9633, 3, Some("Australia/Melbourne"))
            .await
            .unwrap();
        assert_eq!(forecast.current.temperature_2m, 14.3);
        assert_eq!(forecast.daily.time.len(), 3);

        let url = server.received_requests().await.unwrap()[0].url.to_string();
        assert!(!url.contains("hourly"), "{url}");
        assert!(url.contains("forecast_days=3"), "{url}");
        assert!(url.contains("Australia%2FMelbourne"), "{url}");
    }

    #[tokio::test]
    async fn a_non_200_is_an_upstream_error_not_a_panic() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/forecast"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&server)
            .await;

        let err = Client::new(config(&server))
            .forecast(-37.8, 144.9, 3, None)
            .await
            .unwrap_err();
        assert!(matches!(err, WeatherError::Upstream(_)));
        assert!(err.to_string().contains("503"), "{err}");
    }

    /// The spec asks for a timeout to be an error result like any other.
    /// The client under test is built by hand with a short timeout: waiting
    /// out the real ten-second one would make the suite unusable.
    #[tokio::test]
    async fn a_request_that_never_answers_times_out_as_an_upstream_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/forecast"))
            .respond_with(
                ResponseTemplate::new(200).set_delay(REQUEST_TIMEOUT + Duration::from_secs(1)),
            )
            .mount(&server)
            .await;

        let client = Client {
            http: reqwest::Client::builder()
                .timeout(Duration::from_millis(150))
                .build()
                .unwrap(),
            config: config(&server),
        };
        let err = client.forecast(-37.8, 144.9, 3, None).await.unwrap_err();
        assert!(matches!(err, WeatherError::Upstream(_)));
    }

    #[tokio::test]
    async fn malformed_json_is_an_upstream_error_not_a_panic() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/forecast"))
            .respond_with(ResponseTemplate::new(200).set_body_string("{not json"))
            .mount(&server)
            .await;

        let err = Client::new(config(&server))
            .forecast(-37.8, 144.9, 3, None)
            .await
            .unwrap_err();
        assert!(matches!(err, WeatherError::Upstream(_)));
    }
}
```

- [ ] **Step 6: Run them to verify they fail**

Run: `cd mcp-servers/open-meteo-mcp && cargo test open_meteo`
Expected: FAIL — `cannot find type 'Client' in this scope`.

- [ ] **Step 7: Implement the forecast client**

Insert above the test module in `src/open_meteo.rs`:

```rust
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Current {
    pub temperature_2m: f64,
    pub weather_code: i64,
    pub wind_speed_10m: f64,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Daily {
    pub time: Vec<String>,
    pub weather_code: Vec<i64>,
    pub temperature_2m_max: Vec<f64>,
    pub temperature_2m_min: Vec<f64>,
    pub precipitation_sum: Vec<f64>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Forecast {
    pub current: Current,
    pub daily: Daily,
}

#[derive(Debug)]
pub enum WeatherError {
    NoMatch(String),
    Upstream(String),
}

impl std::fmt::Display for WeatherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeatherError::NoMatch(q) => write!(
                f,
                "I could not find a place called \"{q}\". Ask the user to say it another way, \
or to give a nearby larger town."
            ),
            WeatherError::Upstream(why) => write!(f, "The weather service did not answer: {why}."),
        }
    }
}

pub struct Client {
    http: reqwest::Client,
    config: Config,
}

/// A request that never returns would hang the tool for as long as the
/// caller waits. Ten seconds is well past a healthy response and well
/// inside a speech turn's patience.
pub const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

impl Client {
    pub fn new(config: Config) -> Self {
        Client {
            http: reqwest::Client::builder()
                .timeout(REQUEST_TIMEOUT)
                .build()
                .expect("a client with only a timeout set always builds"),
            config,
        }
    }

    pub async fn forecast(
        &self,
        latitude: f64,
        longitude: f64,
        days: u8,
        timezone: Option<&str>,
    ) -> Result<Forecast, WeatherError> {
        // `hourly` is deliberately absent: 24 rows a speech model would read
        // aloud, for an answer two sentences already give.
        let query = vec![
            ("latitude", latitude.to_string()),
            ("longitude", longitude.to_string()),
            ("current", "temperature_2m,weather_code,wind_speed_10m".to_string()),
            (
                "daily",
                "weather_code,temperature_2m_max,temperature_2m_min,precipitation_sum".to_string(),
            ),
            ("forecast_days", days.to_string()),
            ("timezone", timezone.unwrap_or("auto").to_string()),
        ];

        let response = self
            .http
            .get(&self.config.forecast_url)
            .query(&query)
            .send()
            .await
            .map_err(|e| WeatherError::Upstream(e.to_string()))?;
        // Checked before the body is touched: a 400 carries
        // `{"error":true,"reason":"..."}`, which would fail to parse as a
        // Forecast and report the wrong cause.
        if !response.status().is_success() {
            return Err(WeatherError::Upstream(format!(
                "HTTP {}",
                response.status().as_u16()
            )));
        }
        response
            .json()
            .await
            .map_err(|e| WeatherError::Upstream(e.to_string()))
    }
}
```

`WeatherError::NoMatch` is not constructed until Task 3, and that does not
warn: the `Display` impl reads the variant, and a variant that is matched
counts as used. Leave it in — Task 3 is what constructs it.

- [ ] **Step 8: Run them to verify they pass**

Run: `cd mcp-servers/open-meteo-mcp && cargo test open_meteo`
Expected: PASS — 4 tests.

- [ ] **Step 9: Write the failing render tests**

Create `mcp-servers/open-meteo-mcp/src/render.rs`:

```rust
//! Turning a forecast into one short paragraph a speech model can say.

use crate::open_meteo::Forecast;

pub const ATTRIBUTION: &str = "Data by Open-Meteo.com, CC-BY 4.0.";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open_meteo::{Current, Daily, Forecast};

    fn forecast() -> Forecast {
        Forecast {
            current: Current {
                temperature_2m: 14.3,
                weather_code: 3,
                wind_speed_10m: 19.4,
            },
            daily: Daily {
                time: vec![
                    "2026-09-10".into(),
                    "2026-09-11".into(),
                    "2026-09-12".into(),
                ],
                weather_code: vec![61, 80, 0],
                temperature_2m_max: vec![17.0, 16.0, 19.0],
                temperature_2m_min: vec![11.0, 9.0, 10.0],
                precipitation_sum: vec![4.0, 0.0, 0.0],
            },
        }
    }

    #[test]
    fn current_conditions_lead_and_read_as_spoken_words_not_codes() {
        let out = render("Melbourne, Victoria, Australia", &forecast(), None);
        assert!(
            out.starts_with("Melbourne, Victoria, Australia — currently 14°C, overcast, wind 19 km/h."),
            "{out}"
        );
    }

    #[test]
    fn every_successful_result_carries_the_attribution_line() {
        assert!(render("X", &forecast(), None).ends_with(ATTRIBUTION));
    }

    #[test]
    fn a_clamp_note_is_spoken_before_the_attribution() {
        let out = render("X", &forecast(), Some("Asked for 99 days; 7 are shown."));
        assert!(
            out.contains("Asked for 99 days; 7 are shown. Data by Open-Meteo.com"),
            "{out}"
        );
    }

    #[test]
    fn an_unknown_wmo_code_still_produces_a_word() {
        assert_eq!(describe_code(4242), "unsettled");
    }
}
```

- [ ] **Step 10: Run them to verify they fail**

Run: `cd mcp-servers/open-meteo-mcp && cargo test render`
Expected: FAIL — `cannot find function 'render' in this scope`.

- [ ] **Step 11: Implement rendering of current conditions**

Insert above the test module in `src/render.rs`. Note `80..=82`, not
`80 | 81 | 82` — clippy's `manual_range_patterns` rejects the latter under
`-D warnings`.

```rust
/// WMO 4677 present-weather codes, in the words a person would use.
/// The catch-all is a real answer rather than a number, because the
/// consumer has to say it out loud.
pub fn describe_code(code: i64) -> &'static str {
    match code {
        0 => "clear",
        1 => "mostly clear",
        2 => "partly cloudy",
        3 => "overcast",
        45 | 48 => "fog",
        51 | 53 | 55 => "drizzle",
        56 | 57 => "freezing drizzle",
        61 | 63 | 65 => "rain",
        66 | 67 => "freezing rain",
        71 | 73 | 75 => "snow",
        77 => "snow grains",
        80..=82 => "showers",
        85 | 86 => "snow showers",
        95 => "thunderstorms",
        96 | 99 => "thunderstorms with hail",
        _ => "unsettled",
    }
}

pub fn render(place: &str, forecast: &Forecast, clamp_note: Option<&str>) -> String {
    let mut out = format!(
        "{place} — currently {:.0}°C, {}, wind {:.0} km/h.",
        forecast.current.temperature_2m,
        describe_code(forecast.current.weather_code),
        forecast.current.wind_speed_10m,
    );

    // Task 3 appends the daily lines here.

    if let Some(note) = clamp_note {
        out.push(' ');
        out.push_str(note);
    }
    out.push(' ');
    out.push_str(ATTRIBUTION);
    out
}
```

- [ ] **Step 12: Run them to verify they pass**

Run: `cd mcp-servers/open-meteo-mcp && cargo test render`
Expected: PASS — 4 tests.

- [ ] **Step 13: Write the failing server-wiring tests**

In `src/server.rs`, add these two tests to the existing `mod tests`:

```rust
    #[tokio::test]
    async fn half_a_coordinate_pair_is_an_error_result_naming_the_missing_half() {
        let mut a = args();
        a.latitude = Some(-37.8);
        let result = server(None).get_weather(Parameters(a)).await.unwrap();
        assert_eq!(result.is_error, Some(true));
        assert!(
            text_of(&result).contains("longitude is missing"),
            "{}",
            text_of(&result)
        );
    }

    /// Replaced in Task 3, when geocoding lands. Until then the tool must
    /// say what it cannot do rather than answering about the wrong place.
    #[tokio::test]
    async fn a_place_name_is_refused_until_geocoding_lands() {
        let mut a = args();
        a.location = Some("Melbourne".into());
        let result = server(None).get_weather(Parameters(a)).await.unwrap();
        assert_eq!(result.is_error, Some(true));
        assert!(text_of(&result).contains("latitude and longitude"), "{}", text_of(&result));
    }
```

- [ ] **Step 14: Run them to verify they fail**

Run: `cd mcp-servers/open-meteo-mcp && cargo test server`
Expected: FAIL — both assertions fail, because `get_weather` still returns
`ASK_USER` for everything.

- [ ] **Step 15: Wire `get_weather` to the resolution order**

In `src/main.rs`, add the three module declarations, keeping them
alphabetical:

```rust
mod config;
mod location;
mod open_meteo;
mod render;
mod server;
```

In `src/server.rs`, replace the imports and the whole `get_weather` body:

```rust
use crate::config::Config;
use crate::location::{Resolution, clamp_days, resolve};
use crate::open_meteo::Client;
use crate::render::render;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{ErrorData, ServerHandler, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
```

```rust
    #[tool(description = "Current conditions and a short forecast for a place. \
Give a place name, or a latitude/longitude pair.")]
    pub async fn get_weather(
        &self,
        Parameters(args): Parameters<GetWeatherArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let clamped = clamp_days(args.days);
        let client = Client::new(self.config.clone());

        let (place, latitude, longitude) = match resolve(
            args.location.as_deref(),
            args.latitude,
            args.longitude,
            &self.config,
        ) {
            Resolution::Ask => return Ok(error_text(ASK_USER)),
            Resolution::MissingHalf(missing) => {
                return Ok(error_text(format!(
                    "A coordinate pair needs both halves — {missing} is missing. \
Give both latitude and longitude, or a place name instead."
                )));
            }
            // Task 3 replaces this arm with a geocoding call.
            Resolution::Name(_) => {
                return Ok(error_text(
                    "This server cannot look up a place by name yet. \
Give latitude and longitude instead.",
                ));
            }
            Resolution::Coords {
                latitude,
                longitude,
            } => (
                format!("{latitude:.4}, {longitude:.4}"),
                latitude,
                longitude,
            ),
        };

        match client.forecast(latitude, longitude, clamped.days, None).await {
            Ok(forecast) => Ok(CallToolResult::success(vec![ContentBlock::text(render(
                &place,
                &forecast,
                clamped.note.as_deref(),
            ))])),
            Err(e) => Ok(error_text(e.to_string())),
        }
    }
```

- [ ] **Step 16: Run the whole suite**

Run: `cd mcp-servers/open-meteo-mcp && cargo test`
Expected: PASS — **21 tests**: 3 in `config`, 6 in `location`, 4 in
`open_meteo`, 4 in `render`, 4 in `server`.

- [ ] **Step 17: Run the gates**

Run: `cargo fmt --check && cargo clippy --all-targets -- -D warnings`
Expected: both clean.

- [ ] **Step 18: Commit**

```bash
git add mcp-servers/open-meteo-mcp/
git commit -m "feat(open-meteo-mcp): resolution order, days clamp and forecast by coordinates"
```

---

### Task 3: Geocoding, and the full spoken forecast

**Files:**
- Modify: `mcp-servers/open-meteo-mcp/src/open_meteo.rs` (add `Place`, `GeocodeResponse`, `Client::geocode`, and its tests)
- Modify: `mcp-servers/open-meteo-mcp/src/render.rs` (add the daily lines and their tests)
- Modify: `mcp-servers/open-meteo-mcp/src/server.rs` (replace the `Resolution::Name` arm and its test)
- Test: inline in `open_meteo.rs`, `render.rs`, `server.rs`

**Interfaces:**
- Consumes: everything from Task 2.
- Produces:
  - `open_meteo::Place { name: String, latitude: f64, longitude: f64, country: Option<String>, admin1: Option<String>, timezone: Option<String> }`
  - `open_meteo::Place::label(&self) -> String` — `"Melbourne, Victoria, Australia"`
  - `open_meteo::Client::geocode(&self, name: &str) -> Result<Place, WeatherError>`
  - `render::render` gains the daily sentences; the signature is unchanged.

This is the task that makes the spec's example output real. Two behaviours
in it are load-bearing and were argued for in the spec, so do not
"simplify" them away:

1. **The result names the place it picked.** A multi-match name resolves to
   the first result and says which one, so somebody who meant Melbourne,
   Florida hears that it went to Victoria rather than silently getting the
   wrong forecast.
2. **The geocoded IANA timezone is passed to the forecast call.** It comes
   back from the same geocoding response, and using it is what makes
   "Today" mean today *there*.

- [ ] **Step 1: Write the failing geocoding tests**

In `src/open_meteo.rs`, add to the existing `mod tests`. Note the no-match
body: the live API **omits `results` entirely** rather than returning an
empty array — verified against the real endpoint on 2026-09-10.

```rust
    fn melbourne_place() -> serde_json::Value {
        json!({"results": [{
            "name": "Melbourne", "latitude": -37.814, "longitude": 144.96332,
            "country": "Australia", "admin1": "Victoria", "timezone": "Australia/Melbourne"
        }]})
    }

    #[tokio::test]
    async fn a_matched_name_yields_a_place_labelled_with_region_and_country() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/search"))
            .and(query_param("name", "Melbourne"))
            .respond_with(ResponseTemplate::new(200).set_body_json(melbourne_place()))
            .mount(&server)
            .await;

        let place = Client::new(config(&server)).geocode("Melbourne").await.unwrap();
        assert_eq!(place.label(), "Melbourne, Victoria, Australia");
        assert_eq!(place.timezone.as_deref(), Some("Australia/Melbourne"));
        assert_eq!(place.latitude, -37.814);
    }

    #[tokio::test]
    async fn a_name_that_matches_nothing_is_an_error_quoting_the_input() {
        let server = MockServer::start().await;
        // The live API omits `results` entirely; it does not return [].
        Mock::given(method("GET"))
            .and(path("/v1/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"generationtime_ms": 0.5})))
            .mount(&server)
            .await;

        let err = Client::new(config(&server)).geocode("zzzqqq").await.unwrap_err();
        assert!(matches!(err, WeatherError::NoMatch(ref q) if q == "zzzqqq"));
        assert!(err.to_string().contains("\"zzzqqq\""), "{err}");
    }

    #[tokio::test]
    async fn a_place_missing_its_admin_region_is_still_labelled() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"results": [{
                "name": "Singapore", "latitude": 1.28967, "longitude": 103.85007,
                "country": "Singapore", "timezone": "Asia/Singapore"
            }]})))
            .mount(&server)
            .await;

        let place = Client::new(config(&server)).geocode("Singapore").await.unwrap();
        assert_eq!(place.label(), "Singapore, Singapore");
    }
```

Extend the `wiremock::matchers` import at the top of `mod tests` to include
`query_param`:

```rust
    use wiremock::matchers::{method, path, query_param};
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cd mcp-servers/open-meteo-mcp && cargo test open_meteo`
Expected: FAIL — `no method named 'geocode' found for struct 'Client'`.

- [ ] **Step 3: Implement geocoding**

In `src/open_meteo.rs`, add the types above the `Client` struct:

```rust
/// One geocoding hit. `country`, `admin1` and `timezone` are optional
/// because the API genuinely omits them for some places — a city-state has
/// no admin region.
#[derive(Debug, Deserialize, Clone)]
pub struct Place {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub country: Option<String>,
    pub admin1: Option<String>,
    pub timezone: Option<String>,
}

impl Place {
    /// "Melbourne, Victoria, Australia" — enough for a listener to hear
    /// that it picked the wrong Melbourne.
    pub fn label(&self) -> String {
        [
            Some(self.name.clone()),
            self.admin1.clone(),
            self.country.clone(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(", ")
    }
}

/// `results` is `Option` rather than a defaulted `Vec` because the field is
/// genuinely absent on a no-match, and the type should say what the wire
/// actually does.
#[derive(Debug, Deserialize)]
struct GeocodeResponse {
    #[serde(default)]
    results: Option<Vec<Place>>,
}
```

and the method inside `impl Client`:

```rust
    pub async fn geocode(&self, name: &str) -> Result<Place, WeatherError> {
        // count=1: the spec resolves a multi-match to the first hit and
        // names it, rather than asking the model to choose.
        let query = vec![
            ("name", name.to_string()),
            ("count", "1".to_string()),
            ("language", "en".to_string()),
            ("format", "json".to_string()),
        ];

        let response = self
            .http
            .get(&self.config.geocoding_url)
            .query(&query)
            .send()
            .await
            .map_err(|e| WeatherError::Upstream(e.to_string()))?;
        if !response.status().is_success() {
            return Err(WeatherError::Upstream(format!(
                "HTTP {}",
                response.status().as_u16()
            )));
        }
        let body: GeocodeResponse = response
            .json()
            .await
            .map_err(|e| WeatherError::Upstream(e.to_string()))?;
        body.results
            .and_then(|r| r.into_iter().next())
            .ok_or_else(|| WeatherError::NoMatch(name.to_string()))
    }
```

- [ ] **Step 4: Run them to verify they pass**

Run: `cd mcp-servers/open-meteo-mcp && cargo test open_meteo`
Expected: PASS — 7 tests.

- [ ] **Step 5: Write the failing full-paragraph test**

In `src/render.rs`, add to the existing `mod tests`. The literal below is
the verified output of the code in Step 6 — **2026-09-12 is a Saturday**,
and the third day is named rather than dated because "Today" and "Tomorrow"
have already been used:

```rust
    #[test]
    fn a_rendered_forecast_reads_as_one_speakable_paragraph() {
        let out = render("Melbourne, Victoria, Australia", &forecast(), None);
        assert_eq!(
            out,
            "Melbourne, Victoria, Australia — currently 14°C, overcast, wind 19 km/h. \
Today 11-17°C, rain 4mm. Tomorrow 9-16°C, showers. Saturday 10-19°C, clear. \
Data by Open-Meteo.com, CC-BY 4.0."
        );
    }

    #[test]
    fn no_hourly_rows_appear_in_the_output() {
        let out = render("X", &forecast(), None);
        // Three daily sentences, one current sentence, one attribution.
        assert_eq!(out.matches("°C").count(), 4);
    }
```

- [ ] **Step 6: Run it to verify it fails**

Run: `cd mcp-servers/open-meteo-mcp && cargo test render`
Expected: FAIL — the rendered string stops after "wind 19 km/h." and the
daily sentences are missing.

- [ ] **Step 7: Implement the daily lines**

In `src/render.rs`, add the import and the two helpers:

```rust
use chrono::{Datelike, NaiveDate};
```

```rust
/// `Weekday::to_string()` renders "Sat", which a speech model reads aloud
/// as the abbreviation. Spelled out on purpose.
fn weekday_name(day: chrono::Weekday) -> &'static str {
    use chrono::Weekday::*;
    match day {
        Mon => "Monday",
        Tue => "Tuesday",
        Wed => "Wednesday",
        Thu => "Thursday",
        Fri => "Friday",
        Sat => "Saturday",
        Sun => "Sunday",
    }
}

/// Falls back to the ISO date rather than failing: an unparseable date is
/// still worth speaking, and this is a forecast, not a parser.
fn day_label(iso: &str, index: usize) -> String {
    match index {
        0 => "Today".to_string(),
        1 => "Tomorrow".to_string(),
        _ => NaiveDate::parse_from_str(iso, "%Y-%m-%d")
            .map(|d| weekday_name(d.weekday()).to_string())
            .unwrap_or_else(|_| iso.to_string()),
    }
}
```

and replace the `// Task 3 appends the daily lines here.` comment in
`render()` with:

```rust
    let days = &forecast.daily;
    for i in 0..days.time.len() {
        let precip = days.precipitation_sum.get(i).copied().unwrap_or(0.0);
        // The amount is only worth saying when there is any.
        let weather = if precip > 0.0 {
            format!(", {} {precip:.0}mm", describe_code(days.weather_code[i]))
        } else {
            format!(", {}", describe_code(days.weather_code[i]))
        };
        out.push_str(&format!(
            " {} {:.0}-{:.0}°C{weather}.",
            day_label(&days.time[i], i),
            days.temperature_2m_min[i],
            days.temperature_2m_max[i],
        ));
    }
```

- [ ] **Step 8: Run them to verify they pass**

Run: `cd mcp-servers/open-meteo-mcp && cargo test render`
Expected: PASS — 6 tests.

- [ ] **Step 9: Replace the placeholder server test**

In `src/server.rs`, **delete** `a_place_name_is_refused_until_geocoding_lands`
and add in its place:

```rust
    /// The one server test that needs a network seam, so it builds its own
    /// config rather than using `server()`.
    #[tokio::test]
    async fn a_named_place_is_geocoded_and_the_answer_says_which_place_it_picked() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{"name": "Melbourne", "latitude": -37.814, "longitude": 144.96332,
                             "country": "Australia", "admin1": "Victoria",
                             "timezone": "Australia/Melbourne"}]
            })))
            .mount(&mock)
            .await;
        Mock::given(method("GET"))
            .and(path("/v1/forecast"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "current": {"temperature_2m": 14.3, "weather_code": 3, "wind_speed_10m": 19.4},
                "daily": {"time": ["2026-09-10"], "weather_code": [3],
                          "temperature_2m_max": [17.0], "temperature_2m_min": [11.0],
                          "precipitation_sum": [0.0]}
            })))
            .mount(&mock)
            .await;

        let base = mock.uri();
        let server = WeatherServer::new(Config::from_vars(move |k| match k {
            "OPEN_METEO_BASE_URL" => Some(format!("{base}/v1/forecast")),
            "OPEN_METEO_GEOCODING_URL" => Some(format!("{base}/v1/search")),
            _ => None,
        }));

        let mut a = args();
        a.location = Some("Melbourne".into());
        let result = server.get_weather(Parameters(a)).await.unwrap();
        assert_eq!(result.is_error, Some(false));
        assert!(
            text_of(&result).starts_with("Melbourne, Victoria, Australia — "),
            "{}",
            text_of(&result)
        );

        // The geocoded IANA zone must reach the forecast call, or "Today"
        // means today here rather than today there.
        let forecast_url = mock.received_requests().await.unwrap()[1].url.to_string();
        assert!(forecast_url.contains("Australia%2FMelbourne"), "{forecast_url}");
    }
```

- [ ] **Step 10: Run it to verify it fails**

Run: `cd mcp-servers/open-meteo-mcp && cargo test server`
Expected: FAIL — the result is an error saying names are not supported.

- [ ] **Step 11: Wire the geocoder into `get_weather`**

In `src/server.rs`, replace the `Resolution::Name(_)` arm and thread the
timezone through. The destructured tuple gains a fourth element:

```rust
        let (place, latitude, longitude, timezone) = match resolve(
            args.location.as_deref(),
            args.latitude,
            args.longitude,
            &self.config,
        ) {
            Resolution::Ask => return Ok(error_text(ASK_USER)),
            Resolution::MissingHalf(missing) => {
                return Ok(error_text(format!(
                    "A coordinate pair needs both halves — {missing} is missing. \
Give both latitude and longitude, or a place name instead."
                )));
            }
            Resolution::Coords {
                latitude,
                longitude,
            } => (
                format!("{latitude:.4}, {longitude:.4}"),
                latitude,
                longitude,
                None,
            ),
            Resolution::Name(name) => match client.geocode(&name).await {
                Ok(found) => (
                    found.label(),
                    found.latitude,
                    found.longitude,
                    found.timezone.clone(),
                ),
                Err(e) => return Ok(error_text(e.to_string())),
            },
        };

        match client
            .forecast(latitude, longitude, clamped.days, timezone.as_deref())
            .await
        {
            Ok(forecast) => Ok(CallToolResult::success(vec![ContentBlock::text(render(
                &place,
                &forecast,
                clamped.note.as_deref(),
            ))])),
            Err(e) => Ok(error_text(e.to_string())),
        }
```

- [ ] **Step 12: Run the whole suite**

Run: `cd mcp-servers/open-meteo-mcp && cargo test`
Expected: PASS — **26 tests**: 3 in `config`, 6 in `location`, 7 in
`open_meteo`, 6 in `render`, 4 in `server` (the placeholder was replaced,
not added to).

- [ ] **Step 13: Run the gates**

Run: `cargo fmt --check && cargo clippy --all-targets -- -D warnings`
Expected: both clean.

- [ ] **Step 14: Commit**

```bash
git add mcp-servers/open-meteo-mcp/
git commit -m "feat(open-meteo-mcp): geocode a place name and speak the full forecast"
```

---

### Task 4: The config seams — units, API key, and a coordinate default

**Files:**
- Modify: `mcp-servers/open-meteo-mcp/src/config.rs` (add `Units` and `api_key`, and their tests)
- Modify: `mcp-servers/open-meteo-mcp/src/location.rs` (add `parse_default`, and its test)
- Modify: `mcp-servers/open-meteo-mcp/src/open_meteo.rs` (units in the request, key on both calls, and their tests)
- Modify: `mcp-servers/open-meteo-mcp/src/render.rs` (unit strings, and their test)
- Modify: `mcp-servers/open-meteo-mcp/src/server.rs` (pass units to `render`)
- Test: inline in all four

**Interfaces:**
- Consumes: everything from Tasks 1–3.
- Produces:
  - `config::Units` — `Metric | Imperial`
  - `config::Config` gains `units: Units` and `api_key: Option<String>`
  - `render::render(place: &str, forecast: &Forecast, units: &Units, clamp_note: Option<&str>) -> String` — **the signature changes**; the one call site is in `server.rs`

This is the commercial-tier escape hatch and the units decision, together,
because they are the same seam: the paid tier is API-identical to the free
one, so switching to it is entirely the URL and key variables that already
exist plus the one that does not yet.

**Units are configuration, not a tool argument.** A user's preferred units
are a property of the user, not of the question; making it an argument
invites the model to pick, which it will sometimes get wrong for no benefit.
Do not add it to `GetWeatherArgs`.

**Do not echo the API's unit strings.** In imperial mode Open-Meteo labels
wind `"mp/h"` — verified against the live endpoint. The rendered text says
`mph`, because that is what a person says. The response's `*_units` objects
are not deserialized at all.

- [ ] **Step 1: Write the failing config tests**

In `src/config.rs`, add to `mod tests`:

```rust
    #[test]
    fn units_default_to_metric_and_only_the_exact_word_imperial_switches_them() {
        assert_eq!(Config::from_vars(empty).units, Units::Metric);
        assert_eq!(
            Config::from_vars(|k| (k == "OPEN_METEO_UNITS").then(|| "imperial".to_string())).units,
            Units::Imperial
        );
        // A plausible-but-wrong value must not silently become imperial.
        assert_eq!(
            Config::from_vars(|k| (k == "OPEN_METEO_UNITS").then(|| "celsius".to_string())).units,
            Units::Metric
        );
    }

    #[test]
    fn an_empty_api_key_is_treated_as_unset() {
        let c = Config::from_vars(|k| (k == "OPEN_METEO_API_KEY").then(|| "  ".to_string()));
        assert!(c.api_key.is_none());
    }
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cd mcp-servers/open-meteo-mcp && cargo test config`
Expected: FAIL — `cannot find type 'Units' in this scope`, `no field
'api_key' on type 'Config'`.

- [ ] **Step 3: Add units and the key to `Config`**

In `src/config.rs`, add the enum above `Config`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Units {
    Metric,
    Imperial,
}
```

add two fields to `Config`:

```rust
    pub units: Units,
    pub api_key: Option<String>,
```

and in `from_vars`, before the `Config { .. }` literal:

```rust
        // Exact match only: a wrong value falls back to metric rather than
        // being guessed at, because the wrong units are a silent wrong answer.
        let units = match get("OPEN_METEO_UNITS").as_deref() {
            Some("imperial") => Units::Imperial,
            _ => Units::Metric,
        };
```

then add the two fields to the literal:

```rust
            units,
            api_key: get("OPEN_METEO_API_KEY").filter(|v| !v.trim().is_empty()),
```

- [ ] **Step 4: Run them to verify they pass**

Run: `cd mcp-servers/open-meteo-mcp && cargo test config`
Expected: PASS — 5 tests.

- [ ] **Step 5: Write the failing coordinate-default test**

In `src/location.rs`, add to `mod tests`:

```rust
    #[test]
    fn a_default_written_as_lat_lon_is_used_as_coordinates() {
        assert_eq!(
            resolve(None, None, None, &config(Some("-37.814, 144.9633"))),
            Resolution::Coords {
                latitude: -37.814,
                longitude: 144.9633
            }
        );
    }

    #[test]
    fn a_default_that_merely_contains_a_comma_is_still_a_place_name() {
        assert_eq!(
            resolve(None, None, None, &config(Some("Melbourne, Australia"))),
            Resolution::Name("Melbourne, Australia".into())
        );
    }
```

- [ ] **Step 6: Run them to verify they fail**

Run: `cd mcp-servers/open-meteo-mcp && cargo test location`
Expected: FAIL on the first — the default is currently always a `Name`, so
`"-37.814, 144.9633"` comes back as `Resolution::Name`. The second already
passes; that is fine, it is the guard that stops Step 7 overreaching.

- [ ] **Step 7: Parse a coordinate default**

In `src/location.rs`, replace the default arm inside `resolve`:

```rust
                Some(default) => parse_default(default),
```

and add the function below `resolve`:

```rust
/// The spec lets the default be "a place name or `lat,lon`". Both halves
/// must parse as numbers, so "Melbourne, Australia" stays a name.
fn parse_default(value: &str) -> Resolution {
    if let Some((lat, lon)) = value.split_once(',')
        && let (Ok(latitude), Ok(longitude)) =
            (lat.trim().parse::<f64>(), lon.trim().parse::<f64>())
    {
        return Resolution::Coords {
            latitude,
            longitude,
        };
    }
    Resolution::Name(value.trim().to_string())
}
```

- [ ] **Step 8: Run them to verify they pass**

Run: `cd mcp-servers/open-meteo-mcp && cargo test location`
Expected: PASS — 8 tests.

- [ ] **Step 9: Write the failing request-seam tests**

In `src/open_meteo.rs`, change the test helper `config` to take the two new
knobs. It has **seven** call sites by now — six `Client::new(config(&server))`
and the timeout test's `config: config(&server)` — and every one becomes
`config(&server, false, None)`:

```rust
    fn config(server: &MockServer, imperial: bool, key: Option<&str>) -> Config {
        let base = server.uri();
        Config::from_vars(move |k| match k {
            "OPEN_METEO_BASE_URL" => Some(format!("{base}/v1/forecast")),
            "OPEN_METEO_GEOCODING_URL" => Some(format!("{base}/v1/search")),
            "OPEN_METEO_UNITS" if imperial => Some("imperial".to_string()),
            "OPEN_METEO_API_KEY" => key.map(str::to_string),
            _ => None,
        })
    }
```

then add two tests:

```rust
    #[tokio::test]
    async fn imperial_units_reach_the_forecast_request() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/forecast"))
            .and(query_param("temperature_unit", "fahrenheit"))
            .and(query_param("wind_speed_unit", "mph"))
            .and(query_param("precipitation_unit", "inch"))
            .respond_with(ResponseTemplate::new(200).set_body_json(melbourne_forecast()))
            .mount(&server)
            .await;

        Client::new(config(&server, true, None))
            .forecast(28.08, -80.61, 1, None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn the_api_key_is_sent_when_set_and_absent_when_not() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(melbourne_place()))
            .mount(&server)
            .await;

        Client::new(config(&server, false, Some("k-123")))
            .geocode("Melbourne")
            .await
            .unwrap();
        Client::new(config(&server, false, None))
            .geocode("Melbourne")
            .await
            .unwrap();

        let requests = server.received_requests().await.unwrap();
        assert!(requests[0].url.as_str().contains("apikey=k-123"));
        assert!(!requests[1].url.as_str().contains("apikey"));
    }
```

- [ ] **Step 10: Run them to verify they fail**

Run: `cd mcp-servers/open-meteo-mcp && cargo test open_meteo`
Expected: FAIL — the imperial test times out matching (no mock matches a
request without those query params) and the key test's first assertion fails.

- [ ] **Step 11: Implement the request seams**

In `src/open_meteo.rs`, add the import and the helper to `impl Client`:

```rust
use crate::config::{Config, Units};
```

```rust
    /// Sent as a query parameter on both endpoints, and only when set.
    /// The paid tier is API-identical to the free one, so this plus the two
    /// URL variables is the whole of what switching to it requires.
    fn key_param(&self) -> Vec<(&str, String)> {
        match &self.config.api_key {
            Some(key) => vec![("apikey", key.clone())],
            None => vec![],
        }
    }
```

In `forecast`, make `query` mutable and append:

```rust
        let mut query = vec![
            // ...unchanged...
        ];
        if self.config.units == Units::Imperial {
            query.push(("temperature_unit", "fahrenheit".to_string()));
            query.push(("wind_speed_unit", "mph".to_string()));
            query.push(("precipitation_unit", "inch".to_string()));
        }
        query.extend(self.key_param());
```

In `geocode`, make `query` mutable and append:

```rust
        let mut query = vec![
            // ...unchanged...
        ];
        query.extend(self.key_param());
```

- [ ] **Step 12: Run them to verify they pass**

Run: `cd mcp-servers/open-meteo-mcp && cargo test open_meteo`
Expected: PASS — 9 tests.

- [ ] **Step 13: Write the failing render-units test**

In `src/render.rs`, add to `mod tests`:

```rust
    #[test]
    fn imperial_units_change_the_rendered_unit_strings() {
        let out = render("Melbourne", &forecast(), &Units::Imperial, None);
        assert!(out.contains("°F"), "{out}");
        assert!(out.contains("mph"), "{out}");
        assert!(out.contains("4in"), "{out}");
        assert!(!out.contains("°C"), "{out}");
    }
```

and update the **five** existing tests that call `render` to pass
`&Units::Metric` as the third argument:
`current_conditions_lead_and_read_as_spoken_words_not_codes`,
`every_successful_result_carries_the_attribution_line`,
`a_clamp_note_is_spoken_before_the_attribution`,
`a_rendered_forecast_reads_as_one_speakable_paragraph` and
`no_hourly_rows_appear_in_the_output` — whose `"°C"` count of 4 is
unaffected in metric. `an_unknown_wmo_code_still_produces_a_word` calls
`describe_code` directly and does not change.

- [ ] **Step 14: Run it to verify it fails**

Run: `cd mcp-servers/open-meteo-mcp && cargo test render`
Expected: FAIL to compile — `this function takes 3 arguments but 4
arguments were supplied`.

- [ ] **Step 15: Thread units through rendering**

In `src/render.rs`, add the import and the three helpers:

```rust
use crate::config::Units;
```

```rust
fn temp_unit(units: &Units) -> &'static str {
    match units {
        Units::Metric => "°C",
        Units::Imperial => "°F",
    }
}

/// `mph`, not the API's own `"mp/h"` — this string is spoken, not echoed.
fn wind_unit(units: &Units) -> &'static str {
    match units {
        Units::Metric => "km/h",
        Units::Imperial => "mph",
    }
}

fn precip_unit(units: &Units) -> &'static str {
    match units {
        Units::Metric => "mm",
        Units::Imperial => "in",
    }
}
```

change the signature and the two format strings:

```rust
pub fn render(place: &str, forecast: &Forecast, units: &Units, clamp_note: Option<&str>) -> String {
    let t = temp_unit(units);
    let mut out = format!(
        "{place} — currently {:.0}{t}, {}, wind {:.0} {}.",
        forecast.current.temperature_2m,
        describe_code(forecast.current.weather_code),
        forecast.current.wind_speed_10m,
        wind_unit(units),
    );
```

```rust
        let weather = if precip > 0.0 {
            format!(
                ", {} {precip:.0}{}",
                describe_code(days.weather_code[i]),
                precip_unit(units)
            )
        } else {
            format!(", {}", describe_code(days.weather_code[i]))
        };
        out.push_str(&format!(
            " {} {:.0}-{:.0}{t}{weather}.",
            day_label(&days.time[i], i),
            days.temperature_2m_min[i],
            days.temperature_2m_max[i],
        ));
```

- [ ] **Step 16: Update the one call site**

In `src/server.rs`, pass the configured units:

```rust
            Ok(forecast) => Ok(CallToolResult::success(vec![ContentBlock::text(render(
                &place,
                &forecast,
                &self.config.units,
                clamped.note.as_deref(),
            ))])),
```

- [ ] **Step 17: Run the whole suite**

Run: `cd mcp-servers/open-meteo-mcp && cargo test`
Expected: PASS — **33 tests**: 5 in `config`, 8 in `location`, 9 in
`open_meteo`, 7 in `render`, 4 in `server`.

- [ ] **Step 18: Run the gates**

Run: `cargo fmt --check && cargo clippy --all-targets -- -D warnings`
Expected: both clean.

- [ ] **Step 19: Commit**

```bash
git add mcp-servers/open-meteo-mcp/
git commit -m "feat(open-meteo-mcp): units, API key and a coordinate default"
```

---

### Task 5: `.mcpb` packaging

**Files:**
- Create: `mcp-servers/open-meteo-mcp/manifest.json`
- Create: `mcp-servers/open-meteo-mcp/scripts/package-mcpb.sh`
- Modify: `mcp-servers/open-meteo-mcp/src/main.rs` (one new test module)
- Test: inline in `main.rs`

**Interfaces:**
- Consumes: the built binary from `cargo build --release`.
- Produces: `open-meteo-mcp-<platform>.mcpb` in `mcp-servers/open-meteo-mcp/dist/`.

An `.mcpb` bundle is a zip of a `manifest.json` and the files it names. The
manifest below satisfies **the four checks UIA's installer applies** by
construction rather than by exemption. Those checks, read from
`crates/uia-mcp/src/bundle/validate.rs` in the `user-interface-agents` repo:

1. `server.type` must be exactly `"binary"`.
2. The resolved command must stay inside the bundle directory — `..`
   segments are folded textually and an escape is refused.
3. The command must not end in a script extension. The refused list is
   `py js mjs cjs ts rb sh bash ps1 bat cmd php pl` — note `exe` is **not**
   on it, so the Windows override is fine.
4. The command must be a file that exists and whose first two bytes are not
   `#!`.

A fifth, on the name: only `[A-Za-z0-9_-]` is allowed, because the server
name prefixes every tool name. `open-meteo-mcp` passes.

- [ ] **Step 1: Write the failing manifest test**

The manifest is data the installer will refuse if it is wrong, and nothing
else in the crate reads it — so the test is what keeps it honest. Add to
`src/main.rs`:

```rust
#[cfg(test)]
mod manifest_tests {
    /// The four checks `uia` applies to a bundle, asserted against the
    /// manifest this repo ships, so a careless edit fails here rather than
    /// at install time on somebody's machine.
    #[test]
    fn the_manifest_satisfies_uias_bundle_checks() {
        let raw = include_str!("../manifest.json");
        let m: serde_json::Value = serde_json::from_str(raw).expect("manifest.json is valid JSON");

        assert_eq!(m["server"]["type"], "binary");

        let name = m["name"].as_str().unwrap();
        assert!(
            name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
            "server names prefix tool names: {name}"
        );

        const SCRIPT_EXTENSIONS: &[&str] = &[
            "py", "js", "mjs", "cjs", "ts", "rb", "sh", "bash", "ps1", "bat", "cmd", "php", "pl",
        ];
        let mut commands = vec![m["server"]["mcp_config"]["command"].as_str().unwrap()];
        for (_, over) in m["server"]["mcp_config"]["platform_overrides"]
            .as_object()
            .unwrap()
        {
            commands.push(over["command"].as_str().unwrap());
        }

        for command in commands {
            assert!(
                command.starts_with("${__dirname}/"),
                "must be bundle-relative: {command}"
            );
            assert!(!command.contains(".."), "must not escape the bundle: {command}");
            let ext = command.rsplit_once('.').map(|(_, e)| e).unwrap_or("");
            assert!(
                !SCRIPT_EXTENSIONS.contains(&ext),
                "{command} ends in a script extension"
            );
        }
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cd mcp-servers/open-meteo-mcp && cargo test manifest`
Expected: FAIL to compile — `couldn't read ../manifest.json: No such file
or directory`.

- [ ] **Step 3: Write the manifest**

Create `mcp-servers/open-meteo-mcp/manifest.json`:

```json
{
  "manifest_version": "0.1",
  "name": "open-meteo-mcp",
  "display_name": "Open-Meteo weather",
  "version": "0.1.0",
  "description": "Current conditions and a short forecast for a place, from Open-Meteo.",
  "author": { "name": "MycorX" },
  "license": "Apache-2.0",
  "server": {
    "type": "binary",
    "entry_point": "bin/open-meteo-mcp",
    "mcp_config": {
      "command": "${__dirname}/bin/open-meteo-mcp",
      "args": [],
      "env": {},
      "platform_overrides": {
        "win32": { "command": "${__dirname}/bin/open-meteo-mcp.exe" }
      }
    }
  },
  "tools": [
    {
      "name": "get_weather",
      "description": "Current conditions and a short forecast for a place."
    }
  ]
}
```

`env` is left empty on purpose: every variable is optional, and shipping a
`OPEN_METEO_DEFAULT_LOCATION` here would put a location in the bundle that
the installing user never chose.

- [ ] **Step 4: Run it to verify it passes**

Run: `cd mcp-servers/open-meteo-mcp && cargo test manifest`
Expected: PASS — 1 test.

- [ ] **Step 5: Write the packaging script**

Create `mcp-servers/open-meteo-mcp/scripts/package-mcpb.sh`, and
`chmod +x` it:

```bash
#!/usr/bin/env bash
# Build a release binary and wrap it, with the manifest, into a .mcpb
# bundle for the platform this runs on. Cross-compiling is out of scope:
# each platform's bundle is built on that platform, in CI or by hand.
set -euo pipefail

cd "$(dirname "$0")/.."

case "$(uname -s)" in
  Linux*)  platform=linux;  binary=open-meteo-mcp ;;
  Darwin*) platform=darwin; binary=open-meteo-mcp ;;
  MINGW*|MSYS*|CYGWIN*) platform=win32; binary=open-meteo-mcp.exe ;;
  *) echo "unsupported platform: $(uname -s)" >&2; exit 1 ;;
esac

cargo build --release

staging=$(mktemp -d)
trap 'rm -rf "$staging"' EXIT
mkdir -p "$staging/bin"
cp "target/release/$binary" "$staging/bin/$binary"
cp manifest.json "$staging/manifest.json"

mkdir -p dist
out="$PWD/dist/open-meteo-mcp-$platform.mcpb"
rm -f "$out"
# -j would flatten bin/ into the root; the manifest names bin/, so the
# archive must keep the directory.
(cd "$staging" && zip -qr "$out" manifest.json bin)

echo "wrote $out"
```

- [ ] **Step 6: Verify the bundle by hand**

Run:

```bash
cd mcp-servers/open-meteo-mcp
./scripts/package-mcpb.sh
unzip -l dist/open-meteo-mcp-linux.mcpb
head -c 2 target/release/open-meteo-mcp | xxd
```

Expected: the listing shows `manifest.json` and `bin/open-meteo-mcp`, and
the first two bytes are `7f45` (ELF), not `2321` (`#!`). On a check-four
failure the installer would refuse the bundle, so confirm this rather than
assuming it.

- [ ] **Step 7: Run the gates**

Run: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`
Expected: all clean, **34 tests**.

- [ ] **Step 8: Commit**

`dist/` is build output — confirm `.gitignore` covers it before staging, and
add `mcp-servers/open-meteo-mcp/dist/` to the repo `.gitignore` if it does
not.

```bash
git status --short   # dist/ must not appear
git add mcp-servers/open-meteo-mcp/
git commit -m "feat(open-meteo-mcp): .mcpb manifest and packaging script"
```

---

### Task 6: README and the root table row

**Files:**
- Create: `mcp-servers/open-meteo-mcp/README.md`
- Modify: `README.md` (repo root — replace "Nothing here yet." under **MCP servers**)
- Test: `python3 scripts/validate_marketplace.py`

**Interfaces:**
- Consumes: everything above.
- Produces: nothing code-facing.

Per [`mcp-servers/README.md`](../../../mcp-servers/README.md), adding a
server means adding a row to the root table. This is the category's first
entry, so the "Nothing here yet." line is replaced by a table.

**The one thing this README must get right:** the free tier is
**non-commercial only**, and it must say so plainly, next to the
configuration that switches to a paid tier. That is not a legal requirement
— it is a matter of not setting a commercial user up to breach terms
without ever being told they were on them.

- [ ] **Step 1: Write the server README**

Create `mcp-servers/open-meteo-mcp/README.md`:

````markdown
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
````

- [ ] **Step 2: Add the root table row**

In the repo root `README.md`, replace:

```markdown
## MCP servers

Nothing here yet. See [`mcp-servers/`](mcp-servers) for how a new one gets added.
```

with:

```markdown
## MCP servers

Standalone MCP servers, usable by any MCP-capable agent. See
[`mcp-servers/`](mcp-servers) for how a new one gets added.

| Server | Description |
| --- | --- |
| [`open-meteo-mcp`](mcp-servers/open-meteo-mcp) | Current conditions and a short forecast for a place, from Open-Meteo. Free tier is non-commercial only. |
```

- [ ] **Step 3: Run the repo validator**

Run: `python3 scripts/validate_marketplace.py`
Expected: PASS. It validates the Claude Code plugin catalogue and README
links; it has no opinion about `mcp-servers/`, so a failure here means a
broken link was introduced, not that the server is wrong.

- [ ] **Step 4: Run the crate gates one last time**

Run: `cd mcp-servers/open-meteo-mcp && cargo test && cargo fmt --check && cargo clippy --all-targets -- -D warnings`
Expected: **34 tests**, all clean.

- [ ] **Step 5: Commit**

```bash
git add README.md mcp-servers/open-meteo-mcp/README.md
git commit -m "docs(open-meteo-mcp): README and the root table's first MCP server row"
```

---

## What is deliberately not here

- **Caching.** Weather is asked because it is current, and Open-Meteo's
  terms restrict what may be done with the data.
- **Open-Meteo's other endpoints** — archive, marine, air quality, flood,
  climate, ensemble. Each would be another tool the model must choose
  between, and none was asked for.
- **A separate geocoding tool.** Geocoding happens inside `get_weather`;
  exposing it would add a second routing decision for no caller.
- **Severe-weather alerts.** This server answers questions; it does not
  originate messages.
- **A CI workflow for this crate.** The repo runs gitleaks and the
  marketplace validator, neither of which builds Rust. A workflow that runs
  `cargo test`/`fmt`/`clippy` on `mcp-servers/**` is worth adding, but it is
  a repo-infrastructure change with its own blast radius — file it rather
  than folding it into Task 6.
- **Row B9, the UIA home-location seam.** Different repo, different branch,
  its own plan.
