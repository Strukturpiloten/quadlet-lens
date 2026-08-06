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
| `[Container]` | `AddHost`, `ContainerName`, `Image`, `Rootfs`, `Entrypoint`, `RunInit`, `StopSignal`, `StopTimeout`, `Pull`, `PidsLimit`, `HostName`, `ShmSize`, `DropCapability`, `AddCapability`, `Tmpfs`, `Sysctl`, `Ulimit`, `AddDevice`, `Memory`, `Exec`, `Environment`, `EnvironmentFile`, `Label`, `Secret`, `User`, `Group`, `UserNS`, `GroupAdd`, `WorkingDir`, `ReadOnly`, `PublishPort`, `Volume`, `Network`, `Pod`, `HealthCmd`, `Notify`, `HealthInterval`, `HealthRetries`, `HealthStartPeriod`, `HealthTimeout`, `PodmanArgs` |
| `[Pod]` | `AddHost`, `PodName`, `PublishPort`, `Network`, `Volume`, `UserNS`, `ShmSize` |
| `[Network]` | `NetworkName` |
| `[Volume]` | `VolumeName` |
| `[Unit]`, `[Service]`, `[Install]` | Open-ended generic systemd directives with source/order preservation; typed generation and explicit capability evidence exist for `[Unit]` `Requires=`, `Wants=`, and `After=`, and `[Service]` `Restart=`. |

The current manual contains 49 additional container keys, 18 additional pod keys, 17 additional
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
The same process subset includes singleton `RunInit`. Authored omission remains absent, explicit
`true` and `false` remain distinct model values, and raw noncanonical one-line values are preserved
without boolean interpretation. For every supported patch release, the generator matrix proves
that `RunInit=true` emits exactly one `--init` argument and `RunInit=false` emits exactly one
`--init=false`. It does not establish runtime init behavior.

The container stop-lifecycle subset includes singleton `StopSignal` and `StopTimeout`. The native
model retains exact authored one-line values, including zero, while the capability catalogue
records named/numeric signals and non-negative integer seconds as evidenced supported caller forms.
The generator matrix observes `--stop-signal SIGUSR1`, `--stop-signal 9`, a positive
`--stop-timeout 37`, and `--stop-timeout 0` across the supported range. It does not semantically
validate other raw values, start containers, measure elapsed stop time, establish whether zero
sends a signal, or assert equivalence with another format's lifecycle defaults.

The image-acquisition subset includes singleton `Pull`. Omission remains absent and exact one-line
values stay uninterpreted. Capability and generator evidence cover `always`, `missing`, `never`,
and `newer` as matching `--pull` arguments, without contacting a registry or inspecting local
image storage.

The resource subset includes singleton `PidsLimit`. Omission, authored zero, and noncanonical
one-line values remain distinct raw model and builder values. `PidsLimit::unlimited()` and
`PidsLimit::finite` provide safe construction for `-1` or nonzero ASCII-decimal spellings and
reject empty, nondecimal, or all-zero text. They retain leading zeros and arbitrary-precision
digits without parsing or overflow. Capability and generator evidence cover isolated
`--pids-limit 127` and `--pids-limit -1` output across Podman 5.4.0 through 6.0.2. They do not
establish a portable numeric maximum, cover zero, start a container, inspect its cgroup, or
establish process-exhaustion behavior.

The shared-memory subset includes separate singleton container and pod `ShmSize` keys. Parsed and
raw builder values remain exact and opaque. `ShmSize::new` accepts only a non-negative
ASCII-decimal amount with optional lowercase `b`, `k`, `m`, or `g`, while retaining leading zeros
and arbitrary-precision text without parsing. `ShmSize::unlimited()` emits the documented explicit
zero value, distinguishable from omission and Podman's documented `64m` default. The generator
matrix proves one matching `--shm-size 67108864b`, `--shm-size 0`, and pod-owned `--shm-size 32m`
argument, with no duplicate in the joined container. It does not start workloads, inspect shared
IPC or `/dev/shm`, exercise host IPC, establish runtime enforcement, or make rootless claims.

The capability-security subset includes repeatable container `DropCapability` and `AddCapability`.
Omission, empty native reset assignments, repeated entries, authored order, and exact one-line
values remain distinct. The model and builder do not
split space-separated lists, deduplicate capability names, lowercase source text, or validate a
native capability whitelist. Podman 5.4 documentation defines both repeatable space-separated lists
and documents lowercase `all` only for drops. The complete generator matrix observes four ordered
lowercase arguments for each isolated fixture, with no opposite capability form, plus exactly one
drop-all before one specific addition in a combined fixture. Tagged 5.4.0 and 6.0.2 source records
empty resets, lowercasing, drop-before-add construction, and the special merger semantics of
`all`; that special addition behavior is not attributed to the Quadlet prose. These are
definition, source, and generator-output observations, not claims about rootless/rootful
execution, effective bounding sets, user namespaces, SELinux/seccomp interaction, or runtime
privilege outcomes.

