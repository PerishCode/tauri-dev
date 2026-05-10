//! CLI verb implementations: lifecycle (start/stop/restart/status/list/reset)
//! and inspect (line-JSON IPC over a sidecar's `inspect_socket`).

use crate::cli::OutputFormat;
use serde_json::Value;
use sidecar_core::{
    discover_by_app_namespace, discover_by_namespace, inspect_send, signal_terminate, DevState,
    ExecutionPlan, InspectRequest, InspectResponse, SidecarPlan, SocketEndpoint, StampedProcess,
};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

const INSPECT_DEFAULT_TIMEOUT_SECS: u64 = 5;

pub(crate) fn start(state: &DevState, sidecar: Option<&str>) -> Result<(), String> {
    let plan = state.execution_plan();
    let targets = select_targets(&plan, sidecar)?;
    for target in targets {
        let existing = discover_by_app_namespace(&target.stamp.app, &target.stamp.namespace)
            .map_err(|err| format!("discovery failed for `{}`: {err}", target.name))?;
        if let Some(running) = existing.first() {
            return Err(format!(
                "sidecar `{}` is already running (pid {}); run `sidecar stop` first",
                target.name, running.pid
            ));
        }
        let pid = spawn_detached(state.config_path.parent(), target)?;
        println!("started {} pid={pid}", target.name);
    }
    Ok(())
}

pub(crate) fn stop(state: &DevState, sidecar: Option<&str>) -> Result<(), String> {
    let plan = state.execution_plan();
    let targets = select_targets(&plan, sidecar)?;
    let mut stopped_total = 0;
    for target in targets {
        let hits = discover_by_app_namespace(&target.stamp.app, &target.stamp.namespace)
            .map_err(|err| format!("discovery failed for `{}`: {err}", target.name))?;
        if hits.is_empty() {
            println!("not running: {}", target.name);
            continue;
        }
        for hit in &hits {
            signal_terminate(hit.pid).map_err(|err| {
                format!(
                    "failed to terminate sidecar `{}` (pid {}): {err}",
                    target.name, hit.pid
                )
            })?;
            println!("stopped {} pid={}", target.name, hit.pid);
            stopped_total += 1;
        }
    }
    if stopped_total == 0 && sidecar.is_none() {
        println!("no sidecars were running");
    }
    Ok(())
}

pub(crate) fn restart(state: &DevState, sidecar: Option<&str>) -> Result<(), String> {
    stop(state, sidecar)?;
    start(state, sidecar)
}

pub(crate) fn status(state: &DevState, format: OutputFormat) -> Result<(), String> {
    let plan = state.execution_plan();
    let mut rows = Vec::new();
    for sidecar in &plan.sidecars {
        let hits = discover_by_app_namespace(&sidecar.stamp.app, &sidecar.stamp.namespace)
            .map_err(|err| format!("discovery failed for `{}`: {err}", sidecar.name))?;
        rows.push((sidecar.name.clone(), hits));
    }
    print_status(&plan.namespace, &rows, format)
}

pub(crate) fn list(state: &DevState, format: OutputFormat) -> Result<(), String> {
    let plan = state.execution_plan();
    let hits = discover_by_namespace(&plan.namespace)
        .map_err(|err| format!("discovery failed for namespace `{}`: {err}", plan.namespace))?;
    print_list(&plan.namespace, &hits, format)
}

pub(crate) fn reset(state: &DevState) -> Result<(), String> {
    let plan = state.execution_plan();
    let hits = discover_by_namespace(&plan.namespace)
        .map_err(|err| format!("discovery failed for namespace `{}`: {err}", plan.namespace))?;
    if hits.is_empty() {
        println!("namespace `{}` has no stamped processes", plan.namespace);
        return Ok(());
    }
    for hit in &hits {
        signal_terminate(hit.pid)
            .map_err(|err| format!("failed to terminate pid {}: {err}", hit.pid))?;
        println!("terminated pid={} cmd={}", hit.pid, hit.command);
    }
    Ok(())
}

pub(crate) fn inspect(
    state: &DevState,
    sidecar: &str,
    event: &str,
    payload: Option<&str>,
    format: OutputFormat,
) -> Result<(), String> {
    let plan = state.execution_plan();
    let target = plan
        .sidecars
        .iter()
        .find(|item| item.name == sidecar)
        .ok_or_else(|| format!("unknown sidecar `{sidecar}` in this manifest"))?;
    let socket = target.inspect_socket.as_deref().ok_or_else(|| {
        format!("sidecar `{sidecar}` has no inspect_socket configured in this manifest")
    })?;
    let endpoint = SocketEndpoint::parse(socket).map_err(|err| err.to_string())?;
    let payload_value: Value = match payload {
        Some(text) if !text.is_empty() => serde_json::from_str(text).map_err(|err| {
            format!("payload is not valid JSON: {err}; quote the payload as a single argument")
        })?,
        _ => Value::Null,
    };
    let request = InspectRequest {
        event: event.to_string(),
        payload: payload_value,
    };
    let response = inspect_send(
        &endpoint,
        &request,
        Some(Duration::from_secs(INSPECT_DEFAULT_TIMEOUT_SECS)),
    )?;
    print_inspect_response(sidecar, event, &response, format)
}

