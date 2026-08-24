# ADR 0006: Rolling support window and ranged generator evidence

- Status: accepted
- Date: 2026-08-02

## Context

ADR 0004 established a finite, fail-closed capability catalogue at the Podman 5.4.0 baseline. The
product must distinguish its fixed minimum, the newest upstream target, finite reviewed catalogue
coverage, and capability-specific generator evidence.

Official immutable Podman images do not exist for every release. Testing one release or one key
does not justify a wider version or runtime claim.

## Decision

QuadletLens uses:

- Podman 5.4.0 as its fixed minimum;
- the newest stable upstream release as its rolling target;
- a finite catalogue ceiling that advances only with reviewed evidence;
- exact patch-level ranges for capability claims and known bugs; and
- generator evidence only when every release in the claimed range is executed.

Exact releases and execution sources are data in
[`tools/generator-matrix.toml`](../../tools/generator-matrix.toml). Prefer digest-pinned official
images. When none exists, build only the standalone generator from the exact recorded release commit
inside a pinned builder.

Documentation, tagged source, generator output, and runtime behavior remain distinct evidence
levels. A generator run proves generated text, not systemd activation or workload behavior.

Capabilities outside reviewed coverage evaluate as unknown. A newly released Podman version is a
tracked target before it becomes evidence. Optional systemd requirements use separate versioned
evidence and caller-supplied target context.

QuadletLens never probes the installed host or infers distribution backports. Overrides require a
future explicit contract and architectural decision.

## Consequences

- Portable output can be checked over an explicit minimum and maximum.
- The catalogue fails closed while upstream targets continue to advance.
- Missing registry images do not force gaps in generator coverage.
- Source builds cost more than image runs and require immutable commits and a pinned toolchain.
- Runtime claims need a separately documented environment and test tier.

## Alternatives considered

### Treat the catalogue ceiling as the product maximum

Rejected because target tracking and reviewed evidence move at different speeds.

### Test only minor releases

Rejected because patch releases can change parsing and generated output.

### Infer support from the local installation

Rejected because results would depend on distribution backports and hidden host state.
