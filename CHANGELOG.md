# Changelog

All notable changes to QuadletLens will be documented in this file. The project follows Semantic
Versioning for its documented pre-1.0 public API.

## [Unreleased]

### Added

- Typed singleton `Entrypoint` parsing and programmatic construction, kept distinct from `Exec`.
- Podman 5.4.0-through-6.0.2 capability and real-generator coverage for JSON command-array
  entrypoints, including both observed equivalent CLI encodings.
- Typed singleton `RunInit` parsing and programmatic construction with finite capability evidence.
- Exact real-generator verification of one `--init` argument across every supported Podman patch
  release from 5.4.0 through 6.0.2.

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
