# Native Quadlet coverage

This document distinguishes loss-aware parsing from typed construction and version-evidenced
generation. It was audited against the current official
[Quadlet manual](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html) and the
supported Podman 5.4 floor on 2026-08-06. The exact current untyped-key inventory and promotion
order live in the [roadmap](roadmap.md).

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
| `[Container]` | `AddHost`, `ContainerName`, `Image`, `Rootfs`, `Entrypoint`, `RunInit`, `Exec`, `Environment`, `EnvironmentFile`, `Label`, `Secret`, `User`, `Group`, `UserNS`, `GroupAdd`, `WorkingDir`, `ReadOnly`, `PublishPort`, `Volume`, `Network`, `Pod`, `HealthCmd`, `Notify`, `HealthInterval`, `HealthRetries`, `HealthStartPeriod`, `HealthTimeout`, `PodmanArgs` |
| `[Pod]` | `AddHost`, `PodName`, `PublishPort`, `Network`, `Volume`, `UserNS` |
| `[Network]` | `NetworkName` |
| `[Volume]` | `VolumeName` |
| `[Unit]`, `[Service]`, `[Install]` | Open-ended generic systemd directives with source/order preservation; typed generation and explicit capability evidence exist for `[Unit]` `Requires=`, `Wants=`, and `After=`, and `[Service]` `Restart=`. |

The current manual contains 62 additional container keys, 19 additional pod keys, 17 additional
network keys, and 15 additional volume keys that are syntax-preserved but not typed. The complete
lists, plus every current build, image, kube, artifact, and Quadlet-section key, are maintained in
the [specification coverage ledger](roadmap.md#specification-coverage-ledger).

## Next promotion

The execution-identity subset available since the Podman 5.4 floor includes container `User`,
`Group`, `UserNS`, repeatable `GroupAdd`, `WorkingDir`, and `ReadOnly`, plus pod-level `UserNS` for
the namespace shared by pod members. The exact generator matrix confirms the corresponding
`--user`, `--userns`, `--group-add`, `--workdir`, and `--read-only` output across all 20 recorded
patch releases through 6.0.2. Values remain exact authored text; QuadletLens does not resolve
users, groups, paths, or namespace state.

The secret subset includes repeatable mounted-file and environment-variable Podman secret
references, with target, UID, GID, and mode option spellings retained as exact native text.
Generator evidence proves the emitted `--secret` arguments; secret creation, content, rotation,
and runtime availability remain caller-owned concerns.

The label subset includes ordered, repeatable container `Label=key=value` assignments. The full
generator matrix proves ordinary, empty, and JSON-like quote/whitespace values from Podman 5.4.0
through 6.0.2. It explicitly accepts the literal-space systemd spelling emitted by 5.4.x and the
equivalent `\x20` spelling emitted from 5.5.0 onward. Label name conventions, duplicate-name
semantics, and labels owned by network or volume resources remain caller- or future-model
responsibilities.

The workload-source subset accepts exactly one container `Image` or `Rootfs` entry. `Rootfs` is
documented at the Podman 5.4 floor, exercised by the public `containers/qm` unit, and verified as a
generated `--rootfs` argument through the supported generator matrix. QuadletLens retains its exact
value and does not inspect the host filesystem, parse overlay-rootfs options, or verify SELinux
labels.

The container-identity subset includes singleton `ContainerName`. It is documented at the Podman
5.4 floor and verified as an exact `--name` generator argument through 6.0.2. The value is not
derived from the unit basename, checked for host collisions, or treated as a systemd unit name.

The process subset includes singleton `Entrypoint`. QuadletLens retains the exact executable or
JSON command-array text instead of decoding systemd/JSON quoting. The generator matrix verifies
that every supported Podman release passes the JSON array to `podman run`; it records the exact
presentation boundary from a separate argument through 5.8.1 to `--entrypoint=...` from 5.8.2.
The same process subset includes singleton `RunInit`; the matrix proves exactly one `--init`
argument for every supported patch release.

The dependency-readiness subset also includes:

- `Notify=healthy` to delay service readiness until Podman reports a healthy container;
- `Requires=` and `Wants=` for strong and weak systemd activation dependencies; and
- `After=` for independent startup ordering.

The parser still retains every generic systemd directive without forcing it into a closed enum.
Typed systemd keys are a programmatic-generation aid, not a complete systemd semantic model.
Runtime activation, failed-unit propagation, cycles, stop ordering, and restart propagation remain
outside current generator evidence and require separate systemd-aware validation.

The next cohesive promotion should cover the remaining lifecycle behavior required by the
BoxFerry conversion roadmap without conflating Compose restart semantics with systemd or Podman.

## Promotion checklist

A key or unit type becomes supported only with:

1. parser classification and deterministic rendering;
2. builder cardinality and section validation;
3. data-driven minimum/maximum version capability records;
4. exact documentation or source evidence;
5. real-generator fixtures across the claimed support range; and
6. public API and limitation documentation.
