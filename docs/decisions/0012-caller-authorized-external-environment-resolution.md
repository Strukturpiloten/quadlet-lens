# ADR 0012: caller-authorized external environment resolution

- Status: accepted
- Date: 2026-08-26

## Context

ADR 0011 established a source-aware inline `Environment=` view without host access. BoxFerry also
needs to explain `EnvironmentFile=` and environment-exposing `Secret=` inputs. Resolving either from
ambient state during parsing would make results host-dependent and could disclose protected values.

Generated output benefits from deterministic environment-key order, but Quadlet source permits
repeated assignments, groups, and resets whose physical order must remain available.

## Decision

QuadletLens exposes external environment references separately from their values. Discovery retains
source spans and classifies each reference as literal, deferred by a systemd specifier, or
unmodeled. It never reads a file, process environment, Podman API, or secret store.

External resolution accepts only decoded values explicitly supplied by the caller. Exact literal
references may resolve; missing, deferred, and unmodeled references remain unresolved. Missing and
authorized empty values are distinct. Protected payloads use a wrapper with redacted `Debug`, no
`Display`, and an explicit exposure method.

Parsed and canonical rendering continue preserving authored order. Generated literal environment
plans gain an opt-in stable name sort. Sorting is limited by explicit resets, preserves duplicate
same-name order and last-wins behavior, and may normalize assignment groups into individual
directives. Default generation remains insertion-ordered.

QuadletLens reports native evidence only. BoxFerry owns authorization prompts or flags, neutral-model
promotion, loss policy, and cross-format secret representation.

## Rejected alternatives

### Read referenced files or Podman secrets automatically

Rejected because parsing would gain ambient authority, become non-deterministic, and risk secret
disclosure.

### Parse environment-file bytes inside QuadletLens

Rejected because the native document contains only a reference. The caller owns acquisition,
encoding, and any file-format policy and supplies decoded assignments.

### Sort every `Environment=` entry during rendering

Rejected because source preservation requires physical repetition, grouping, resets, quoting, and
order to remain unchanged. Sorting is caller-authorized and limited to validated generated literals.
