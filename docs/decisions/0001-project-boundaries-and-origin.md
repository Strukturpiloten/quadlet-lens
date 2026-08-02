# ADR 0001: Project boundaries and from-scratch origin

- Status: accepted
- Date: 2026-07-31

## Context

BoxFerry needs native Quadlet parsing and rendering that can also explain compatibility across Podman and relevant systemd versions. This concern is independently useful to validators, editors, generators, and other Podman tooling.

Embedding Quadlet in BoxFerry would couple native format evolution to one conversion application. Forking an existing converter would also inherit unrelated Compose, CLI, and conversion architecture.

## Decision

QuadletLens is an independent repository and public Rust library. It owns native Quadlet documents and an evidence-backed, data-driven capability catalogue. It has no dependency on BoxFerry.

QuadletLens is implemented from scratch. It is not a fork of Podlet, and source code will not be copied or mechanically translated from Podlet, Podman, systemd, or another implementation.

Public documentation, released behavior, and identified source versions may be used for research and differential testing.

## Consequences

- Quadlet parsing and target-version policy have a reusable home from the beginning.
- BoxFerry contains mappings rather than native parser implementation.
- The project must build comprehensive syntax and real-generator test suites.
- Capability claims require maintained evidence as Podman evolves.
- Initial development takes longer than extracting existing model structs.

## Alternatives considered

### Keep Quadlet implementation inside BoxFerry

Rejected because it would require later extraction and would discourage independent use.

### Extract or port Podlet's Quadlet implementation

Rejected in favor of a clear from-scratch rule and an architecture designed for loss-aware parsing and data-driven version support.
