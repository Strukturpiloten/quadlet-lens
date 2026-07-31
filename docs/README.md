# QuadletLens documentation

This directory defines QuadletLens's architecture and version-aware behavior.

## Start here

- [Architecture](architecture.md) — syntax, typed models, document sets, and validation flow
- [Project structure](project-structure.md) — intended crate, modules, catalogue, and fixtures
- [Capability model](capability-model.md) — version ranges, evidence, fallbacks, and known issues
- [Testing strategy](testing.md) — syntax, rendering, catalogue, and real-generator tests
- [Dependency and license policy](dependency-policy.md) — dependency selection, allowed sources, and license checks
- [Roadmap](roadmap.md) — implementation order
- [Architecture decisions](decisions/README.md) — durable design choices

## Documentation rules

- Compatibility claims identify an exact Podman/systemd version or range.
- Prefer tagged documentation, tagged source, release notes, and observed generator behavior over assumptions from the latest manual.
- Distinguish syntactic acceptance, generated-command semantics, and runtime success.
- Record known test gaps instead of converting assumptions into facts.
- Use ADRs for representation, catalogue, version-policy, and public-API decisions.

Coding agents must also follow the repository-root `AGENTS.md`.
