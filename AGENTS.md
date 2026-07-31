# Repository guidance for coding agents

This file applies to the entire QuadletLens repository.

## Read before changing code

Read these documents in order:

1. `README.md`
2. `docs/architecture.md`
3. `docs/project-structure.md`
4. `docs/capability-model.md`
5. `docs/testing.md`
6. `docs/dependency-policy.md`
7. `docs/decisions/README.md` and all accepted ADRs

Architectural changes require documentation and an ADR update in the same change.

## Scope

QuadletLens owns native Quadlet syntax, native typed models, document-set relationships, rendering, diagnostics, and capability evidence for supported Podman/systemd targets.

QuadletLens does not own Compose, Kubernetes, runtime inspection, BoxFerry's application model, or cross-format conversion decisions. It must not depend on BoxFerry.

## Origin policy

QuadletLens is implemented from scratch. Do not copy or mechanically translate source code from Podlet, Podman, systemd, or another Quadlet parser. Public documentation, released source behavior, and commands from identified versions may inform independent implementation and differential tests.

External fixtures and behavior-oracle results must record source, version, command, environment, license, and expected result.

## Non-negotiable behavior

- Unknown keys and generic systemd sections are never silently discarded.
- Preserve repeated keys and their ordering; a map is not a sufficient syntax representation.
- Keep systemd specifiers such as `%h` distinct from shell or Compose substitutions.
- Keep source syntax, typed values, and target-version validation as separate layers.
- A capability claim requires evidence and a test or an explicitly documented test gap.
- Distinguish a recognized key from a key that behaves correctly in a particular patch release.
- Invalid user input returns structured diagnostics and never panics.
- Rendering is deterministic for the same document, target, and options.
- Parsing and rendering never install, enable, start, or otherwise mutate units.

## Development rules

- Do not hard-code a closed Podman version enum in application logic; use the data-driven capability catalogue.
- Do not attempt to model all Podman features. Model Quadlet features and fallbacks relevant to the library contract.
- Keep target range evaluation independent of the parser.
- Add parser, renderer, capability-boundary, and real-generator tests with behavior changes.
- Record Podman tag/commit, documentation source, and observed generator behavior for capability updates.
- Update documentation and compatibility claims in the same change.
- Pin every GitHub Action to its full commit SHA and append its exact release tag as a comment. Verify new pins upstream; Renovate must preserve and update both values.

## Canonical development commands

The crate uses Rust 2024, supports Rust 1.85.0 and newer, and pins the normal development toolchain in `rust-toolchain.toml`.

```shell
cargo fmt --all -- --check
cargo ci-check
cargo ci-clippy
cargo ci-test
cargo ci-doctest
RUSTDOCFLAGS="-D warnings" cargo ci-doc
cargo +1.85.0 ci-check
cargo deny check
```

The `ci-*` aliases in `.cargo/config.toml` use locked resolution and all workspace features and targets where applicable. Catalogue validation and real-generator matrix commands must be added here when their harnesses are introduced.
