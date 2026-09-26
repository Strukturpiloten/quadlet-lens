#!/usr/bin/env bash
set -euo pipefail

label_key='io.github.strukturpiloten.quadlet-lens.generator-run'
run_id="${QUADLET_LENS_GENERATOR_RUN_ID:?an exact generator run ID is required for cleanup}"
engine="${QUADLET_LENS_CONTAINER_ENGINE:-podman}"

if [[ ! "${run_id}" =~ ^[a-zA-Z0-9_-]{1,80}$ ]]; then
  printf 'Invalid generator run ID; refusing container cleanup.\n' >&2
  exit 1
fi
case "${engine##*/}" in
  podman | docker) ;;
  *)
    printf 'Unsupported generator container engine; refusing cleanup.\n' >&2
    exit 1
    ;;
esac

filter="label=${label_key}=${run_id}"
if ! listed="$(timeout 20s "${engine}" ps --all --quiet --no-trunc --filter "${filter}")"; then
  printf 'Could not list exact run-owned generator containers.\n' >&2
  exit 1
fi
readarray -t ids <<< "${listed}"
if ((${#ids[@]} > 64)); then
  printf 'Too many generator containers for bounded cleanup.\n' >&2
  exit 1
fi

status=0
for id in "${ids[@]}"; do
  [[ -n "${id}" ]] || continue
  if [[ ! "${id}" =~ ^[a-f0-9]{64}$ ]]; then
    printf 'Refusing non-full generator container ID.\n' >&2
    status=1
    continue
  fi
  template="{{ index .Config.Labels \"${label_key}\" }}"
  if ! actual="$(timeout 10s "${engine}" inspect --format "${template}" "${id}")"; then
    printf 'Could not inspect generator container %s.\n' "${id}" >&2
    status=1
    continue
  fi
  if [[ "${actual}" != "${run_id}" ]]; then
    printf 'Refusing generator container %s without exact run ownership.\n' "${id}" >&2
    status=1
    continue
  fi
  if ! timeout 10s "${engine}" rm --force "${id}"; then
    printf 'Could not remove run-owned generator container %s.\n' "${id}" >&2
    status=1
  fi
done
exit "${status}"
