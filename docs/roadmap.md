# Roadmap

The roadmap is ordered by dependencies rather than dates.

Cross-repository delivery uses the stable task numbers in the [implementation plan](implementation-plan.md). This roadmap remains the detailed internal phase order for QuadletLens.

## Status key

- [x] Completed and validated
- [ ] Open

## Phase 0: foundation — completed

- [x] Accept architecture and origin ADR.
- [x] Prototype and accept the systemd-style syntax representation.
- [x] Scaffold the crate, CI, lints, MSRV policy, and fixture metadata.
- [x] Establish the initial capability schema.
- [x] Define source spans and structured diagnostics.

## Phase 1: syntax and rendering — in progress

- [x] Parse ordered physical sections and entries with source locations.
- [x] Preserve repeated keys, comments, continuation context, line endings, and unknown lines.
- [ ] Preserve quote, newline, and command-argument semantics through real-generator fixtures.
- [x] Distinguish literal paths, relative paths, and native systemd specifiers such as `%h`.
- [x] Implement deterministic canonical rendering.
- [x] Provide validated programmatic construction for the supported native document types.
- [x] Establish malformed-input, round-trip, and property tests.

## Phase 2: typed Quadlet documents — in progress

- [x] Implement the first shared and `.container`, `.pod`, `.network`, and `.volume` unit-specific sections.
- [x] Add conservative path and native unit-reference value forms for the first conversion.
- [x] Model `.container` relationships with `.pod`, `.network`, and `.volume` resources.
- [ ] Model health-command forms and generic systemd readiness/ordering dependencies separately.
- [x] Build document sets and dependency graphs.
- [x] Preserve generic systemd sections, repeated entries, and unknown Quadlet keys.

## Phase 3: Podman 5.4 minimum through rolling current — in progress

- [x] Establish the initial capability catalogue for Podman 5.4.
- [x] Run the first-conversion fixture against every official image from Podman 5.4.0 through 5.8.2.
- [x] Build and run exact generators for Podman 5.8.3 through current 6.0.2.
- [ ] Validate the remaining native keys, value forms, fallbacks, and known limitations.
- [x] Cover path handling, pod membership, resource references, health commands, restart behavior, and fallback arguments.
- [ ] Run the real-generator fixture suite in rootless and rootful contexts where relevant.

## Phase 4: version expansion — open

- [ ] Complete capability evidence for each later Podman minor version in order.
- [ ] Track introductions, changes, deprecations, removals, and known patch bugs.
- [ ] Add systemd capability checks required by supported Quadlet behavior.
- [ ] Support explicit distribution capability overrides.

## Phase 5: ecosystem hardening — in progress

- [ ] Expand the licensed real-world corpus.
- [x] Establish the supported 0.1.x public API and versioned catalogue schema.
- [x] Prepare and validate compatibility documentation and the QuadletLens 0.1.0 release candidate.
- [ ] Provide optional verification tooling for installed Podman generators.

## Maintainer-controlled 0.1.0 release operation — completed

- [x] Publish QuadletLens 0.1.0 from the reviewed clean default-branch commit using the documented
  one-time crates.io bootstrap.
- [x] Revoke the bootstrap token, configure trusted publishing, and run the protected release
  workflow from the same commit to create the tag, attestation, and GitHub release.

## Additive 0.1.1 generation boundary — release candidate

- [x] Add typed native document construction without exposing parser internals.
- [x] Reject wrong-section keys, duplicate singleton keys, line endings, and NUL bytes.
- [x] Preserve repeatable native and generic systemd entries deterministically.
- [x] Parse and validate every generated document before returning it.
- [x] Add consumer, document-set, rejection-path, MSRV, and documentation coverage.
- [ ] Publish QuadletLens 0.1.1 through the protected trusted-publishing workflow.

Follow the exact [release process](releasing.md). BoxFerry can consume the released 0.1.1 API
without a sibling path or Git dependency.

## Issue-derived evidence

The dated [Podlet regression map](research/podlet-regressions-2026-08-01.md) records concrete
syntax, document-set, capability, and generator cases behind these tasks. Issue closure is not
compatibility evidence; exact Podman/systemd documentation and observations remain required.
