# ADR 0021: Protected inert extension sections

- Status: accepted
- Date: 2026-10-27
- Extends: [ADR 0009](0009-validated-programmatic-generation.md) and
  [ADR 0005](0005-source-aware-native-typed-model.md)

## Context

Consumers need to retain opaque vendor metadata without turning it into an active systemd
configuration directive. Unknown-section parsing already preserves native syntax, but a
construction API and protected display boundary are needed for payloads that may contain
sensitive values. Using printable `EntryValue` would expose the payload before section context
could protect it.

## Decision

Provide validated `XSectionName`, protected `ExtensionValue`, and an ordered
`push_extension_entry` builder API. Section names have a strict ASCII `X-*` grammar and keys
are ASCII alphanumeric. Values reject NUL, physical line endings, and final backslashes even
before trailing whitespace. Errors contain no attempted input. Extension entries use separate
protected storage, preserve repeated-entry order, and group sections lexically after standard
sections. Build rejects native/systemd values with final backslashes when extension entries are
present, so an earlier continuation cannot consume their section header. The complete output
still passes through ordinary parse-back validation.

Conservatively treat all parsed values in source containing any physical `X-*` header candidate
as sensitive, without knowing vendor names or schemas. This display-only over-redaction also
protects malformed headers and headers consumed by preceding continuations, without adding a
second semantic parser or changing source interpretation. Protect typed values and their continuation segments as well as whole results.
Derived native views must also protect copied source strings. Carry private sensitivity metadata
into port specifications, environment-secret references, and unit references. Preserve ordinary
Debug for unprotected source. Public mount-option and authored-environment enums cannot carry
sensitivity metadata without breaking variant shapes, so their Debug conservatively redacts all
copied names and literals. Explicit access and semantic results remain unchanged.

Source Debug omits raw text whenever an X-section header occurs, protecting fresh syntax parses
and malformed source independently of typed interpretation. Explicit raw/source/value/render
access remains available. Existing syntax and semantic validation stay separate.

Generator fixtures check native dry-run pass-through and that payloads do not become active
configuration. Only a passing exact pinned generator run is observed generator evidence; no
runtime-version support or unit execution follows from an authored expectation. Never install
or execute generated units. JSON framing and cross-format vendor semantics stay outside the
library.

## Consequences

- Consumers can construct safe native extension sections without assembling complete files.
- Repeated values and deterministic grouping remain explicit.
- Debug is conservative; explicit byte access still requires caller-controlled disclosure.
- No dependencies, runtime acquisition, or cross-format model are introduced.
- The final-backslash restriction is deliberately conservative, including paired backslashes.

## Alternatives

Generic arbitrary-section generation would allow active directives and weaken the structural
boundary. Reusing `EntryValue` would expose protected payloads through its standalone Debug.
Protecting only builder Debug would leave freshly parsed and public subobject Debug paths open.
Schema-aware protection would require vendor knowledge outside QuadletLens ownership.
