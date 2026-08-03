# Native Quadlet coverage

This document distinguishes loss-aware parsing from typed construction and version-evidenced
generation. It was audited against the current official
[Quadlet manual](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html) and the
supported Podman 5.4 floor on 2026-08-03.

## Coverage layers

| Layer | Contract |
| --- | --- |
| Syntax | Ordered sections, repeated keys, continuations, comments, unknown keys, and systemd specifiers are retained. |
| Native type | A unit, section, or key can be inspected and constructed through a typed public API. |
| Capability | The data catalogue states support over an explicit Podman version range and cites evidence. |
| Generator | Repository fixtures have been accepted by the recorded real Quadlet generators. |

Recognition is not a version claim. A key is ready for BoxFerry generation only when the native
type, capability, and relevant generator evidence agree.

## Unit types

| Quadlet unit | Syntax preservation | Typed document/builder | Current BoxFerry output |
| --- | --- | --- | --- |
| `.container` | yes | yes | yes |
| `.pod` | yes | yes | optional explicit grouping |
| `.network` | yes | yes | application-owned networks |
| `.volume` | yes | yes | application-owned volumes |
| `.image` | yes | no | no |
| `.build` | yes | no | no |
| `.kube` | yes | no | no |
| `.artifact` | yes | no | no; the current manual marks it experimental |

Unsupported native sections remain available through the syntax tree. They are not mislabeled as
one of the four typed unit types.

## Typed key boundary

| Section | Typed keys |
| --- | --- |
| `[Container]` | `AddHost`, `Image`, `Exec`, `Environment`, `EnvironmentFile`, `User`, `Group`, `UserNS`, `GroupAdd`, `WorkingDir`, `ReadOnly`, `PublishPort`, `Volume`, `Network`, `Pod`, `HealthCmd`, `Notify`, `HealthInterval`, `HealthRetries`, `HealthStartPeriod`, `HealthTimeout`, `PodmanArgs` |
| `[Pod]` | `AddHost`, `PodName`, `PublishPort`, `Network`, `Volume` |
| `[Network]` | `NetworkName` |
| `[Volume]` | `VolumeName` |
| `[Unit]`, `[Service]`, `[Install]` | Open-ended generic systemd directives with source/order preservation; typed generation and explicit capability evidence exist for `[Unit]` `Requires=`, `Wants=`, and `After=`, and `[Service]` `Restart=`. |

All other current manual keys are syntax-preserved but not yet part of the typed builder contract.
This includes many useful container keys such as DNS, capabilities, entrypoint, startup-health
settings, hostname, labels, resource limits, mounts, network aliases, secrets, security labeling,
and stop behavior. Pod, network, and volume sections likewise have broader native surfaces than
the first conversion subset.

## Next promotion

The execution-identity subset available since the Podman 5.4 floor now includes `User`, `Group`,
`UserNS`, repeatable `GroupAdd`, `WorkingDir`, and `ReadOnly`. The exact generator matrix confirms
their `--user`, `--userns`, `--group-add`, `--workdir`, and `--read-only` output across all 20
recorded patch releases through 6.0.2. Values remain exact authored text; QuadletLens does not
resolve users, groups, paths, or namespace state.

The dependency-readiness subset also includes:

- `Notify=healthy` to delay service readiness until Podman reports a healthy container;
- `Requires=` and `Wants=` for strong and weak systemd activation dependencies; and
- `After=` for independent startup ordering.

The parser still retains every generic systemd directive without forcing it into a closed enum.
Typed systemd keys are a programmatic-generation aid, not a complete systemd semantic model.
Runtime activation, failed-unit propagation, cycles, stop ordering, and restart propagation remain
outside current generator evidence and require separate systemd-aware validation.

The next cohesive promotion should cover lifecycle behavior required by the BoxFerry conversion
roadmap without conflating Compose restart and entrypoint semantics with systemd or Podman.

## Promotion checklist

A key or unit type becomes supported only with:

1. parser classification and deterministic rendering;
2. builder cardinality and section validation;
3. data-driven minimum/maximum version capability records;
4. exact documentation or source evidence;
5. real-generator fixtures across the claimed support range; and
6. public API and limitation documentation.
