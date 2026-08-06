# Changelog

All notable changes to QuadletLens will be documented in this file. The project follows Semantic
Versioning for its documented pre-1.0 public API.

## [Unreleased]

## [0.1.10] - 2026-08-06

### Added

- Typed singleton `Entrypoint` parsing and programmatic construction, kept distinct from `Exec`.
- Podman 5.4.0-through-6.0.2 capability and real-generator coverage for JSON command-array
  entrypoints, including both observed equivalent CLI encodings.
- Typed singleton `RunInit` parsing and programmatic construction that preserves omission,
  explicit `true`/`false`, and raw noncanonical one-line values without boolean interpretation.
- Exact real-generator verification across every supported Podman patch release from 5.4.0 through
  6.0.2: `RunInit=true` emits one `--init`, while a dedicated `RunInit=false` fixture emits one
  `--init=false`.
- Typed singleton container `StopSignal` and `StopTimeout` parsing and programmatic construction,
  retaining native signal spelling and zero timeout values.
- Podman 5.4.0-through-6.0.2 capability and generator observations for named and numeric stop
  signals and positive and zero stop timeouts, without asserting runtime or cross-format lifecycle
  equivalence.
- Typed singleton container `Pull` parsing and programmatic construction, preserving omission and
  raw noncanonical one-line values without policy interpretation.
- Podman 5.4.0-through-6.0.2 capability and generator-output evidence for isolated `always`,
  `missing`, `never`, and `newer` forms; image-pull runtime behavior remains untested.
- Typed singleton container `PidsLimit` parsing and raw programmatic construction, preserving
  omission, zero, and noncanonical one-line values without semantic interpretation.
- Additive `PidsLimit` construction helper for only `-1` unlimited or nonzero ASCII-decimal finite
  values, preserving leading zeros and arbitrary-precision spelling without integer overflow.
- Podman 5.4.0-through-6.0.2 generator-output evidence for isolated positive and `-1`
  `--pids-limit` forms; runtime cgroup behavior and zero remain outside the capability claim.
- Typed singleton container `HostName` parsing and programmatic construction, preserving omission
  and raw native values without Compose RFC-1123 validation or normalization.
- Podman 5.4.0-through-6.0.2 generator-output evidence for exactly one isolated logical
  `--hostname app.example` argument. Runtime hostname behavior and the documented pod shared-UTS
  hostname precedence remain outside the evidence.
- Typed singleton container and pod `ShmSize` parsing and programmatic construction, preserving
  omission and raw native values without imposing a cross-format size grammar.
- Additive `ShmSize` construction for non-negative ASCII-decimal amounts with optional lowercase
  `b`, `k`, `m`, or `g`, preserving leading zeros and arbitrary precision; explicit zero remains
  distinguishable as Podman's documented unlimited value.
- Separate container and pod capabilities plus 5.4.0-through-6.0.2 generator-output evidence for
  positive container, zero container, and pod-owned `--shm-size` arguments without runtime,
  default, rootless, or `/dev/shm` enforcement claims.
- Typed repeatable container `DropCapability` parsing and programmatic construction, preserving
  omission, repetition, insertion order, and exact opaque one-line values without splitting,
  deduplication, case normalization, or capability-name validation.
- `quadlet.container.drop-capability` native capability evidence from Podman 5.4.0 through 6.0.2,
  with a complete 20-patch generator fixture requiring four ordered lowercase `--cap-drop` forms
  and no `--cap-add`; runtime privilege outcomes remain outside the claim.
- Typed repeatable container `AddCapability` parsing and programmatic construction, preserving
  omission, empty native reset assignments, duplicates, insertion order, case, and exact opaque
  space-separated text without splitting, deduplication, normalization, or capability validation.
- `quadlet.container.add-capability` native capability evidence from Podman 5.4.0 through 6.0.2,
  with isolated four-argument addition and combined drop-all/add-one generator fixtures. Tagged
  source, rather than the Quadlet prose, records `all`, resets, lowercasing, ordering, and merger
  semantics; runtime privilege outcomes remain outside the claim.
