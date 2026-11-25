#!/usr/bin/env bash
# The selected automation source owns acquisition and package verification.
set -euo pipefail
script_directory="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
repository_root="$(cd -- "${script_directory}/.." && pwd -P)"
if [[ -n "${QUADLET_LENS_NODE_TOOLS_ROOT:-}" ]]; then
  if [[ "${QUADLET_LENS_NODE_TOOLS_ROOT}" != /opt/quadlet-lens-file-tools ]]; then
    printf 'Unexpected QuadletLens image Node installation; rebuild the Dev Container.\n' >&2
    exit 2
  fi
  exec python3 /opt/quadlet-lens-file-tools/boxferry-node-tools.py image-node-tool \
    --repository-root "${repository_root}" --cwd "${repository_root}" "$@"
fi
exec python3 "${script_directory}/shared-ci.py" node-tool --cwd "${repository_root}" "$@"
