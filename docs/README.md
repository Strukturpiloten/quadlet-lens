# QuadletLens documentation

This directory defines QuadletLens's architecture and version-aware behavior.

## Start here

- [Architecture](architecture.md) — syntax, typed models, document sets, and validation flow
- [Project structure](project-structure.md) — intended crate, modules, catalogue, and fixtures
- [Native typed model](typed-model.md) — supported units, source retention, value boundary, and diagnostics
- [Native coverage](coverage.md) — syntax, typed-builder, capability, and generator coverage
- [Programmatic generation](generation.md) — typed document builder, exact-value boundary, and validation
- [Capability model](capability-model.md) — version ranges, evidence, fallbacks, and known issues
- [Podman generator matrix](generator-matrix.md) — support window, exact images, commands, and evidence gaps
- [Testing strategy](testing.md) — syntax, rendering, catalogue, and real-generator tests
- [Real-world Quadlet corpus](real-world-quadlet-corpus.md) — pinned ecosystem projects and feature pressure
- [Development environment](development-environment.md) — reproducible VS Code tooling and update policy
- [API stability](api-stability.md) — supported 0.1.x consumer contract and exclusions
- [Release process](releasing.md) — trusted publishing, repository setup, and recovery
- [Fixture format](fixture-format.md) — shared metadata, provenance, and secrets contract
- [Podlet regression map](research/podlet-regressions-2026-08-01.md) — native syntax, document-set, and capability lessons from user reports
- [Dependency and license policy](dependency-policy.md) — dependency selection, allowed sources, and license checks
- [Implementation plan](implementation-plan.md) — synchronized cross-repository tasks T1–T8
- [Roadmap](roadmap.md) — implementation order
- [0.1.12 release notes](releases/0.1.12.md) — container logging, networking, and volume settings
- [0.1.11 release notes](releases/0.1.11.md) — networking, metadata, and security settings
- [0.1.10 release notes](releases/0.1.10.md) — lifecycle, resource, capability, and device settings
- [0.1.9 release notes](releases/0.1.9.md) — explicit Podman container names
- [0.1.6 release notes](releases/0.1.6.md) — pod user namespaces and container secrets
- [0.1.5 release notes](releases/0.1.5.md) — container execution identity and context
- [0.1.0 release notes](releases/0.1.0.md) — initial public-delivery scope and known limits
- [Architecture decisions](decisions/README.md) — durable design choices

## Documentation rules

- Compatibility claims identify an exact Podman/systemd version or range.
- Prefer tagged documentation, tagged source, release notes, and observed generator behavior over assumptions from the latest manual.
- Distinguish syntactic acceptance, generated-command semantics, and runtime success.
- Record known test gaps instead of converting assumptions into facts.
- Use ADRs for representation, catalogue, version-policy, and public-API decisions.

Coding agents must also follow the repository-root `AGENTS.md`.