The temporary-filesystem subset includes repeatable container `Tmpfs`. Omission, empty reset
assignments, duplicates, order, case, and exact `CONTAINER-DIR[:OPTIONS]` spelling remain distinct
opaque values. QuadletLens does not split, normalize, deduplicate, validate target mount options,
or conflate `Tmpfs` with the separate `Volume` grammar. Podman's Quadlet documentation establishes
the repeatable mapping; separate Podman CLI documentation records Linux mount flags and the
`rw,noexec,nosuid,nodev` omission default. Tagged source and the complete generator matrix prove
that `LookupAll` leaves exactly one final post-reset
`--tmpfs /data:mode=755,uid=1009,gid=1009` command form. They do not start a container, create or
inspect a mount, enforce defaults, exercise copy-up, or establish rootless/runtime behavior.
There is no pod `Tmpfs` typed key or capability in this slice.

The kernel-parameter subset includes repeatable container `Sysctl`. Omission, empty resets,
duplicates, ordering, case, whitespace, systemd quoting/specifiers, and exact one-line values are
preserved without parsing `name=value` assignments, splitting lists, normalization, or namespace
validation. Endpoint manuals and tagged source establish native spelling, `LookupAllStrv`
tokenization, command construction, and reset behavior. The complete generator matrix proves one
final post-reset `--sysctl net.ipv4.ip_forward=1`, with neither pre-reset setting nor any other
sysctl argument. It does not start a container or establish namespace state, rootless behavior,
kernel acceptance, runtime equivalence, or actual parameter effects. There is no pod `Sysctl`
typed key or capability; Compose and BoxFerry mapping are outside this native slice.

The resource-limit subset includes repeatable container `Ulimit`. Omission, empty resets,
duplicates, order, case, quotes/specifiers, and every exact one-line value are preserved without
splitting, unquoting, or validating `TYPE=SOFT[:HARD]`. Endpoint manuals, Podman-run grammar/default
caveats, and tagged source establish the native spelling and `LookupAll` command/reset path. The
complete generator matrix proves exactly two ordered final post-reset `--ulimit` arguments for
`nproc=4096:8192` and `stack=-1:-1`, with no pre-reset, duplicate, empty, or alternate form. It does
not execute a container or claim runtime enforcement, host inheritance, defaults, cgroups,
rootless behavior, or acceptance of unverified resource names. There is no pod `Ulimit` typed key
or capability; Compose and BoxFerry mapping are outside this native slice.

The host-device subset includes repeatable container `AddDevice`. Omission, every physical value,
empty resets, duplicates, order, case, quotes/specifiers, whitespace-token-containing lines, and a
leading `-` are preserved without splitting, unquoting, parsing, or validation. Endpoint manuals,
Podman-run caveats, and tagged source establish native spelling plus generator-only
`LookupAllStrv`, reset, conditional leading-minus, and command construction behavior. The complete
generator matrix proves exactly two ordered final post-reset `--device` arguments and exactly two
total, with neither pre-reset mapping nor empty or alternate form. The fixture deliberately uses no
leading `-`, accesses no device, and starts no workload. There is no pod `AddDevice` typed key or
capability; CDI, runtime access, rootless, SELinux, cgroup, device existence, symlink behavior,
Compose, and BoxFerry mapping remain outside this native slice.

The memory-limit subset includes singleton container `Memory`, introduced by Podman 5.5.0. Parsed
and raw builder values preserve omission, duplicates, empty assignments, quotes, specifiers, zero,
and noncanonical text without runtime interpretation. `Memory::new` constructs positive
arbitrary-precision ASCII-decimal amounts with no suffix or one lowercase `b`, `k`, `m`, or `g`,
retaining leading zeros without parsing. A separate fixture keeps the existing 5.4-compatible
matrix unchanged: the three 5.4.x generators reject or exclude `Memory`, while every one of the 17
patch releases from 5.5.0 through 6.0.2 emits exactly one final `--memory 16777216b`. This does not
establish cgroup enforcement, page rounding, swap interaction, host-memory availability, rootless
behavior, runtime inspection, or cross-format equivalence. Pod `Memory` remains unknown.

Cross-format selection remains BoxFerry-owned. A future exact Compose mapping is intentionally
bounded to a positive explicitly byte-qualified value on a separate container with private IPC;
pod lifting, host IPC, implicit defaults, zero equivalence, and runtime inspection require separate
policy and evidence.

The container-network-identity subset includes singleton `HostName`. Its exact one-line value is
opaque and omission remains omission; QuadletLens neither applies Compose RFC-1123 validation nor
normalizes native spelling. Podman documents that the key requires a private UTS namespace. The
isolated generator fixture relies on the default private UTS namespace and verifies exactly one
logical `--hostname app.example` argument across Podman 5.4.0 through 6.0.2. When a container joins
a pod with the default shared UTS namespace, the pod hostname wins. The fixture does not start a
container, inspect its runtime hostname or namespace, or prove pod precedence.

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
