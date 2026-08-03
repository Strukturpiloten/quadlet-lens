# ADR 0009: validated programmatic generation

- Status: accepted
- Date: 2026-08-03
- Additive amendment: 2026-08-03

## Context

BoxFerry and other library consumers need to create native Quadlet documents without assembling
section names and key spellings themselves. QuadletLens 0.1.0 could parse and render authored
documents, but it exposed no supported construction path. Generating raw Quadlet text in an
adapter would put native syntax policy outside the library that owns it.

The existing typed model intentionally keeps most systemd and Podman value grammars opaque. A
first construction API must not imply that quoting, command arguments, environment assignments,
ports, or mount options have already been fully modeled.

## Decision

QuadletLens provides a programmatic document builder that:

- selects one supported native unit type;
- accepts typed native keys and open-ended generic systemd directives;
- accepts typed `Requires`, `Wants`, and `After` `[Unit]` directives for the evidence-backed
  dependency subset while retaining open-ended generic systemd construction;
- retains repeated supplementary-group entries while keeping the other execution-identity keys
  singleton;
- rejects native keys from the wrong unit type and repeated singleton keys;
- retains repeatable entries in insertion order;
- emits sections in deterministic `[Unit]`, native, `[Service]`, `[Install]` order;
- rejects NUL bytes and physical line endings in values; and
- reparses generated text and returns it only when syntax and native-model validation succeed.

Values remain exact, already-semantic native values. The builder does not quote, split, expand, or
normalize key-specific value grammars. Focused value encoders may be added later when their rules
are protected by documentation and generator evidence.

## Consequences

- Native section/key spelling and structural validation stay inside QuadletLens.
- BoxFerry can generate typed native documents without depending on parser internals or composing
  complete files as strings.
- Callers remain responsible for choosing a valid semantic representation for each value.
- Adding value-specific encoders is additive and does not require replacing this document-level
  boundary.
- The returned syntax and typed model share the same generated source and source identifier.

## Alternatives considered

### Construct raw text in BoxFerry

Rejected because it duplicates native spelling, ordering, repeatability, and validation policy in
the conversion layer.

### Expose mutable parser internals

Rejected because physical syntax mutation would couple consumers to representation details and
make source-span consistency difficult to guarantee.

### Wait for complete typed value grammars

Rejected because complete systemd and Podman value modeling is much larger than the safe first
conversion subset. The exact-value boundary permits useful generation without overstating what the
library understands.
