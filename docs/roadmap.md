# Roadmap

The roadmap is ordered by dependencies rather than dates.

Cross-repository delivery uses the stable task numbers in the [implementation plan](implementation-plan.md). This roadmap remains the detailed internal phase order for QuadletLens.

## Status key

- [x] Completed and validated
- [ ] Open

## Phase 0: foundation — in progress

- [x] Accept architecture and origin ADR.
- [ ] Prototype the systemd-style syntax representation.
- [x] Scaffold the crate, CI, lints, MSRV policy, and fixture metadata.
- [ ] Establish the initial capability schema.
- [ ] Define source spans and structured diagnostics.

## Phase 1: syntax and rendering — open

- [ ] Parse ordered sections and entries with source locations.
- [ ] Preserve repeated keys, comments, continuations, and unknown data.
- [ ] Preserve quote, newline, and command-argument semantics through real-generator fixtures.
- [ ] Distinguish literal paths, relative paths, and native systemd specifiers such as `%h`.
- [ ] Implement deterministic canonical rendering.
- [ ] Establish malformed-input, round-trip, and property tests.

## Phase 2: typed Quadlet documents — open

- [ ] Implement shared and unit-specific sections.
- [ ] Add typed value forms and cross-file references.
- [ ] Model `.container` relationships with `.pod`, `.network`, and `.volume` resources.
- [ ] Model health-command forms and generic systemd readiness/ordering dependencies separately.
- [ ] Build document sets and dependency graphs.
- [ ] Preserve generic systemd sections and unknown Quadlet keys.

## Phase 3: Podman 5.4 baseline — open

- [ ] Establish the initial capability catalogue for Podman 5.4.
- [ ] Validate native keys, value forms, fallbacks, and known limitations.
- [ ] Cover path handling, pod membership, resource references, health commands, restart behavior, and fallback arguments.
- [ ] Run the real-generator fixture suite in rootless and rootful contexts where relevant.

## Phase 4: version expansion — open

- [ ] Add each later Podman minor version in order.
- [ ] Track introductions, changes, deprecations, removals, and known patch bugs.
- [ ] Add systemd capability checks required by supported Quadlet behavior.
- [ ] Support explicit distribution capability overrides.

## Phase 5: ecosystem hardening — open

- [ ] Expand the licensed real-world corpus.
- [ ] Stabilize the public API and catalogue schema.
- [ ] Publish compatibility documentation and releases.
- [ ] Provide optional verification tooling for installed Podman generators.

## Issue-derived evidence

The dated [Podlet regression map](research/podlet-regressions-2026-08-01.md) records concrete
syntax, document-set, capability, and generator cases behind these tasks. Issue closure is not
compatibility evidence; exact Podman/systemd documentation and observations remain required.
