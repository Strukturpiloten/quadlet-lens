# QuadletLens maintainer guide

Use this index to open only the document needed for the current task. Public library guides live
in [`public/`](public/index.md) and are published on boxferry.dev.

## Choose a task

Environment-file and secret work starts with
[Environment and secrets](environment-and-secrets.md), then uses the typed-model and generation
guides for the affected boundary.

| Task                                      | Read                                                          |
| ----------------------------------------- | ------------------------------------------------------------- |
| Understand boundaries or module ownership | [Architecture](architecture.md)                               |
| Parse or inspect Quadlet                  | [Typed model](typed-model.md)                                 |
| Construct a document                      | [Programmatic generation](generation.md)                      |
| Evaluate or update target support         | [Capability model](capability-model.md)                       |
| Run exact Podman generators               | [Generator matrix](generator-matrix.md)                       |
| Add or select tests                       | [Testing](testing.md) and [fixture format](fixture-format.md) |
| Refresh the external corpus               | [Real-world corpus](real-world-quadlet-corpus.md)             |
| Set up the repository                     | [Development environment](development-environment.md)         |
| Change dependencies                       | [Dependency policy](dependency-policy.md)                     |
| Assess a public API change                | [API stability](api-stability.md)                             |
| Prepare or recover a release              | [Release process](releasing.md)                               |
| See current and deferred work             | [Roadmap](roadmap.md)                                         |
| Change an architectural decision          | [ADR index](decisions/README.md)                              |

## Sources of truth

Prose explains contracts and procedures. Exact inventories belong to checked data and tests:

| Claim                                 | Authoritative source                                                              |
| ------------------------------------- | --------------------------------------------------------------------------------- |
| Current Quadlet manual keys           | [manual inventory](../fixtures/specification-drift/quadlet-manual-current.toml)   |
| Podman capability ranges and evidence | [capability catalogue](../catalogue/v1/podman-supported-range.toml)               |
| Exact generator releases and images   | [generator matrix](../tools/generator-matrix.toml)                                |
| Generator expectations                | [generator fixtures](../fixtures/generators/) and [tests](../tests/generators.rs) |
| External project sources and licenses | [corpus catalogue](../fixtures/real-world/corpus.toml)                            |
| Released changes                      | [changelog](../CHANGELOG.md)                                                      |
| Public Rust items                     | [source documentation](../src/lib.rs)                                             |

Do not copy complete key lists, fixture assertions, version matrices, or release histories into
prose. Link to the source instead.

## Writing rules

- Organize a page around a reader task, not implementation chronology.
- Lead with the contract, then show the shortest useful example or procedure.
- Keep paragraphs short and use headings that work as search results.
- State whether evidence covers syntax, generator output, or runtime behavior.
- Put durable architectural choices in ADRs and current work in the roadmap.
- Record exact compatibility evidence in the catalogue or fixture that validates it.

Coding agents must also follow the repository-root [`AGENTS.md`](../AGENTS.md).
