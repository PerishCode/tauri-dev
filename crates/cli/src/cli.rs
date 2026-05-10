use crate::commands;
use crate::output::{print_diagnostics, print_plan};
use sidecar_core::{DevState, Severity};

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
    option_env!("SIDECAR_BUILD_VERSION").unwrap_or(concat!("v", env!("CARGO_PKG_VERSION")))
}

pub fn help_text() -> &'static str {
    "sidecar\n\n\
IPC-based sidecars project manager. Stamp args + inspect bridge over Unix sockets.\n\
Product-agnostic: consumers provide explicit config and own product semantics.\n\n\
Commands:\n  \
  doctor   --config <path> [--format text|json]\n  \
  plan     --config <path> [--format text|json]\n  \
  inspect  config --config <path> [--format text|json]\n  \
  inspect  <sidecar> <event> [<json-payload>] --config <path> [--format text|json]\n  \
  start    --config <path> [<sidecar>]\n  \
  restart  --config <path> [<sidecar>]\n  \
  stop     --config <path> [<sidecar>]\n  \
  status   --config <path> [--format text|json]\n  \
  list     --config <path> [--format text|json]\n  \
  reset    --config <path>\n  \
  help\n  \
  version\n\n\
Config:\n  \
  Use explicit --config <path>. No default config filename is reserved.\n\n\
Feedback:\n  \
  Report parser gaps, diagnostics noise, install issues, and missing capabilities at:\n  \
  https://github.com/PerishCode/sidecar/issues"
}

pub fn run(args: Vec<String>) -> Result<(), String> {
    let parsed = parse(args)?;
    if parsed.command.is_empty() {
        print!("{help}", help = help_text());
        println!();
        return Ok(());
    }

    let cmd = parsed.command[0].as_str();
    match cmd {
        "help" | "--help" | "-h" => {
            println!("{}", help_text());
            Ok(())
        }
        "version" | "--version" | "-V" => {
            println!("sidecar {}", version());
            Ok(())
        }
        "doctor" => {
            require_no_extra_args(&parsed, 1, "doctor")?;
            let state = load_state(&parsed)?;
            let diagnostics = state.diagnostics();
            print_diagnostics(&diagnostics, parsed.format)?;
            if diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == Severity::Error)
            {
                Err("sidecar doctor found configuration errors".to_string())
            } else {
                Ok(())
            }
        }
        "plan" => {
            require_no_extra_args(&parsed, 1, "plan")?;
            let state = load_state(&parsed)?;
            print_plan(&state.execution_plan(), parsed.format)
        }
        "inspect" => match parsed.command.len() {
            1 => Err("inspect requires `config` or `<sidecar> <event> [payload]`".to_string()),
            _ if parsed.command[1] == "config" => {
                require_no_extra_args(&parsed, 2, "inspect config")?;
                let state = load_state(&parsed)?;
                print_plan(&state.execution_plan(), parsed.format)
            }
            len => {
                if len < 3 {
                    return Err(
                        "inspect <sidecar> <event> [payload] — event is required".to_string()
                    );
                }
                if len > 4 {
                    return Err(format!(
                        "unsupported inspect arguments: {}",
                        parsed.command[4..].join(" ")
                    ));
                }
                let state = load_state(&parsed)?;
                let payload = parsed.command.get(3).map(String::as_str);
                commands::inspect(
                    &state,
                    &parsed.command[1],
                    &parsed.command[2],
                    payload,
                    parsed.format,
                )
            }
        },
        "start" | "stop" | "restart" => {
            let target = optional_target(&parsed, cmd)?;
            let state = load_state(&parsed)?;
            match cmd {
                "start" => commands::start(&state, target),
                "stop" => commands::stop(&state, target),
                "restart" => commands::restart(&state, target),
                _ => unreachable!(),
            }
        }
        "status" => {
            require_no_extra_args(&parsed, 1, "status")?;
            let state = load_state(&parsed)?;
            commands::status(&state, parsed.format)
        }
        "list" => {
            require_no_extra_args(&parsed, 1, "list")?;
            let state = load_state(&parsed)?;
            commands::list(&state, parsed.format)
        }
        "reset" => {
            require_no_extra_args(&parsed, 1, "reset")?;
            let state = load_state(&parsed)?;
            commands::reset(&state)
        }
        _ => Err(format!(
            "unknown command: {}; run `sidecar help`",
            parsed.command.join(" ")
        )),
    }
}

fn optional_target<'a>(parsed: &'a ParsedArgs, command: &str) -> Result<Option<&'a str>, String> {
    match parsed.command.len() {
        1 => Ok(None),
        2 => Ok(Some(parsed.command[1].as_str())),
        _ => Err(format!(
            "unsupported {command} arguments: {}",
            parsed.command[2..].join(" ")
        )),
    }
}

fn require_no_extra_args(
    parsed: &ParsedArgs,
    expected_len: usize,
    command: &str,
) -> Result<(), String> {
    if parsed.command.len() > expected_len {
        return Err(format!(
            "unsupported {command} arguments: {}",
            parsed.command[expected_len..].join(" ")
        ));
    }
    Ok(())
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
        assert!(help.contains("https://github.com/PerishCode/sidecar/issues"));
        assert!(help.contains("--config <path>"));
    }

    #[test]
    fn parses_global_config_after_command() {
        let parsed = parse(vec![
            "sidecar".into(),
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
        let parsed = parse(vec!["sidecar".into(), "--version".into()]).unwrap();
        assert_eq!(parsed.command, vec!["--version"]);
    }

    #[test]
    fn parses_inspect_with_payload_argument() {
        let parsed = parse(vec![
            "sidecar".into(),
            "inspect".into(),
            "controller".into(),
            "host".into(),
            "{\"window\":\"main\"}".into(),
            "--config".into(),
            "x.toml".into(),
        ])
        .unwrap();
        assert_eq!(
            parsed.command,
            vec!["inspect", "controller", "host", "{\"window\":\"main\"}"]
        );
        assert_eq!(parsed.config.as_deref(), Some("x.toml"));
    }
}
