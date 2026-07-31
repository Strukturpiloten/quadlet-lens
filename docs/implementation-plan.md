# Cross-repository implementation plan

This plan gives BoxFerry, ComposeLens, and QuadletLens one stable task numbering scheme. Repository roadmaps describe internal phases; this document describes delivery order across repositories.

Last synchronized: 2026-07-31.

## Status convention

- `planned` — scoped but not started
- `in progress` — implementation is currently active
- `completed` — exit criteria are met and validation is documented
- `blocked` — progress requires a named external decision or capability

The repository that owns a task is authoritative for its detailed status. Update the summary copies in the other two repositories whenever a task changes state.

## Program status

| Task | Owner | Status | Deliverable |
| --- | --- | --- | --- |
| T1 | All repositories | completed | Executable testing and fixture foundations |
| T2 | ComposeLens | completed | Loss-aware YAML syntax and diagnostic kernel |
| T3 | QuadletLens | planned | Ordered Quadlet syntax and rendering kernel |
| T4 | BoxFerry | planned | Independent neutral model and conversion engine |
| T5 | All repositories | planned | Minimum native typed subsets for the first conversion |
| T6 | BoxFerry, integrating both Lens libraries | planned | First Compose-to-Quadlet vertical slice |
| T7 | All repositories | planned | Expanded conformance, runtime, and release testing tiers |

## T1: Testing foundations

Status: completed.

The repositories have Cargo-discovered policy tests, versioned fixture manifests, provenance and secret-review rules, immutable GitHub Action checks, stable/MSRV CI execution, and documented suite ownership. Product suites are created only with meaningful behavior.

## T2: ComposeLens YAML syntax kernel

Status: completed. ComposeLens owns this task.

ComposeLens evaluated loss-aware YAML representations, accepted ADR 0002, implemented source and diagnostic primitives, and proved exact preservation and malformed-input recovery on stable Rust and Rust 1.85.0. Its repository copy contains the detailed evidence.

## T3: QuadletLens ordered syntax kernel

Status: planned. QuadletLens owns this task.

Work:

1. Implement ordered sections and entries without collapsing repeated keys.
2. Preserve comments, blank lines, continuations, unknown keys, and generic systemd sections.
3. Preserve `%h` and other systemd specifiers as source values rather than shell substitutions.
4. Add source identifiers, spans, and structured diagnostics.
5. Implement deterministic canonical rendering plus preservation-oriented round trips.
6. Add malformed-input and parser/renderer fixtures.
7. Define the capability schema and establish the Podman 5.4 baseline.

Exit criteria:

- Ordered and repeated source constructs survive parsing and rendering.
- Invalid input returns source-spanned diagnostics without panicking.
- Capability entries express minimum/maximum versions, evidence, fallbacks, and known-bug ranges.
- Podman 5.4 baseline claims have tests or explicit evidence gaps.
- Public syntax and diagnostic primitives compile on Rust 1.85.0.

## T4: BoxFerry independent conversion core

Status: planned. BoxFerry owns this task.

Implement neutral application, service, volume, network, port, environment, and tolerant image-reference models; provenance and redacted diagnostics; exact, approximate, unsupported, and invalid outcomes; target version ranges; adapter contracts; and an in-memory adapter. This task does not depend on unfinished Lens APIs.

## T5: Minimum native typed subsets

Status: planned. Each repository owns its native types; BoxFerry owns mappings.

- ComposeLens: services, images, commands, environment, ports, volumes, networks, and profiles.
- QuadletLens: `.container`, `.volume`, `.network`, and required generic systemd sections.
- BoxFerry: mappings, path-policy differences, and Podman 5.4 fallback decisions.

Before integration, document dependency and release mechanics. Prefer early pre-1.0 Lens releases; use commit-pinned Git dependencies only as a temporary fallback.

## T6: First end-to-end milestone

Status: planned. BoxFerry coordinates this task.

Deliver tested Compose-to-Quadlet conversion for images, commands, environment, ports, named volumes, bind mounts, networks, and explicit Compose profile selection. Every conversion emits compatibility and manual-action reports. After synthetic scenarios are stable, use `Strukturpiloten/typo3-container` as the first public real-world showcase and regression corpus.

## T7: Expanded testing tiers

Status: planned.

- Per pull request: unit, integration, golden, round-trip, and property tests.
- Scheduled: Docker Compose, Podman Compose, and real Quadlet generator conformance.
- Release validation: supported Podman matrices, rootless/rootful contexts, real-world projects, and eventually disposable Kubernetes clusters.

Each harness becomes required only after its command, isolation model, version source, fixture provenance, and failure policy are documented.
