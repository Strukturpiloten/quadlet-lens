# ADR 0004: versioned evidence-backed capability catalogue

- Status: accepted
- Date: 2026-08-02

> Amended by [ADR 0006](0006-rolling-support-window-and-generator-evidence.md): Podman 5.4.0 is the
> fixed support minimum, while finite catalogue and generator evidence can expand toward rolling
> current. The single-version clauses below describe the initial bootstrap catalogue.

## Context

Quadlet features, fallbacks, deprecations, removals, and patch regressions must be evaluated across
an explicit Podman minimum and optional maximum. Rust enums or version checks distributed through
the typed model would be hard to review, update, and audit against evidence. Documentation evidence
also must not be presented as if an exact generator had executed successfully.

## Decision

1. Capability data uses a strict versioned TOML schema under `catalogue/v1/` and stable namespaced
   semantic identifiers.
2. The runtime parser uses `serde` 1.0 and `toml` 1.1. These focused, actively maintained,
   crates.io dependencies are compatible with the MPL-2.0 project and Rust 1.85 MSRV policy.
3. Catalogue coverage is finite and distinct from upstream feature introduction. The first file
   covers exactly Podman 5.4.0; it does not claim that listed features were introduced in 5.4.
4. Records describe applicable unit types and sections, required/repeatable behavior, value forms,
   native ranges, optional fallback ranges, deprecation/removal boundaries, known broken ranges,
   and evidence identifiers.
5. Evidence distinguishes primary documentation review from exact-version generator execution.
   Documentation-only records require an explicit evidence gap.
6. Evaluation is fail-closed outside finite catalogue coverage. When
   `podmanMaximumVersion` is omitted, evaluation stops at the newest catalogue version and reports
   later versions as untested assumptions.
7. Known broken ranges take precedence over native support. A fallback is selected only when it
   covers the complete requested range.

## Evidence for the first baseline

The Podman 5.4 [`podman-systemd.unit` manual](https://docs.podman.io/en/v5.4.0/markdown/podman-systemd.unit.5.html)
documents regular systemd-section pass-through and the `.container`, `.pod`, `.network`, and
`.volume` surfaces used by the first conversion. Each catalogue evidence record links to a narrower
manual section and states that exact 5.4 generator verification remains open.

## Consequences

- Capability updates are reviewable data changes with executable schema and boundary tests.
- Downstream callers can distinguish native, fallback, deprecated, removed, unsupported, unknown,
  and broken results without embedding Podman-version conditionals.
- Podman 5.3 and 5.5 are intentionally unknown until their own evidence is added.
- Exact generator matrices remain required before documentation-only claims are promoted.

## Alternatives considered

Hard-coded version enums were rejected because every new Podman release would require application
logic changes. Unvalidated loose TOML was rejected because unknown fields and incoherent ranges
could silently broaden compatibility claims. Treating current documentation as open-ended support
was rejected because absence of a maximum is not evidence for future releases.
