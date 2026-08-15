#!/usr/bin/env bash

set -Eeuo pipefail

if (($# != 1)); then
  printf 'Usage: %s PATH_TO_PODMAN_SYSTEMD_UNIT_MANUAL\n' "$0" >&2
  exit 2
fi

manual_path="$1"
if [[ ! -f "${manual_path}" ]]; then
  printf 'Quadlet manual does not exist: %s\n' "${manual_path}" >&2
  exit 2
fi

awk '
  /^## / {
    section = ""
    if ($0 == "## Container units [Container]") section = "Container"
    if ($0 == "## Pod units [Pod]") section = "Pod"
    if ($0 == "## Network units [Network]") section = "Network"
    if ($0 == "## Volume units [Volume]") section = "Volume"
    if ($0 == "## Build units [Build]") section = "Build"
    if ($0 == "## Image units [Image]") section = "Image"
    if ($0 == "## Kube units [Kube]") section = "Kube"
    if ($0 == "## Artifact units [Artifact]") section = "Artifact"
    if ($0 == "## Quadlet section [Quadlet]") section = "Quadlet"
    next
  }
  /^### `[^`]+=`/ && section != "" {
    key = $0
    sub(/^### `/, "", key)
    sub(/=`.*$/, "", key)
    print section "\t" key
  }
' "${manual_path}"
