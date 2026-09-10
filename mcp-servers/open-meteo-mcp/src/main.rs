mod config;
mod location;
mod open_meteo;
mod render;
mod server;

use rmcp::ServiceExt;
use rmcp::transport::stdio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // stderr, never stdout: stdout is the MCP transport, and one stray
    // line on it corrupts the JSON-RPC stream.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let service = server::WeatherServer::new(config::Config::from_env())
        .serve(stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod manifest_tests {
    const SCRIPT_EXTENSIONS: &[&str] = &[
        "py", "js", "mjs", "cjs", "ts", "rb", "sh", "bash", "ps1", "bat", "cmd", "php", "pl",
    ];

    /// Mirrors `validate.rs`'s own case-insensitive extension check: a
    /// manifest naming `bin/server.PY` must be caught here, not at install.
    fn ends_in_script_extension(command: &str) -> bool {
        let ext = command.rsplit_once('.').map(|(_, e)| e).unwrap_or("");
        SCRIPT_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str())
    }

    #[test]
    fn detects_uppercase_script_extensions_like_uias_validator() {
        assert!(ends_in_script_extension("${__dirname}/bin/server.PY"));
        assert!(ends_in_script_extension("${__dirname}/bin/server.Sh"));
        assert!(!ends_in_script_extension("${__dirname}/bin/open-meteo-mcp"));
    }

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
            name.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
            "server names prefix tool names: {name}"
        );

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
            assert!(
                !command.contains(".."),
                "must not escape the bundle: {command}"
            );
            assert!(
                !ends_in_script_extension(command),
                "{command} ends in a script extension"
            );
        }
    }
}
