# Changelog

All notable changes to QuadletLens will be documented in this file. The project follows Semantic
Versioning for its documented pre-1.0 public API.

## [Unreleased]

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
