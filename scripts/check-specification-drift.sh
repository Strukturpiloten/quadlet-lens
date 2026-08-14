#!/usr/bin/env bash

set -Eeuo pipefail

script_directory="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
repository_root="$(cd -- "${script_directory}/.." && pwd -P)"
readonly script_directory repository_root

if (($# != 1)); then
  printf 'Usage: %s PATH_TO_UPSTREAM_PODMAN_SYSTEMD_UNIT_MANUAL\n' "$0" >&2
  exit 2
fi

manual_path="$1"
inventory_path="${repository_root}/fixtures/specification-drift/quadlet-manual-current.toml"
if [[ ! -f "${manual_path}" || ! -f "${inventory_path}" ]]; then
  printf 'Quadlet specification drift inputs are missing.\n' >&2
  exit 2
fi

temporary_directory="$(mktemp -d)"
trap 'rm -rf -- "${temporary_directory}"' EXIT
manual_keys="${temporary_directory}/manual.keys"
inventory_keys="${temporary_directory}/inventory.keys"

bash "${script_directory}/extract-quadlet-manual-keys.sh" "${manual_path}" | LC_ALL=C sort -u > "${manual_keys}"
sed -nE 's/^[[:space:]]*\["([A-Za-z]+)", "([A-Za-z][A-Za-z0-9]+)", "(typed|preserved-only|intentionally-unsupported)"(, "[^"]+")?\],?$/\1\t\2/p' "${inventory_path}" | LC_ALL=C sort -u > "${inventory_keys}"

if diff --unified=3 --label inventory --label upstream "${inventory_keys}" "${manual_keys}"; then
  printf 'Quadlet manual inventory matches upstream keys.\n'
  exit 0
fi

printf 'Quadlet manual key drift detected; review the readable added/removed rows above.\n' >&2
exit 1
