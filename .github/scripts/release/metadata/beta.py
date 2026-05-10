#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path


STABLE_RE = re.compile(r"^(\d+)\.(\d+)\.(\d+)$")
BETA_RE = re.compile(r"^v?(\d+\.\d+\.\d+)-beta\.([1-9][0-9]*)$")


def fail(message: str) -> None:
    print(f"[release-beta] {message}", file=sys.stderr)
    raise SystemExit(1)


def version_tuple(value: str) -> tuple[int, int, int]:
    match = STABLE_RE.match(value)
    if match is None:
        fail(f"expected stable x.y.z version, got {value}")
    return (int(match.group(1)), int(match.group(2)), int(match.group(3)))


def read_cargo_version() -> str:
    cargo_toml = Path("crates/cli/Cargo.toml")
    match = re.search(r'^version = "([^"]+)"$', cargo_toml.read_text(encoding="utf-8"), re.M)
    if match is None:
        fail(f"missing version in {cargo_toml}")
    version = match.group(1)
    version_tuple(version)
    return version


def parse_beta(value: str, source: str) -> tuple[str, int, str]:
    match = BETA_RE.match(value)
    if match is None:
        fail(f"{source} must look like vX.Y.Z-beta.N, got {value}")
    base_version = match.group(1)
    beta_number = int(match.group(2))
    return base_version, beta_number, f"v{base_version}-beta.{beta_number}"


def output(name: str, value: str) -> None:
    output_path = os.environ.get("GITHUB_OUTPUT")
    if output_path:
        with open(output_path, "a", encoding="utf-8") as handle:
            handle.write(f"{name}={value}\n")


def fetch_metadata_from_r2() -> str | None:
    bucket = os.environ.get("SIDECAR_RELEASES_S3_BUCKET")
    endpoint = os.environ.get("SIDECAR_RELEASES_S3_URL")
    access_key = os.environ.get("SIDECAR_RELEASES_S3_AK")
    secret_key = os.environ.get("SIDECAR_RELEASES_S3_SK")
    for name, value in (
        ("SIDECAR_RELEASES_S3_BUCKET", bucket),
        ("SIDECAR_RELEASES_S3_URL", endpoint),
        ("SIDECAR_RELEASES_S3_AK", access_key),
        ("SIDECAR_RELEASES_S3_SK", secret_key),
    ):
        if not value:
            fail(f"{name} is required")

    env = {
        **os.environ,
        "AWS_ACCESS_KEY_ID": access_key,
        "AWS_SECRET_ACCESS_KEY": secret_key,
        "AWS_DEFAULT_REGION": "auto",
        "AWS_EC2_METADATA_DISABLED": "true",
    }
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as handle:
        target = Path(handle.name)
    try:
        result = subprocess.run(
            [
                "aws",
                "--endpoint-url",
                endpoint.rstrip("/"),
                "s3api",
                "get-object",
                "--bucket",
                bucket,
                "--key",
                "beta/latest/metadata.json",
                "--no-cli-pager",
                str(target),
            ],
            env=env,
            capture_output=True,
            text=True,
        )
        if result.returncode == 0:
            return target.read_text(encoding="utf-8")
        stderr = result.stderr.strip()
        if "NoSuchKey" in stderr or "Not Found" in stderr or "404" in stderr:
            return None
        fail(f"failed to read R2 beta metadata via s3api: {stderr}")
    finally:
        target.unlink(missing_ok=True)


def read_metadata_beta(metadata: dict[str, object]) -> tuple[str, int, str]:
    value = metadata.get("betaVersion") or metadata.get("releaseVersion")
    if isinstance(value, str) and value:
        return parse_beta(value, "R2 beta metadata")

    base_version = metadata.get("baseVersion")
    beta_number = metadata.get("betaNumber")
    if isinstance(base_version, str) and isinstance(beta_number, int):
        version_tuple(base_version)
        if beta_number < 1:
            fail(f"R2 beta metadata betaNumber must be >= 1, got {beta_number}")
        return base_version, beta_number, f"v{base_version}-beta.{beta_number}"

    fail("R2 beta metadata must include betaVersion or releaseVersion")


def next_beta(cargo_version: str) -> tuple[str, int, str, str]:
    print("[release-beta] reading beta/latest/metadata.json from R2 via s3api")
    text = fetch_metadata_from_r2()
    if text is None:
        print("[release-beta] R2 beta metadata not found; starting beta.1")
        return cargo_version, 1, f"v{cargo_version}-beta.1", "missing R2 beta metadata"

    try:
        metadata = json.loads(text)
    except json.JSONDecodeError as error:
        fail(f"R2 beta metadata is invalid JSON: {error}")
    if not isinstance(metadata, dict):
        fail("R2 beta metadata must be a JSON object")

    base_version, beta_number, beta_version = read_metadata_beta(metadata)
    ordering = (version_tuple(cargo_version) > version_tuple(base_version)) - (
        version_tuple(cargo_version) < version_tuple(base_version)
    )
    if ordering < 0:
        fail(f"Cargo version {cargo_version} regressed below beta base {base_version}")
    if ordering > 0:
        return cargo_version, 1, f"v{cargo_version}-beta.1", "R2 beta metadata base advanced"
    return cargo_version, beta_number + 1, f"v{cargo_version}-beta.{beta_number + 1}", (
        f"R2 beta metadata {beta_version}"
    )


def main() -> None:
    cargo_version = read_cargo_version()
    override = os.environ.get("BETA_VERSION_OVERRIDE", "").strip()
    if override:
        base_version, beta_number, beta_version = parse_beta(override, "BETA_VERSION_OVERRIDE")
        if base_version != cargo_version:
            fail(f"override base {base_version} does not match Cargo version {cargo_version}")
        state_source = "workflow override"
    else:
        base_version, beta_number, beta_version, state_source = next_beta(cargo_version)

    print("[release-beta] channel: beta")
    print(f"[release-beta] base version: {base_version}")
    print(f"[release-beta] beta number: {beta_number}")
    print(f"[release-beta] beta version: {beta_version}")
    print(f"[release-beta] state source: {state_source}")

    output("base_version", base_version)
    output("beta_number", str(beta_number))
    output("beta_version", beta_version)
    output("release_version", beta_version)
    output("state_source", state_source)


if __name__ == "__main__":
    main()
