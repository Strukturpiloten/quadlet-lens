# ADR 0009: Validated programmatic generation

- Status: accepted
- Date: 2026-08-03

## Context

BoxFerry and direct consumers need to construct native Quadlet without assembling section names,
key spellings, and complete files as strings. The typed model intentionally leaves most Podman and
systemd value grammars opaque, so construction must not overstate semantic validation.

## Decision

QuadletLens provides `QuadletDocumentBuilder` as the supported construction path. The builder:

- selects one supported native unit type;
- accepts typed native keys and open-ended generic systemd directives;
- enforces section ownership and singleton or repeatable cardinality;
- retains repeatable entries in insertion order;
- emits deterministic section and entry order;
- rejects NUL bytes and physical line endings in one-line values; and
- reparses generated text through the syntax and native-model pipeline before success.

`EntryValue` is exact, physical-line-safe text. The builder does not generally quote, split,
expand, normalize, or validate key-specific values.

Focused value helpers may add stronger construction when exact evidence defines a useful boundary.
They remain additive and do not narrow parsed or raw values. The environment helpers and
`ContainerEnvironmentPlan` preserve authored directive order, expose only explicit literal
projection, and redact repository-owned debug output.

Sensitive values remain exact for explicit access and rendering while keyed repository-owned debug
representations redact them.

Generation performs no filesystem discovery, host lookup, Podman invocation, systemd operation, or
runtime mutation. Target-version evaluation and BoxFerry conversion policy remain separate.

## Consequences

- Native spelling, placement, cardinality, and validation stay inside QuadletLens.
- BoxFerry can construct documents without depending on parser internals.
- Callers remain responsible for selecting a compatible semantic value representation.
- Dedicated helpers can grow without replacing the exact-value boundary.
- Successful output and its typed parse share one source identifier and source text.

## Alternatives considered

### Construct raw files in BoxFerry

Rejected because it duplicates native syntax and validation policy in a conversion layer.

### Expose mutable parser internals

Rejected because it couples consumers to physical representation and complicates source-span
consistency.

### Wait for complete value grammars

Rejected because safe structural generation is useful before every Podman and systemd value has a
dedicated parser.