fn select_targets<'plan>(
    plan: &'plan ExecutionPlan,
    sidecar: Option<&str>,
) -> Result<Vec<&'plan SidecarPlan>, String> {
    if let Some(name) = sidecar {
        let hit = plan
            .sidecars
            .iter()
            .find(|item| item.name == name)
            .ok_or_else(|| format!("unknown sidecar `{name}` in this manifest"))?;
        Ok(vec![hit])
    } else {
        if plan.sidecars.is_empty() {
            return Err("manifest declares no sidecars".to_string());
        }
        Ok(plan.sidecars.iter().collect())
    }
}

fn spawn_detached(config_dir: Option<&Path>, target: &SidecarPlan) -> Result<u32, String> {
    let cwd = resolve_cwd(config_dir, &target.cwd);
    let child = Command::new(&target.command)
        .args(target.spawn_args())
        .current_dir(&cwd)
        .spawn()
        .map_err(|err| format!("failed to spawn `{}`: {err}", target.command))?;
    Ok(child.id())
}

fn resolve_cwd(config_dir: Option<&Path>, cwd: &str) -> std::path::PathBuf {
    let path = std::path::Path::new(cwd);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    match config_dir {
        Some(dir) => dir.join(path),
        None => path.to_path_buf(),
    }
}

fn print_status(
    namespace: &str,
    rows: &[(String, Vec<StampedProcess>)],
    format: OutputFormat,
) -> Result<(), String> {
    match format {
        OutputFormat::Text => {
            println!("namespace: {namespace}");
            for (name, hits) in rows {
                if let Some(first) = hits.first() {
                    println!("- {name}: running (pid {})", first.pid);
                    for extra in hits.iter().skip(1) {
                        println!("  + duplicate (pid {})", extra.pid);
                    }
                } else {
                    println!("- {name}: stopped");
                }
            }
            Ok(())
        }
        OutputFormat::Json => {
            let value = serde_json::json!({
                "namespace": namespace,
                "sidecars": rows.iter().map(|(name, hits)| serde_json::json!({
                    "name": name,
                    "running": !hits.is_empty(),
                    "pids": hits.iter().map(|hit| hit.pid).collect::<Vec<_>>(),
                })).collect::<Vec<_>>(),
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&value).map_err(|err| err.to_string())?
            );
            Ok(())
        }
    }
}

fn print_list(
    namespace: &str,
    hits: &[StampedProcess],
    format: OutputFormat,
) -> Result<(), String> {
    match format {
        OutputFormat::Text => {
            println!("namespace: {namespace}");
            if hits.is_empty() {
                println!("no stamped processes");
                return Ok(());
            }
            for hit in hits {
                println!("- pid={} cmd={}", hit.pid, hit.command);
            }
            Ok(())
        }
        OutputFormat::Json => {
            let value = serde_json::json!({
                "namespace": namespace,
                "processes": hits.iter().map(|hit| serde_json::json!({
                    "pid": hit.pid,
                    "command": hit.command,
                })).collect::<Vec<_>>(),
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&value).map_err(|err| err.to_string())?
            );
            Ok(())
        }
    }
}

fn print_inspect_response(
    sidecar: &str,
    event: &str,
    response: &InspectResponse,
    format: OutputFormat,
) -> Result<(), String> {
    match format {
        OutputFormat::Text => match response {
            InspectResponse::Ok(value) => {
                println!("ok {sidecar} {event}");
                println!(
                    "{}",
                    serde_json::to_string_pretty(value).unwrap_or_default()
                );
                Ok(())
            }
            InspectResponse::Err(message) => Err(format!("inspect error: {message}")),
        },
        OutputFormat::Json => {
            let body = match response {
                InspectResponse::Ok(value) => serde_json::json!({
                    "sidecar": sidecar,
                    "event": event,
                    "ok": true,
                    "data": value,
                }),
                InspectResponse::Err(message) => serde_json::json!({
                    "sidecar": sidecar,
                    "event": event,
                    "ok": false,
                    "error": message,
                }),
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&body).map_err(|err| err.to_string())?
            );
            if matches!(response, InspectResponse::Err(_)) {
                return Err("inspect endpoint returned ok=false".to_string());
            }
            Ok(())
        }
    }
}
