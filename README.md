# QuadletLens

QuadletLens is a Rust library for parsing, modeling, validating, and rendering Podman Quadlet files across supported Podman versions.

It combines a source-aware Quadlet document model with a data-driven capability catalogue so callers can answer not only “is this valid Quadlet?” but also “for which Podman and systemd environments is this valid?”

> [!IMPORTANT]
> QuadletLens is in its initial design phase. It does not yet provide a usable crate, capability catalogue, or stable API.

## Goals

- Parse all supported Quadlet unit types and their shared systemd sections.
- Preserve comments, ordering, repeated keys, continuations, specifiers, and unknown fields where practical.
- Expose a typed native Quadlet model without losing the source document.
- Render preservation-oriented and deterministic canonical output.
- Validate against explicit Podman version ranges and relevant systemd capabilities.
- Describe native support, deprecation, removal, known bugs, and available Podman-argument fallbacks.
- Model relationships between multiple Quadlet files.
- Attach evidence and tests to every compatibility claim.

## Non-goals

- Running or installing Quadlet files
- Reimplementing Podman or systemd
- Parsing arbitrary generated systemd service files as Quadlet
- Converting Compose or Kubernetes directly to Quadlet
- Defining BoxFerry's cross-format conversion policy

Cross-format conversion belongs to [BoxFerry](https://github.com/Strukturpiloten/boxferry). Compose handling belongs to [ComposeLens](https://github.com/Strukturpiloten/compose-lens).

## Planned processing levels

```text
source text
  → loss-aware unit document
  → typed Quadlet document set
  → dependency graph
  → target-version validation
  → rendered Quadlet files
```

## Documentation

- [Documentation index](docs/README.md)
- [Software architecture](docs/architecture.md)
- [Target project structure](docs/project-structure.md)
- [Capability model](docs/capability-model.md)
- [Testing strategy](docs/testing.md)
- [Roadmap](docs/roadmap.md)
- [Architecture decisions](docs/decisions/README.md)

Repository-specific guidance for coding agents is in [AGENTS.md](AGENTS.md).

## Origin

QuadletLens is implemented from scratch. It is not a fork of Podlet and does not copy or mechanically translate Podlet source code.

## License

QuadletLens is licensed under the [Mozilla Public License 2.0](LICENSE).