- Typed repeatable container `Tmpfs` parsing and programmatic construction, preserving omission,
  empty native reset assignments, duplicates, insertion order, case, options, and exact opaque
  one-line values without splitting, normalization, validation, or conflation with `Volume`.
- `quadlet.container.tmpfs` native capability evidence from Podman 5.4.0 through 6.0.2, with a
  complete 20-patch post-reset generator fixture requiring exactly one final logical
  `--tmpfs /data:mode=755,uid=1009,gid=1009` and no pre-reset or extra tmpfs form. Separate CLI
  documentation records Linux mount flags and the `rw,noexec,nosuid,nodev` default; target-option,
  rootless, mount, copy-up, and runtime behavior remain outside the generator claim.
- Typed repeatable container `Sysctl` parsing and programmatic construction, preserving omission,
  empty native reset assignments, duplicates, insertion order, case, systemd quoting/specifiers,
  whitespace, and exact opaque one-line text without parsing or validating assignments.
- `quadlet.container.sysctl` native capability evidence from Podman 5.4.0 through 6.0.2, with
  endpoint manuals, Podman-run namespace limitations, tagged `LookupAllStrv` construction,
  tokenization/reset source, and a complete 20-patch fixture requiring exactly one final
  `--sysctl net.ipv4.ip_forward=1`. Runtime namespace state, rootless behavior, kernel acceptance,
  actual parameter effects, Compose, and BoxFerry mapping remain outside this generator-only claim.
- Typed repeatable container `Ulimit` parsing and programmatic construction, preserving omission,
  empty native resets, duplicates, insertion order, case, quotes/specifiers, and exact opaque
  one-line values without splitting, unquoting, normalization, or grammar validation. Pod
  `Ulimit` remains unknown and preserved.
- `quadlet.container.ulimit` native capability evidence from Podman 5.4.0 through 6.0.2, with
  distinct endpoint manuals, Podman-run grammar/default caveats, tagged `LookupAll` command/reset
  source, and a complete 20-patch fixture requiring exactly two ordered post-reset `--ulimit`
  arguments. Runtime enforcement, host inheritance, defaults, cgroups, rootless behavior, and
  unverified resource-name acceptance remain outside the claim.
- Typed repeatable container `AddDevice` parsing and programmatic construction, preserving
  omission, empty native resets, duplicates, insertion order, case, quotes/specifiers,
  whitespace-token-containing physical values, leading `-`, and exact opaque text without
  splitting, unquoting, normalization, device checks, or semantic validation. Pod `AddDevice`
  remains unknown and preserved.
- `quadlet.container.add-device` native capability evidence from Podman 5.4.0 through 6.0.2, with
  endpoint manuals, Podman-run caveats, tagged `LookupAllStrv` tokenization/reset and conditional
  leading-minus command source, and a complete 20-patch fixture requiring exactly two ordered
  final post-reset `--device` arguments and exactly two total. The fixture has no leading `-`,
  accesses no device, and makes no CDI, runtime, rootless, SELinux, cgroup, existence, or symlink
  claim.
- Typed singleton container `Memory` parsing and programmatic construction, preserving omission,
  duplicates, empty values, quoting, specifiers, and exact opaque one-line text without semantic
  interpretation. Pod `Memory` remains unknown and preserved.
- Additive `Memory` construction for positive ASCII-decimal amounts with optional lowercase `b`,
  `k`, `m`, or `g`, preserving leading zeros and arbitrary precision without integer parsing.
- `quadlet.container.memory` native capability evidence from its Podman 5.5.0 introduction through
  6.0.2, with all 17 patch generators requiring exactly one final
  `--memory 16777216b` argument. Separate 5.4.x observations reject or exclude the unsupported
  key. Runtime cgroup, page-size, swap, host-memory, rootless, and cross-format claims remain out
  of scope.

## [0.1.9] - 2026-08-05

### Added

