# ADR 0009: validated programmatic generation

- Status: accepted
- Date: 2026-08-03
- Additive amendments: 2026-08-03, 2026-08-05, and 2026-08-06

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
- retains repeated container label assignments in insertion order;
- retains repeated container secret entries for distinct mounted-file and environment exposures;
- retains pod `UserNS` as a singleton distinct from container-level namespace selection;
- retains container `HostName` as an opaque singleton without adding cross-format hostname
  validation or normalization;
- retains separate container and pod `ShmSize` values as opaque singletons without selecting IPC
  modes or importing cross-format size grammar;
- retains repeated container `DropCapability` entries as exact opaque one-line values in insertion
  order without splitting, deduplication, lowercasing, or native capability validation;
- retains repeated container `AddCapability` entries, including empty native reset assignments, as
  exact opaque one-line values in insertion order without splitting, deduplication, lowercasing,
  or native capability validation;
- retains repeated container `Tmpfs` entries, including empty native reset assignments, as exact
  opaque one-line values in insertion order without splitting destination/options text,
  normalization, deduplication, mount-option validation, or conflation with `Volume`;
- retains repeated container `Sysctl` entries, including empty native reset assignments, as exact
  opaque one-line values in insertion order without parsing `name=value`, splitting whitespace,
  normalization, namespace validation, or runtime/kernel interpretation;
- retains repeated container `Ulimit` entries, including empty native reset assignments, as exact
  opaque one-line values in insertion order without splitting, unquoting, parsing
  `TYPE=SOFT[:HARD]`, normalization, resource-name validation, or runtime interpretation;
- retains repeated container `AddDevice` entries, including empty native reset assignments, as
  exact opaque one-line values in insertion order without splitting, unquoting, parsing host and
  container paths or permissions, interpreting whitespace tokenization or a leading `-`, checking
  device existence, or applying runtime semantics;
- retains container `Memory` as an opaque singleton without applying runtime, cgroup, swap,
  page-size, host-memory, rootless, or cross-format semantics;
- rejects native keys from the wrong unit type and repeated singleton keys;
- retains repeatable entries in insertion order;
- emits sections in deterministic `[Unit]`, native, `[Service]`, `[Install]` order;
- rejects NUL bytes and physical line endings in values;
- provides a focused process-ID-limit helper that constructs only `-1` unlimited or nonzero
  ASCII-decimal finite spelling, retaining arbitrary precision without parsing while leaving the
  existing raw `EntryValue` boundary unchanged;
- provides a focused shared-memory-size helper for a non-negative ASCII-decimal amount with an
  optional lowercase `b`, `k`, `m`, or `g`, preserving exact arbitrary-precision spelling and
  explicitly representing zero as Podman's documented unlimited value while leaving parsed and
  raw values opaque;
- provides a focused container-memory helper for a positive ASCII-decimal amount with an optional
  lowercase `b`, `k`, `m`, or `g`, preserving leading zeros and arbitrary precision without
  narrowing the raw `EntryValue` boundary; and
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
