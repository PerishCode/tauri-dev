use crate::output::{print_diagnostics, print_plan};
use tauri_dev_core::{DevState, Severity};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ParsedArgs {
    command: Vec<String>,
    config: Option<String>,
    format: OutputFormat,
}

pub fn version() -> &'static str {
    option_env!("TAURI_DEV_BUILD_VERSION").unwrap_or(concat!("v", env!("CARGO_PKG_VERSION")))
}

pub fn help_text() -> &'static str {
    "tauri-dev\n\n\
Tauri development orchestration for app, sidecar, socket, inspect, and diagnostics loops.\n\
It is a product-agnostic CLI: consumers provide explicit config and own product semantics.\n\n\
Commands:\n  \
  doctor --config <path> [--format text|json]\n  \
  inspect config --config <path> [--format text|json]\n  \
  plan --config <path> [--format text|json]\n  \
  help\n  \
  version\n\n\
Config:\n  \
  Use explicit --config <path>. No default config filename is reserved in this scaffold.\n\n\
Feedback:\n  \
  Report parser gaps, diagnostics noise, install issues, and missing Tauri-dev capabilities at:\n  \
  https://github.com/PerishCode/tauri-dev/issues"
}

pub fn run(args: Vec<String>) -> Result<(), String> {
    let parsed = parse(args)?;
    if parsed.command.is_empty() {
        print!("{help}", help = help_text());
        println!();
        return Ok(());
    }

    match parsed.command.as_slice() {
        [command] if command == "help" || command == "--help" || command == "-h" => {
            println!("{}", help_text());
            Ok(())
        }
        [command] if command == "version" || command == "--version" || command == "-V" => {
            println!("tauri-dev {}", version());
            Ok(())
        }
        [command] if command == "doctor" => {
            let state = load_state(&parsed)?;
            let diagnostics = state.diagnostics();
            print_diagnostics(&diagnostics, parsed.format)?;
            if diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == Severity::Error)
            {
                Err("tauri-dev doctor found configuration errors".to_string())
            } else {
                Ok(())
            }
        }
        [command] if command == "plan" => {
            let state = load_state(&parsed)?;
            print_plan(&state.execution_plan(), parsed.format)
        }
        [command, target] if command == "inspect" && target == "config" => {
            let state = load_state(&parsed)?;
            print_plan(&state.execution_plan(), parsed.format)
        }
        [command] if matches!(command.as_str(), "start" | "stop" | "restart" | "status") => {
            Err(format!(
                "{command} is reserved for lifecycle execution; use `plan --config <path>` in this scaffold"
            ))
        }
        _ => Err(format!(
            "unknown command: {}; run `tauri-dev help`",
            parsed.command.join(" ")
        )),
    }
}

fn load_state(parsed: &ParsedArgs) -> Result<DevState, String> {
    let config = parsed
        .config
        .as_ref()
        .ok_or_else(|| "--config <path> is required".to_string())?;
    DevState::from_config_file(config).map_err(|error| error.to_string())
}

fn parse(args: Vec<String>) -> Result<ParsedArgs, String> {
    let mut command = Vec::new();
    let mut config = None;
    let mut format = OutputFormat::Text;
    let mut args = args.into_iter();
    let _binary = args.next();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" => {
                config = Some(
                    args.next()
                        .ok_or_else(|| "--config requires a value".to_string())?,
                );
            }
            "--format" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--format requires a value".to_string())?;
                format = parse_format(&value)?;
            }
            value if value.starts_with("--config=") => {
                config = Some(value.trim_start_matches("--config=").to_string());
            }
            value if value.starts_with("--format=") => {
                format = parse_format(value.trim_start_matches("--format="))?;
            }
            value
                if value.starts_with('-')
                    && !matches!(value, "-h" | "--help" | "-V" | "--version") =>
            {
                return Err(format!("unknown option: {value}"));
            }
            value => command.push(value.to_string()),
        }
    }

    Ok(ParsedArgs {
        command,
        config,
        format,
    })
}

fn parse_format(value: &str) -> Result<OutputFormat, String> {
    match value {
        "text" => Ok(OutputFormat::Text),
        "json" => Ok(OutputFormat::Json),
        _ => Err(format!("unsupported output format: {value}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_exposes_issue_boundary() {
        let help = help_text();
        assert!(help.contains("https://github.com/PerishCode/tauri-dev/issues"));
        assert!(help.contains("--config <path>"));
    }

    #[test]
    fn parses_global_config_after_command() {
        let parsed = parse(vec![
            "tauri-dev".into(),
            "doctor".into(),
            "--config".into(),
            "examples/minimal.toml".into(),
            "--format=json".into(),
        ])
        .unwrap();

        assert_eq!(parsed.command, vec!["doctor"]);
        assert_eq!(parsed.config.as_deref(), Some("examples/minimal.toml"));
        assert_eq!(parsed.format, OutputFormat::Json);
    }

    #[test]
    fn parses_version_flags_as_commands() {
        let parsed = parse(vec!["tauri-dev".into(), "--version".into()]).unwrap();
        assert_eq!(parsed.command, vec!["--version"]);
    }
}
