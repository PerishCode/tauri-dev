use crate::cli::OutputFormat;
use tauri_dev_core::{Diagnostic, ExecutionPlan, Severity};

pub(crate) fn print_diagnostics(
    diagnostics: &[Diagnostic],
    format: OutputFormat,
) -> Result<(), String> {
    match format {
        OutputFormat::Text => {
            if diagnostics.is_empty() {
                println!("tauri-dev doctor found no issues");
                return Ok(());
            }

            println!("tauri-dev doctor found {} issue(s)", diagnostics.len());
            for diagnostic in diagnostics {
                let severity = match diagnostic.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                };
                println!("{severity} {} - {}", diagnostic.path, diagnostic.message);
            }
            Ok(())
        }
        OutputFormat::Json => {
            let items: Vec<_> = diagnostics
                .iter()
                .map(|diagnostic| {
                    let severity = match diagnostic.severity {
                        Severity::Error => "error",
                        Severity::Warning => "warning",
                    };
                    serde_json::json!({
                        "severity": severity,
                        "path": diagnostic.path,
                        "message": diagnostic.message,
                    })
                })
                .collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({ "diagnostics": items }))
                    .map_err(|error| error.to_string())?
            );
            Ok(())
        }
    }
}

pub(crate) fn print_plan(plan: &ExecutionPlan, format: OutputFormat) -> Result<(), String> {
    match format {
        OutputFormat::Text => {
            println!("project: {}", plan.project);
            match &plan.app {
                Some(app) => println!(
                    "app: {} -> {}",
                    app.name,
                    command_line(&app.command, &app.args)
                ),
                None => println!("app: <none>"),
            }
            println!("sidecars: {}", plan.sidecars.len());
            for sidecar in &plan.sidecars {
                println!(
                    "- {} -> {}",
                    sidecar.name,
                    command_line(&sidecar.command, &sidecar.args)
                );
            }
            println!("inspect endpoints: {}", plan.inspect_endpoints.len());
            for endpoint in &plan.inspect_endpoints {
                println!("- {} {} {}", endpoint.name, endpoint.kind, endpoint.url);
            }
            Ok(())
        }
        OutputFormat::Json => {
            let value = serde_json::json!({
                "project": plan.project,
                "app": plan.app.as_ref().map(|app| serde_json::json!({
                    "name": app.name,
                    "command": app.command,
                    "args": app.args,
                    "cwd": app.cwd,
                    "healthUrl": app.health_url,
                })),
                "sidecars": plan.sidecars.iter().map(|sidecar| serde_json::json!({
                    "name": sidecar.name,
                    "command": sidecar.command,
                    "args": sidecar.args,
                    "cwd": sidecar.cwd,
                    "socket": sidecar.socket,
                    "healthUrl": sidecar.health_url,
                })).collect::<Vec<_>>(),
                "inspectEndpoints": plan.inspect_endpoints.iter().map(|endpoint| serde_json::json!({
                    "name": endpoint.name,
                    "kind": endpoint.kind,
                    "url": endpoint.url,
                })).collect::<Vec<_>>(),
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&value).map_err(|error| error.to_string())?
            );
            Ok(())
        }
    }
}

fn command_line(command: &str, args: &[String]) -> String {
    std::iter::once(command)
        .chain(args.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ")
}
