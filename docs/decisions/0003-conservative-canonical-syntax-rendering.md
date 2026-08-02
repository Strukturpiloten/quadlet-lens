# ADR 0003: conservative canonical syntax rendering

- Status: accepted
- Date: 2026-08-02

## Context

Preservation rendering already returns the exact source bytes. QuadletLens also needs deterministic
output for generated or normalized files, but the syntax layer does not yet decode systemd quoting,
command arguments, resets, or key-specific path semantics. Sorting sections, joining continuations,
or rewriting values at this layer could therefore change behavior.

## Decision

1. Canonical syntax rendering is available only from a parse result with no error diagnostics.
2. It preserves physical-line order, repeated keys and sections, comment markers and bodies,
   blank-line count, continuation structure, raw value fragments, quote spelling, and `%`
   specifiers.
3. It removes structural indentation before sections, keys, comments, and continuation fragments;
   emits exactly one `=` between the trimmed key and retained value; uses LF; and terminates every
   non-empty physical line.
4. It does not sort, merge, decode, unquote, expand, or infer typed values.
5. Invalid syntax is refused with the original structured diagnostics instead of producing output
   that could appear authoritative.

## Consequences

- Canonical output is deterministic and parse-render-parse idempotent without claiming semantic
  normalization of systemd values.
- A later typed renderer may choose section order and encode native values, while this renderer
  remains useful for safe structural normalization and testing.
- Generator fixtures remain necessary before quote and command-argument equivalence is claimed.

## Alternatives considered

Sorting and merging syntax entries was rejected because ordering and reset behavior can matter.
Returning canonical output for recoverable invalid input was rejected because the renderer cannot
know which correction the author intended.
