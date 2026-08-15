# ADR 0011: authored environment and systemd target boundaries

- Status: accepted
- Date: 2026-08-15

## Context

The builder-owned container environment plan safely constructs only validated literal directives.
It cannot describe authored `Environment=` syntax, where quote processing, escapes, resets, bare
names, continuations, and `%` specifiers have materially different provenance and certainty.

`Upholds=` is the one reviewed Quadlet capability with a direct systemd release boundary. Podman
version alone cannot establish that a selected target supports the directive.

## Decision

QuadletLens provides a separate `QuadletDocument::container_environment()` semantic view. It
preserves physical directive order and leaves `AuthoredValue` source spelling untouched. The view
recognizes blank resets, literal ASCII `NAME=VALUE` assignments, bare names, and the documented
systemd word/quote/escape subset needed for those forms. Later directives win for explicit lookup;
reset clears prior values and empty values remain literal values. Bare names and `%`-bearing values
are deferred. Malformed or unmodeled input produces recoverable value-free diagnostics. Debug
output redacts environment values.

The view performs no environment-file or secret loading, manager/process/runtime expansion,
command parsing, host access, or `%` expansion.

`PodmanTarget` gains optional caller-supplied `SystemdVersion` context. The catalogue records a
minimum release only for `systemd.unit.upholds`: without the context it is `Unknown`, below 249 it
is `Unsupported`, and 249 or newer retain the ordinary Podman result. Systemd requirements cite a
separate typed `systemd_evidence` collection with versioned URLs and finite systemd release ranges;
each declared minimum must reference evidence covering that release. Podman evidence remains
separate: `CapabilityEvaluation::evidence()` remains Podman-only and
`CapabilityEvaluation::systemd_evidence()` exposes the relevant systemd record identifiers for an
in-coverage capability regardless of missing, too-old, or sufficient systemd context. Unknown,
out-of-coverage, and non-systemd evaluations expose no systemd evidence. QuadletLens neither
probes systemd nor introduces a generic systemd catalogue,
distribution-backport model, or override layer.

## Consequences

Consumers can distinguish exact authored source from a bounded usable environment projection, and
can make `Upholds=` target checks without host coupling. They must still resolve deferred entries
in the appropriate systemd context and cannot treat this API as a runtime environment evaluator.

## Alternatives considered

### Extend `AuthoredValue`

Rejected because source preservation must not imply semantic parsing, effective selection, or
expansion behavior.

### Reuse the builder environment plan

Rejected because builder inputs are already validated literals and cannot represent authored
deferred or malformed forms faithfully.

### Detect the host systemd version

Rejected because parsing and capability evaluation must remain side-effect-free and portable.
