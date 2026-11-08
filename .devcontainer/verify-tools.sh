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
