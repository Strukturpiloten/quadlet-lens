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
- [x] Model regular health-command and timing keys separately from startup/readiness behavior.
- [x] Model generic systemd readiness/ordering dependencies for `Requires`, `Wants`, and `After`,
  plus container `Notify=healthy` readiness.
- [x] Build document sets and dependency graphs.
- [x] Preserve generic systemd sections, repeated entries, and unknown Quadlet keys.

## Phase 3: Podman 5.4 minimum through rolling current — in progress

- [x] Establish the initial capability catalogue for Podman 5.4.
- [x] Run the first-conversion fixture against every official image from Podman 5.4.0 through 5.8.2.
- [x] Build and run exact generators for Podman 5.8.3 through current 6.0.2.
- [ ] Validate the remaining native keys, value forms, fallbacks, and known limitations.
- [x] Cover path handling, pod membership, resource references, regular health commands/timings, restart behavior, and fallback arguments.
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

## Additive 0.1.1 generation boundary — completed

- [x] Add typed native document construction without exposing parser internals.
- [x] Reject wrong-section keys, duplicate singleton keys, line endings, and NUL bytes.
- [x] Preserve repeatable native and generic systemd entries deterministically.
- [x] Parse and validate every generated document before returning it.
- [x] Add consumer, document-set, rejection-path, MSRV, and documentation coverage.
- [x] Publish QuadletLens 0.1.1 through the protected trusted-publishing workflow.

Follow the exact [release process](releasing.md). BoxFerry can consume the released 0.1.1 API
without a sibling path or Git dependency.

## Additive 0.1.2 host-mapping boundary — completed

- [x] Type repeatable `AddHost` entries in `.container` and `.pod` documents.
- [x] Preserve ordinary addresses and the runtime-specific `host-gateway` value without
  normalization.
- [x] Add finite capability entries for container and pod host mappings.
- [x] Verify generated `--add-host` arguments across every Podman patch release from 5.4.0 through
  current 6.0.2.
- [x] Add release notes and validate the Rust 1.85.0 consumer boundary.
- [x] Publish QuadletLens 0.1.2 through the protected trusted-publishing workflow.

## Additive 0.1.3 regular-health boundary — completed

- [x] Type regular health interval, retries, start period, and timeout keys.
- [x] Add capability records from the Podman 5.4.0 floor through current 6.0.2.
- [x] Verify all four keys against every recorded Podman patch generator in that range.
- [x] Keep Compose `start_interval` distinct from Podman's startup-healthcheck mechanism.
- [x] Add release notes and validate the Rust 1.85.0 consumer boundary.
- [x] Publish QuadletLens 0.1.3 through the protected trusted-publishing workflow.

## Additive 0.1.4 dependency-readiness boundary — completed

- [x] Type the container `Notify` key and evidence the `healthy` readiness form.
- [x] Add typed programmatic construction for `[Unit]` `Requires`, `Wants`, and `After`.
- [x] Add separate capability records for strong, weak, and ordering dependencies.
- [x] Verify readiness and dependency fragments against every recorded Podman patch generator from
  5.4.0 through current 6.0.2.
- [x] Add release notes and validate the Rust 1.85.0 consumer boundary.
- [x] Publish QuadletLens 0.1.4 through the protected trusted-publishing workflow.

## Additive 0.1.5 execution-identity boundary — completed

- [x] Type container `User`, `Group`, `UserNS`, repeatable `GroupAdd`, `WorkingDir`, and `ReadOnly`.
- [x] Add capability records from the Podman 5.4.0 floor through current 6.0.2.
- [x] Verify the generated Podman arguments against every recorded patch generator in that range.
- [x] Add parser, builder, singleton/repetition, public-consumer, and documentation coverage.
- [x] Add release notes and validate the Rust 1.85.0 consumer boundary.
- [x] Publish QuadletLens 0.1.5 through the protected trusted-publishing workflow.

## Additive 0.1.6 pod user-namespace and secret boundary — completed

- [x] Type singleton pod `UserNS` parsing and programmatic construction.
- [x] Add a separate `quadlet.pod.userns` capability record from Podman 5.4.0 through 6.0.2.
- [x] Verify a pod-specific `--userns auto:size=8192` fragment at the support floor, image
  boundary, and current ceiling.
- [x] Add parser, builder, duplicate-singleton, catalogue, and public-consumer coverage.
- [x] Type repeatable container `Secret` parsing and programmatic construction.
- [x] Add `quadlet.container.secret` capability evidence for mounted-file and environment-variable
  option forms from Podman 5.4.0 through 6.0.2.
- [x] Verify both secret forms in the complete real-generator matrix.
- [x] Run the complete 20-patch generator matrix.
- [x] Add release notes and validate the Rust 1.85.0 consumer boundary.
- [x] Publish QuadletLens 0.1.6 through the protected trusted-publishing workflow.

## Additive 0.1.7 container-label boundary — release candidate

- [x] Type repeatable container `Label` parsing and programmatic construction.
- [x] Add `quadlet.container.label` capability evidence from Podman 5.4.0 through 6.0.2.
- [x] Verify repeated native label arguments in the complete 20-patch generator matrix.
- [x] Append the public key-enum variant without changing published discriminants.
- [x] Add parser, builder, catalogue, public-consumer, and documentation coverage.
- [x] Add release notes and validate the Rust 1.85.0 consumer boundary.
- [ ] Publish QuadletLens 0.1.7 through the protected trusted-publishing workflow.

## Issue-derived evidence

The dated [Podlet regression map](research/podlet-regressions-2026-08-01.md) records concrete
syntax, document-set, capability, and generator cases behind these tasks. Issue closure is not
compatibility evidence; exact Podman/systemd documentation and observations remain required.
