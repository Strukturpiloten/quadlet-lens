# ADR 0013: Bounded native value decoding

Status: accepted
Date: 2026-09-07

## Context

Consumers need structured access to common Quadlet values without losing authored ordering,
reset directives, source spans, or native forms that lack a portable equivalent. Treating every
unknown mount option or unresolved systemd specifier as an error would conflate source decoding
with target validation and cross-format conversion. Conversely, copying Podman's complete command
line semantics into the typed document would make generator behavior an unchecked API promise.

## Decision

QuadletLens provides additive, document-level native views for container and pod port and mount
directives, container commands, and build environment assignments. The views retain directive
order and source spans, model blank directives as resets, and distinguish decoded values from
deferred and malformed values with recoverable diagnostics. `Exec=` uses bounded lexical word
tokenization only: command prefixes and separators are retained as argv rather than interpreted as
systemd executable semantics. `Entrypoint=` accepts JSON string arrays and literal executable
spellings, retaining the source form in `NativeCommand::syntax()`. Long mount options remain
opaque native evidence even when their names are not otherwise modeled.

The `serde_json` dependency is limited to decoding that JSON string-array syntax. It neither
defines a public model representation nor participates in general Quadlet syntax parsing.

The views do not choose a Podman version, expand systemd specifiers, access the filesystem,
validate runtime behavior, or decide portability. Those remain capability, caller-context, and
BoxFerry responsibilities.

## Consequences

BoxFerry and other consumers can reuse one source-aware decoding boundary rather than reparse
Quadlet text. New public types are additive and covered by the external consumer contract. A
valid native form may be retained without being promoted into a portable mapping.

## Alternatives considered

### Decode in BoxFerry

Rejected because source-aware native parsing belongs to QuadletLens and duplicate parsers would
drift on resets, systemd quoting, and source locations.

### Reject unrecognized mount options

Rejected because an option can be valid native evidence without a portable interpretation.

### Validate against a selected Podman target here

Rejected because target support is the capability catalogue's separate, evidence-backed concern.
