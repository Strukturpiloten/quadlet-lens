# ADR 0002: loss-aware ordered systemd-style syntax

- Status: accepted
- Date: 2026-08-02

## Context

Podman 5.4 documents that Quadlet files use the same format as regular systemd unit files and that
ordinary systemd sections pass through to systemd. The systemd syntax contract uses ordered
sections and `key=value` entries, ignores whitespace adjacent to `=`, permits both `#` and `;`
comments, and joins lines ending in a backslash. Comment blocks inside a continued logical line are
ignored while continuation proceeds.

Quadlet and systemd keys may be repeated, and ordering can affect generated Podman arguments or
systemd list/reset semantics. Values may contain systemd specifiers such as `%h` and `%n`; treating
them as Compose interpolation or shell syntax would change meaning. A deserialized INI map would
therefore lose required syntax and provenance before typed Quadlet processing begins.

Evidence reviewed for this decision:

- [Podman 5.4 `podman-systemd.unit`](https://docs.podman.io/en/v5.4.0/markdown/podman-systemd.unit.5.html),
  including the regular-systemd-format statement, repeated-key examples, and `%` path behavior;
- [current `systemd.syntax`](https://www.freedesktop.org/software/systemd/man/systemd.syntax.html),
  reviewed 2026-08-02; and
- the same comments, assignment, and continuation rules as published for systemd 252, the systemd
  baseline relevant to Debian 12 environments.

## Decision

1. QuadletLens starts with a custom, dependency-free physical-line syntax kernel rather than an
   unordered INI parser.
2. The immutable `SyntaxDocument` retains the complete source text and every physical line in
   order, including blank lines, comment markers, CRLF/LF spelling, and a missing final line ending.
3. Section names, entry keys and values, comments, and continuation fragments use QuadletLens-owned
   source spans. Repeated sections and keys remain separate source items.
4. Continuation context is explicit. Comments inside a continuation remain preserved and marked as
   continuation comments; non-comment continuation fragments are not reparsed as new entries or
   sections.
5. `%h` and all other specifier-shaped text remain ordinary authored value bytes. Expansion and
   path classification belong to later typed and target-aware layers.
6. Recoverable invalid lines, empty section names, entries before a section, empty keys, and
   dangling continuations return stable structured diagnostics alongside a renderable document.
7. The first parser does not unquote values, interpret C escapes, apply reset behavior, type keys,
   merge drop-ins, or emulate the Podman generator. Those are explicit later layers with their own
   evidence and tests.
8. Preservation rendering returns the original bytes. Deterministic canonical rendering is added
   only after its comment, continuation, quoting, and typed-value semantics are separately defined.

## Consequences

- The initial syntax tree is larger than a section-to-map representation but retains the evidence
  needed by editing, diagnostics, and target conversion.
- Unknown Quadlet keys and generic systemd sections survive without needing a complete key list.
- Source lines can be analyzed safely before the capability catalogue or typed model exists.
- Logical-value decoding remains open work; callers cannot mistake raw continued or quoted text
  for a normalized systemd argument list.
- A later incremental or editable tree may supplement this representation without changing the
  source/diagnostic primitives.

## Alternatives considered

### Generic INI deserialization

Rejected because common map-oriented APIs collapse repeated keys and sections, discard comments
and line endings, and cannot preserve continuation comment blocks.

### Parse directly into typed Quadlet structures

Rejected because unknown future keys, generic systemd sections, and target-version differences
must remain available even when the current typed model does not recognize them.

### Reuse a full systemd implementation

Rejected for the initial kernel because QuadletLens must not invoke systemd, and available parser
interfaces do not demonstrate the required loss-aware public representation. Exact systemd and
Podman generators remain external behavior oracles for conformance tests.
