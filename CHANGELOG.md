# Changelog

All notable changes to QuadletLens will be documented in this file. The project follows Semantic
Versioning for its documented pre-1.0 public API.

## [Unreleased]

### Added

- Adds source-located environment-file and environment-secret discovery with explicit caller-only
  value authorization, default payload redaction, and opt-in reset-safe deterministic environment
  sorting for generated literal plans ([#56](https://github.com/Strukturpiloten/quadlet-lens/issues/56)).

### Changed

- Promote curated `Unreleased` notes under a blank-line-safe release-plz version heading instead
  of generating duplicate changelog groups ([#65](https://github.com/Strukturpiloten/quadlet-lens/issues/65)).

- Replace repeated key, capability, generator, and release ledgers with bounded task-oriented
  guides that route exact support claims to catalogues, matrices, fixtures, and tests
  ([#58](https://github.com/Strukturpiloten/quadlet-lens/pull/58)).

## [0.2.1](https://github.com/Strukturpiloten/quadlet-lens/compare/v0.2.0...v0.2.1) - 2026-08-18

### Other

- Fix release-plz authentication and Renovate coverage ([#41](https://github.com/Strukturpiloten/quadlet-lens/pull/41))
- Update Podman Versions ([#39](https://github.com/Strukturpiloten/quadlet-lens/pull/39))

### Added

- Extends the reviewed Podman catalogue and exact generator matrix through 5.8.6 and 6.1.0,
  including the 6.1.0 `Container.ImageVolume` introduction boundary.

### Changed

- Automates version and changelog preparation with release-plz, makes this changelog the sole
  release-history source, and retains the protected trusted-publishing workflow as publisher.
- Treats `Container.ImageVolume` as a singleton, matching Podman's effective-value lookup.

## [0.2.0] - 2026-08-17

### Added

- Typed opaque Container support for automatic updates, cgroup and environment/proxy controls,
  user mappings, native mounts, retry/start/timezone/health-failure controls, with repeatability
  and relationship diagnostics preserved without host inspection or runtime claims.
- Completes current Container-key recognition with opaque configuration/global arguments,
  health logging and startup controls, image volumes, and generated service names.
- Completes current Pod-key recognition with opaque networking, labels, configuration/global
  arguments, mappings, and source-spanned mapping-conflict diagnostics.
- Completes current Network and Image-key recognition with opaque native values, ordered
  repeatable entries, and duplicate-singleton diagnostics; target-specific generator evidence is
  recorded only where separately verified.
- Completes current Kube-key recognition with a required opaque `Yaml` source, source-spanned
  missing/empty diagnostics, ordered native inputs, and exact `.network` document-set references.
- Extends Kube coverage through the audited 6.0.2 `LogOpt` and user-remapping keys, including
  reset-aware `Yaml` working-directory diagnostics and ordered `AutoUpdate` values.
- Adds experimental `.artifact` and shared `[Quadlet] DefaultDependencies` typing, redacted
  Artifact credentials/key debug output, exact document-set references, and finite 5.7.0–6.0.2
  generator evidence; pre-5.7 Artifact output remains unsupported.
- Types all nine reviewed systemd Unit relationships with reset-aware native-reference graphs,
  relationship identity, the Podman 5.5 rewrite boundary, and systemd 249 `Upholds` guidance.

### Changed

- Starts the 0.2.x API line with `SystemdUnitKey` owned only by `model`; removes the
  compatibility-only `render` re-export.
- SemVer validation derives the release type from Cargo package versions instead of forcing every
  candidate to be a patch.

## [0.1.13] - 2026-08-13

### Added

- Paired positive and negative tests for UTF-8 source locations, capability-schema validation,
  version ranges, catalogue evaluation, filenames, and document-set reference spans.
- A pinned CI and release coverage ratchet for the locked all-feature, all-target test suite.
- A shared VS Code and Dev Container workflow with one-command Rust, file-quality, policy,
  coverage, MSRV, offline-link, package, and API checks.

### Changed

- Patch releases now run an explicit patch-level public-API comparison in both CI and release
  validation.

### Fixed

- Local API checks always use an isolated writable Cargo cache instead of the container image's
  potentially read-only global package lock.

## [0.1.12] - 2026-08-10

### Added

- Native `.image` support with source preservation, sensitive debug redaction, and exact
  document-set references.
- Complete current native Volume coverage for image references, naming, ownership fields, and
  opaque configuration and argument entries.
- Complete current native Build coverage for environment and configuration arguments, service
  naming, and exact `.volume` references.
- Container reload entries with conflict diagnostics, plus Pod exit policy, stop timeout, and
  service naming.
- Evidence-scoped generic `PodmanArgs`, container logging and network identity, Network, and
  Volume support.
- Exact capability and dry-run generator evidence across Podman 5.4.0–6.0.2, including recorded
  version boundaries; no runtime or cross-format equivalence is claimed.

## [0.1.11] - 2026-08-07

### Added

- Opaque container keys for DNS, exposed ports, annotations, AppArmor, no-new-privileges,
  seccomp, SELinux label settings, Mask, and Unmask (`ContainerKey` ordinals 41–55).
- Capability and dry-run generator evidence across all 20 recorded Podman patches from 5.4.0
  through 6.0.2. AppArmor is unsupported through 5.7.1 and native from 5.8.0.
- Source-aware preservation for repeatable and singleton values without host inspection or runtime
  interpretation.

## [0.1.10] - 2026-08-06

### Added

- Native lifecycle and identity keys: `Entrypoint`, `RunInit`, `StopSignal`, `StopTimeout`,
  `Pull`, `PidsLimit`, `HostName`, and container/pod `ShmSize`.
- Native resource and device keys: `DropCapability`, `AddCapability`, `Tmpfs`, `Sysctl`,
  `Ulimit`, `AddDevice`, and container `Memory`.
- Capability and dry-run generator evidence across Podman 5.4.0 through 6.0.2; `Memory` is native
  from 5.5.0.

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
