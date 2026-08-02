# ADR 0005: Source-aware native typed model

Status: accepted

Date: 2026-08-02

> Extended by [ADR 0007](0007-exact-name-document-set-resolution.md): the typed surface now includes
> `.pod`, and exact-name document-set reference resolution and dependency edges are implemented.
> The initial and deferred wording below records the state when this ADR was accepted.

## Context

The syntax kernel deliberately preserves physical source structure without interpreting values.
BoxFerry and other library consumers also need to identify the native Quadlet fields used by the
first conversion without losing unknown keys, repeated sections, generic systemd configuration, or
the spelling that a future preservation-oriented editor needs.

Flattening a unit into maps would discard ordering and repeated keys. Fully decoding systemd
command, environment, and mount syntax before exact generator tests exist would turn unverified
assumptions into API promises. Tying native model enums directly to Podman versions would duplicate
the capability catalogue and make distribution backports difficult to represent.

## Decision

QuadletLens exposes a typed view layered over, but independent from, the loss-aware syntax result:

- the first explicit unit types are `.container`, `.network`, and `.volume`;
- supported generic systemd sections are classified while their open-ended keys stay authored;
- recognized native keys use enums, and unknown keys and sections remain ordered entries;
- every section name, key, primary value, and continuation segment owns its authored text and
  source span;
- values are only classified where the distinction is safe and useful: opaque, lexical path form,
  or native unit reference;
- systemd command lines, environment assignments, ports, health commands, and Podman arguments
  remain opaque until dedicated value parsers have evidence-backed behavior;
- a combined parse result retains both the complete syntax document and separate model
  diagnostics;
- the model recognizes syntax independently from target-version support. The capability catalogue
  remains authoritative for Podman version claims.

Unit types and suffixes are explicit and fail closed. Unsupported unit types are not treated as a
supported generic document.

## Consequences

BoxFerry can map the minimum native surface using stable key and reference categories without
depending on parser internals. Callers can always return to preserved source and can distinguish
syntax errors from native-model errors. Future value parsers can refine opaque forms without
forcing the syntax tree to change.

The initial typed model is intentionally not a complete semantic representation. Cross-document
reference resolution, dependency graphs, target-aware validation, typed rendering, and exact
systemd value decoding remain separate milestones. Owned authored strings use more memory than
borrowing source slices, but avoid a lifetime-coupled public document and keep source provenance
explicit.

## Alternatives considered

### One flattened map per section

Rejected because it loses repeated keys, repeated sections, and authored order.

### Fully decode every known value immediately

Rejected because systemd and Podman argument semantics need exact-version generator evidence, and
an incorrect parser would be more dangerous than an explicit opaque value.

### Put target versions on model variants

Rejected because syntax recognition and target capability are different questions. Version
coverage belongs to the data-driven catalogue.

### Borrow all strings from the syntax document

Deferred because it would make the public typed model lifetime-coupled and harder for external
applications to store. Source spans still link every copied authored value to its origin.
