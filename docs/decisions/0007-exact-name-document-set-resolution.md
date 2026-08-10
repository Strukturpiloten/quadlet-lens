# ADR 0007: Exact-name document-set resolution

- Status: accepted
- Date: 2026-08-02

## Context

A useful Quadlet application usually contains several files. Native values can refer to `.pod`,
`.network`, `.volume`, `.image`, and `.build` units by basename, but the typed model previously
classified those values without determining whether their targets existed. BoxFerry needs a graph
that can represent both a valid application and an incomplete migration without losing the
authored reference.

Resolution must not depend on the current working directory, installed Quadlet search paths, or a
host filesystem that may differ from the eventual deployment target. Selecting the first of two
same-named files would also hide a deployment ambiguity.

## Decision

1. A document set pairs every typed document with a validated unit-file basename. Paths and
   unsupported suffixes fail closed.
2. The filename suffix must match the document's selected native unit type.
3. Caller-owned `SourceId` values must be unique. Reusing one is a construction error because
   source-labelled diagnostics would otherwise be ambiguous.
4. Native references resolve by exact authored basename only. The library does not search or read
   the filesystem, expand paths, or normalize case.
5. Duplicate basenames remain in the document set so they can produce diagnostics and make
   references explicitly ambiguous; no candidate wins by insertion order.
6. The graph retains every reference with a `Resolved`, `Missing`, or `Ambiguous` state. Only an
   exactly resolved reference becomes a dependency edge.
7. The initial resolvable unit surface is `.container`, `.pod`, `.network`, `.volume`, `.build`,
   and `.image` once their native typed models are available. Classified references to untyped unit
   kinds remain missing rather than being guessed.
8. Graph edges express native resource references only. Generic systemd ordering, cycles, target
   version support, and runtime availability remain separate validation layers.

## Consequences

Callers can inspect an incomplete document set and report precise manual work instead of losing or
guessing relationships. Results are deterministic and independent of the development machine.
Duplicate names and missing files are visible with source spans, while unique source identities
keep diagnostics trustworthy.

The API requires callers to provide basenames explicitly. It does not yet answer whether a graph is
acyclic, whether systemd ordering is sufficient, or whether a target Podman version supports each
referenced form.

## Alternatives considered

Resolving against the host filesystem was rejected because migration inputs and deployment targets
may have different search paths. Choosing the first duplicate was rejected because input ordering
is not compatibility evidence. Rejecting the entire set on the first missing reference was
rejected because conversion tools must be able to report all incomplete relationships together.
