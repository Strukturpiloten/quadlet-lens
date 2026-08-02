# Cross-repository implementation plan

This plan gives BoxFerry, ComposeLens, and QuadletLens one stable task numbering scheme. Repository roadmaps describe internal phases; this document describes delivery order across repositories.

Last synchronized: 2026-08-02.

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
| T3 | QuadletLens | completed | Ordered Quadlet syntax and rendering kernel |
| T4 | BoxFerry | completed | Independent neutral model and conversion engine |
| T5 | All repositories | in progress | Minimum native typed subsets for the first conversion |
| T6 | BoxFerry, integrating both Lens libraries | in progress | First Compose-to-Quadlet vertical slice |
| T7 | All repositories | in progress | Expanded conformance, runtime, and release testing tiers |

## T1: Testing foundations

Status: completed.

The repositories have Cargo-discovered policy tests, versioned fixture manifests, provenance and secret-review rules, immutable GitHub Action checks, stable/MSRV CI execution, and documented suite ownership. Product suites are created only with meaningful behavior.

## T2: ComposeLens YAML syntax kernel

Status: completed. ComposeLens owns this task.

ComposeLens evaluated loss-aware YAML representations, accepted ADR 0002, implemented source and diagnostic primitives, and proved exact preservation and malformed-input recovery on stable Rust and Rust 1.85.0. Its repository copy contains the detailed evidence.

## T3: QuadletLens ordered syntax kernel

Status: completed. QuadletLens owns this task.

Work:

1. Implement ordered sections and entries without collapsing repeated keys.
2. Preserve comments, blank lines, continuations, unknown keys, and generic systemd sections.
3. Preserve `%h` and other systemd specifiers as source values rather than shell substitutions.
4. Add source identifiers, spans, and structured diagnostics.
5. Implement deterministic canonical rendering plus preservation-oriented round trips.
6. Add malformed-input and parser/renderer fixtures.
7. Define the capability schema, establish Podman 5.4 as the minimum, and expand evidence toward rolling current.

Exit criteria:

- Ordered and repeated source constructs survive parsing and rendering.
- Invalid input returns source-spanned diagnostics without panicking.
- Capability entries express minimum/maximum versions, evidence, fallbacks, and known-bug ranges.
- Podman 5.4 minimum claims have tests or explicit evidence gaps, and later targets fail closed until verified.
- Public syntax and diagnostic primitives compile on Rust 1.85.0.

Completion evidence: accepted ADRs 0002–0004; exact preservation and conservative canonical
rendering; source locations and structured recovery; lexical path/specifier classification; a
bounded generated property corpus; the strict versioned TOML catalogue schema; and Podman 5.4.0 as
the support floor. The generator harness now verifies the first-conversion subset on all 20 patch
releases through current 6.0.2, using 14 digest-pinned official images and six exact source builds.
Capability and boundary tests fail closed below 5.4 and above verified coverage, exercise fallback
and known-bug precedence synthetically, and retain explicit gaps for unverified capabilities.

## T4: BoxFerry independent conversion core

Status: completed. BoxFerry owns this task.

Implement neutral application, service, volume, network, port, environment, and tolerant image-reference models; provenance and redacted diagnostics; exact, approximate, unsupported, and invalid outcomes; target version ranges; adapter contracts; and an in-memory adapter. This task does not depend on unfinished Lens APIs.

BoxFerry has implemented the public library facade, neutral application graph, provenance,
protected values, redacted structured diagnostics, version-bounded target profiles, validated
fidelity outcomes, explicit loss authorization, adapter contracts, and in-memory adapter. Its
stable and Rust 1.85.0 tests exercise the same orchestration available to external Rust projects.
The component crates remain unpublished until their release contract is finalized.

## T5: Minimum native typed subsets

Status: in progress. Each repository owns its native types; BoxFerry owns mappings.

- ComposeLens (completed): services, images, commands, environment, ports, volumes, networks, profiles, configs, and secrets.
- QuadletLens (completed): `.container`, `.pod`, `.volume`, `.network`, required generic systemd sections, and exact document-set relationships.
- BoxFerry (in progress): Compose-to-neutral mappings are implemented for the first subset;
  Quadlet export, path policy, and Podman 5.4-to-current fallback decisions remain.

ComposeLens has completed its T5 native subset with source-aware typed resources, tolerant image
references, deferred values, and representation-preserving command, environment, port, volume,
network, profile, config, secret, and label forms. QuadletLens has completed its first native subset
with ordered source-aware `.container`, `.pod`, `.network`, and `.volume` documents, generic systemd
and unknown entry preservation, native key enums, conservative path/reference forms, separate
syntax/model diagnostics, and exact document-set dependency resolution. BoxFerry now consumes
ComposeLens 0.1.1's native merged-project view through its independent `boxferry-compose` crate. The adapter maps
images, commands, environment, single ports, named volumes, bind mounts, networks, explicit
profiles, provenance, and short/long SELinux relabel intent into the neutral model. Source
omissions are structured outcomes governed by `LossPolicy`, not warning-only side effects.

ComposeLens 0.1.1 is published on crates.io with a documented pre-1.0 compatibility contract.
BoxFerry will consume released Lens crates through compatible crates.io requirements and commit its
application lockfile. Commit-pinned Git dependencies remain an emergency-only fallback.

QuadletLens's first crates.io release is now the dependency gate for `boxferry-quadlet`; BoxFerry
does not use a temporary sibling path or Git dependency. QuadletLens 0.1.0 now has a documented
public API, consumer contract test, verified package, and trusted-publishing release workflow. Its
one-time crates.io bootstrap and first GitHub release remain external maintainer actions.

## T6: First end-to-end milestone

Status: in progress. BoxFerry coordinates this task. Compose import is implemented; Quadlet export
and the combined compatibility report remain.

Deliver tested Compose-to-Quadlet conversion for images, commands, environment, ports, named volumes, bind mounts, networks, and explicit Compose profile selection. Every conversion emits compatibility and manual-action reports. After synthetic scenarios are stable, use `Strukturpiloten/typo3-container` as the first public real-world showcase and regression corpus.

## T7: Expanded testing tiers

Status: in progress. ComposeLens has delivered its repository tier. QuadletLens has an exact
Podman 5.4-to-current generator matrix, and BoxFerry has its first provenance-reviewed Compose
adapter fixture; broader BoxFerry tiers remain.

- Per pull request: unit, integration, golden, round-trip, and property tests.
- Scheduled: Docker Compose, Podman Compose, and real Quadlet generator conformance.
- Release validation: supported Podman matrices, rootless/rootful contexts, real-world projects, and eventually disposable Kubernetes clusters.

Each harness becomes required only after its command, isolation model, version source, fixture provenance, and failure policy are documented.
