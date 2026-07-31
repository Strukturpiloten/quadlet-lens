# Roadmap

The roadmap is ordered by dependencies rather than dates.

Cross-repository delivery uses the stable task numbers in the [implementation plan](implementation-plan.md). This roadmap remains the detailed internal phase order for QuadletLens.

## Phase 0: foundation

- Accept architecture and origin ADR.
- Prototype the systemd-style syntax representation.
- Scaffold crate, CI, lints, MSRV policy, capability schema, and fixture metadata.
- Define source spans and structured diagnostics.

## Phase 1: syntax and rendering

- Parse ordered sections and entries with source locations.
- Preserve repeated keys, comments, continuations, and unknown data.
- Implement deterministic canonical rendering.
- Establish malformed-input, round-trip, and property tests.

## Phase 2: typed Quadlet documents

- Implement shared and unit-specific sections.
- Add typed value forms and cross-file references.
- Build document sets and dependency graphs.
- Preserve generic systemd sections and unknown Quadlet keys.

## Phase 3: Podman 5.4 baseline

- Establish the initial capability catalogue for Podman 5.4.
- Validate native keys, value forms, fallbacks, and known limitations.
- Run the real-generator fixture suite in rootless and rootful contexts where relevant.

## Phase 4: version expansion

- Add each later Podman minor version in order.
- Track introductions, changes, deprecations, removals, and known patch bugs.
- Add systemd capability checks required by supported Quadlet behavior.
- Support explicit distribution capability overrides.

## Phase 5: ecosystem hardening

- Expand the licensed real-world corpus.
- Stabilize the public API and catalogue schema.
- Publish compatibility documentation and releases.
- Provide optional verification tooling for installed Podman generators.
