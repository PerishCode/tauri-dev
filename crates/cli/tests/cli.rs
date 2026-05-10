use std::process::Command;

#[test]
fn doctor_accepts_minimal_config() {
    let output = Command::new(env!("CARGO_BIN_EXE_tauri-dev"))
        .args(["doctor", "--config", "../../examples/minimal.toml"])
        .output()
        .expect("run tauri-dev doctor");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn plan_outputs_project() {
    let output = Command::new(env!("CARGO_BIN_EXE_tauri-dev"))
        .args([
            "plan",
            "--config",
            "../../examples/minimal.toml",
            "--format=json",
        ])
        .output()
        .expect("run tauri-dev plan");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"project\": \"example-tauri-app\""));
}
