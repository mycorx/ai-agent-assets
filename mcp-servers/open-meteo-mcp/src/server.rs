//! The MCP surface: one tool, `get_weather`.

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
// Temporary: fields are only taken by reference in the Task-1 stub
// `get_weather`, never field-accessed, until Task 2 reads them.
#[allow(dead_code)]
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