- Typed singleton `ContainerName` parsing and programmatic construction.
- Capability and real-generator evidence for explicit Podman container names across every
  supported Podman patch release from 5.4.0 through 6.0.2.

## [0.1.8] - 2026-08-05

### Added

- A license-reviewed, blob-pinned real-world Quadlet corpus spanning 35 files from ten upstream,
  vendor, distribution, platform, organization-example, and community projects.
- Typed singleton `Rootfs` parsing and programmatic construction as the mutually exclusive
  alternative to `Image`, with Podman 5.4.0 through 6.0.2 capability and generator evidence.

### Fixed

- Rootfs-backed `.container` units no longer receive the incorrect missing-`Image` diagnostic;
  empty sources and conflicting `Image`/`Rootfs` entries remain explicit errors.

## [0.1.7] - 2026-08-05

### Added

- Typed repeatable container `Label` parsing and programmatic construction.
- Capability and complete real-generator evidence for repeated, empty, and quote-bearing OCI label
  assignments from Podman 5.4.0 through 6.0.2, including their version-specific systemd whitespace
  spelling.

## [0.1.6] - 2026-08-03

### Added

- Typed singleton pod `UserNS` parsing and programmatic construction.
- Capability evidence for pod-level user namespaces from Podman 5.4.0 through 6.0.2.
- A distinct pod `--userns auto:size=8192` assertion in the complete real-generator matrix.
- Typed repeatable container `Secret` parsing and programmatic construction.
- Capability and complete real-generator evidence for mounted-file and environment-variable
  secret option forms from Podman 5.4.0 through 6.0.2.

## [0.1.5] - 2026-08-03

### Added

- Typed container `User`, `Group`, `UserNS`, repeatable `GroupAdd`, `WorkingDir`, and `ReadOnly`
  keys for execution identity and context.
- Capability records covering Podman 5.4.0 through 6.0.2 for all six keys.
- Full 20-patch real-generator evidence for the corresponding Podman arguments.

## [0.1.4] - 2026-08-03

### Added

- Typed `Notify` container readiness signaling and a `notify-healthy` capability record.
- Typed `Requires`, `Wants`, and `After` programmatic `[Unit]` construction with separate
  capability records.
- Full 20-patch real-generator evidence for health-gated readiness and dependency ordering from
  Podman 5.4.0 through 6.0.2.

## [0.1.3] - 2026-08-03

### Added

- Typed `HealthInterval`, `HealthRetries`, `HealthStartPeriod`, and `HealthTimeout` container keys.
- Capability records and full 20-patch real-generator evidence for regular health-check timing
  from Podman 5.4.0 through 6.0.2.

## [0.1.2] - 2026-08-03

### Added

- Repeatable typed `AddHost` keys for container and pod units, including the `host-gateway` value
  form and capability evidence across Podman 5.4.0 through 6.0.2.
- Real-generator coverage for container and pod host mappings across every supported Podman patch
  release.

## [0.1.1] - 2026-08-03

### Added

- Validated programmatic construction for the supported native Quadlet document types.
- Typed native keys, open-ended generic systemd directives, exact one-line values, deterministic
  section ordering, repeated-entry preservation, singleton rejection, and parse-back validation.
- A documented generation boundary and external-consumer API contract for BoxFerry and other
  library users.

## [0.1.0] - 2026-08-02

### Added

- Loss-aware ordered Quadlet syntax with exact preservation, structured recovery, source spans,
  and deterministic conservative rendering.
- Source-aware native typed documents for `.container`, `.pod`, `.network`, and `.volume` units,
  including generic systemd sections, unknown entries, repeated keys, and conservative path and
  unit-reference classifications.
- Exact-name multi-document dependency resolution with explicit resolved, missing, and ambiguous
  reference states.
- A strict data-driven capability catalogue with Podman 5.4.0 as the minimum, finite evidence
  through Podman 6.0.2, version ranges, fallbacks, known bugs, and evidence levels.
- A digest-pinned real-generator harness covering every Podman patch release from 5.4.0 through
  6.0.2 for the first BoxFerry conversion subset.
