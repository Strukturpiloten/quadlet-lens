#!/usr/bin/env bash

set -euo pipefail

script_directory="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
repository_root="$(cd -- "${script_directory}/.." && pwd -P)"
tooling_root="/opt/quadlet-lens-tooling"

for cargo_directory_name in CARGO_HOME CARGO_TARGET_DIR; do
  cargo_directory="${!cargo_directory_name:-}"
  if [[ -z "${cargo_directory}" ]]; then
    printf 'QuadletLens Dev Container is missing %s.\n' "${cargo_directory_name}" >&2
    exit 1
  fi
  if [[ ! -w "${cargo_directory}" ]]; then
    sudo chown -R "$(id -u):$(id -g)" "${cargo_directory}"
  fi
  if [[ ! -w "${cargo_directory}" ]]; then
    printf 'QuadletLens Dev Container cannot make %s writable: %s\n' \
      "${cargo_directory_name}" "${cargo_directory}" >&2
    exit 1
  fi
done

tools=(
  actionlint
  cargo
  cargo-clippy
  cargo-deny
  cargo-llvm-cov
  cargo-semver-checks
  curl
  gh
  git
  hadolint
  jq
  lychee
  markdownlint-cli2
  node
  npm
  prettier
  python3
  ruff
  rustc
  rustfmt
  rustup
  shellcheck
  shfmt
  tombi
  ty
  uv
  zizmor
)

for tool in "${tools[@]}"; do
  if ! command -v "${tool}" > /dev/null 2>&1; then
    printf 'QuadletLens Dev Container is missing required tool: %s\n' "${tool}" >&2
    exit 1
  fi
done

if ! rustup component list --installed | grep -q '^llvm-tools-'; then
  printf 'QuadletLens Dev Container is missing Rust component: llvm-tools-preview\n' >&2
  exit 1
fi

# Verify selected Python tools, declared Node and the candidate compiler.
python3 - "${repository_root}" "${tooling_root}" << 'PYTHON'
import json
from pathlib import Path
import re
import subprocess
import sys
import tomllib

try:
    repository, tooling = map(Path, sys.argv[1:])
    selected = json.loads((repository / ".github/automation.json").read_text())
    retained = json.loads((tooling / ".github/automation.json").read_text())
    revision = selected["revision"]
    if (
        selected != retained
        or selected.get("repository") != "Strukturpiloten/boxferry"
        or selected.get("interface") != 1
        or re.fullmatch(r"[a-f0-9]{40}", revision) is None
    ):
        raise ValueError("retained shared authority differs from the selected immutable source")
    project = tooling / "target/boxferry-automation" / revision / "automation"
    metadata = tomllib.loads((project / "pyproject.toml").read_text())
    requirement = metadata["tool"]["uv"]["required-version"]
    if not requirement.startswith("=="):
        raise ValueError("selected uv requirement must be exact")
    versions = {"uv": requirement[2:]}
    for name in ("ruff", "ty"):
        declarations = [
            entry.removeprefix(name + "==")
            for entry in metadata["dependency-groups"]["dev"]
            if entry.startswith(name + "==")
        ]
        if len(declarations) != 1:
            raise ValueError(f"selected project needs one exact {name} declaration")
        versions[name] = declarations[0]
    image = (repository / ".devcontainer/Dockerfile").read_text()
    nodes = list(re.finditer(
        r"(?m)^# release: (?P<tag>(?P<version>[0-9]+\.[0-9]+\.[0-9]+)-[^\s]+)\n"
        r"FROM docker\.io/library/node:(?P=tag)@sha256:[a-f0-9]{64} AS node_toolchain$",
        image,
    ))
    if len(nodes) != 1:
        raise ValueError("image must declare one exact tagged Node release matching its annotation")
    versions["node"] = nodes[0].group("version")
    for name, expected in versions.items():
        if re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", expected) is None:
            raise ValueError(f"selected {name} version must be exact")
        actual = subprocess.run(
            [name, "--version"], check=True, capture_output=True, text=True, timeout=30
        ).stdout.strip()
        pattern = "v" + re.escape(expected) if name == "node" else (
            re.escape(name + " " + expected) + r"(?: \([^\r\n]+\))?"
        )
        if re.fullmatch(pattern, actual) is None:
            raise ValueError(f"{name} executable differs from its selected version")
        subprocess.run(
            [name, "--help"], check=True, stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE, timeout=30,
        )
    installation = Path("/opt/quadlet-lens-file-tools")
    locked = json.loads((installation / "package-lock.json").read_text())["packages"]
    for name in ("prettier", "markdownlint-cli2"):
        command = [
            "python3", str(installation / "boxferry-node-tools.py"), "image-node-tool",
            "--repository-root", str(repository), "--cwd", str(repository), name,
        ]
        version = subprocess.run(
            [*command, "--version"], check=True, capture_output=True, text=True, timeout=30,
        ).stdout.strip()
        expected = locked["node_modules/" + name]["version"]
        if name == "prettier" and version != expected:
            raise ValueError(f"{name} differs from the retained selected lock")
        # Keep the real CLI help invocation: markdownlint-cli2 includes its
        # resolved markdownlint library version in this version-sensitive probe.
        help_result = subprocess.run(
            [*command, "--help"], check=False, capture_output=True, text=True, timeout=30,
        )
        help_status = 2 if name == "markdownlint-cli2" else 0
        if help_result.returncode != help_status or help_result.stderr:
            raise ValueError(f"{name} help status or stderr differs from its CLI contract")
        if name == "markdownlint-cli2":
            dependency = "node_modules/markdownlint-cli2/node_modules/markdownlint"
            library = locked.get(dependency, locked.get("node_modules/markdownlint"))
            lines = help_result.stdout[:8192].splitlines()
            banner = next((line for line in lines if line), "")
            usage = next((line for line in lines if line.startswith(
                "Syntax: markdownlint-cli2 glob0 [glob1] "
            )), "")
            if library is None or (
                banner != f"markdownlint-cli2 v{expected} (markdownlint v{library['version']})"
                or "https://github.com/DavidAnson/markdownlint-cli2" not in lines
                or "[--help]" not in usage
                or "[--no-globs]" not in usage
            ):
                raise ValueError("markdownlint-cli2 help differs from the selected lock")
    channel = tomllib.loads((repository / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
    compiler = subprocess.run(
        ["rustup", "run", channel, "rustc", "--version"], check=True,
        capture_output=True, text=True, timeout=30,
    ).stdout.strip()
    if not compiler.startswith("rustc " + channel + " "):
        raise ValueError("installed Rust compiler differs from candidate toolchain")
except (KeyError, OSError, TypeError, ValueError, subprocess.SubprocessError) as error:
    print(f"QuadletLens Dev Container tool verification failed: {error}", file=sys.stderr)
    raise SystemExit(1) from None
PYTHON

printf 'QuadletLens Dev Container tooling is ready.\n'
