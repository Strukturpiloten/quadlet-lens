# ADR 0008: Versioned public API and release contract

- Status: accepted
- Date: 2026-08-02

## Context

QuadletLens now exposes the native stages required by the first BoxFerry Quadlet adapter:
loss-aware syntax, source-aware typed documents, exact document-set relationships, and a
versioned Podman capability catalogue. Keeping the crate unpublished would force BoxFerry to use a
sibling path or Git dependency and would prevent independent consumers from relying on a named
compatibility boundary.

The implemented surface is intentionally incomplete. Health/readiness semantics, typed
target-aware rendering, broader unit types, and runtime verification remain later work. A public
release therefore needs a narrow pre-1.0 promise rather than either an unbounded experiment or a
premature 1.0 contract.

## Decision

QuadletLens will publish one library crate with a supported `0.1.x` API line.

- Patch releases preserve the documented module paths and supported source compatibility.
- Intentional public breaks require the next 0.x minor version and migration guidance.
- Public interfaces use QuadletLens-owned types.
- Syntax, typed models, document sets, and capability evaluation remain explicit stages.
- Diagnostic codes, byte-preserving rendering, and deterministic canonical syntax rendering are
  behavioral contracts.
- Capability evaluation remains finite and evidence-backed; a library release does not broaden
  the recorded Podman evidence range.
- A consumer-facing integration test compiles and executes the supported path.
- Release archives retain source, tests, fixtures, catalogue evidence, generator-matrix records,
  and project documentation so compatibility claims remain auditable.
- CI builds rustdoc with warnings denied and verifies the crates.io package before publication.

The exact guarantees and exclusions live in the versioned
[API stability policy](../api-stability.md).

## Consequences

BoxFerry can consume QuadletLens from crates.io without coupling repositories. Other Rust tools can
use the parser and capability catalogue independently. The project can still add missing behavior
within the supported contract and can redesign public interfaces in 0.2 with documented migration
costs.

The release archive is intentionally larger than a minimal source-only package because evidence
is part of the compatibility claim. Unfinished runtime and typed-rendering work must remain visible
in release notes and roadmaps.

## Alternatives considered

### Keep QuadletLens unpublished until every Quadlet feature is implemented

Rejected because completeness is not a realistic precondition for useful integration, and it
would force temporary repository coupling.

### Publish without an API stability policy

Rejected because downstream users need to know which patch-level changes are safe.

### Publish separate crates for syntax, model, and capability data

Rejected for 0.1 because the layers have architectural boundaries but currently share one release
need and one evidence corpus.
