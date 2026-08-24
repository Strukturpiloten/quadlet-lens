# QuadletLens

QuadletLens is a Rust library for reading, inspecting, generating, and rendering Podman Quadlet
documents. It preserves source details while keeping Podman and systemd compatibility decisions
explicit.

## Install

```console
cargo add quadlet-lens
```

QuadletLens requires Rust 1.85.0 or newer.

## Parse a document

```rust
use quadlet_lens::{
    model::{QuadletDocument, QuadletUnitType},
    source::SourceId,
};

let source = "[Container]\nImage=example.invalid/web:1\n";
let parsed = QuadletDocument::parse(
    QuadletUnitType::Container,
    SourceId::new(1),
    source,
)
.expect("valid Quadlet input");

assert!(parsed.is_valid());
```

The parse result retains typed entries, the original syntax document, source spans, and separate
syntax and model diagnostics.

## What the library guarantees

- Ordered sections, repeated keys, comments, continuations, unknown fields, and systemd specifiers
  remain available.
- Typed documents cover the supported Quadlet unit types without flattening their source.
- Document sets resolve native cross-file references without filesystem discovery.
- Preservation and canonical rendering are deterministic.
- Programmatic generation validates and reparses its output before returning it.
- Compatibility queries use a versioned, evidence-backed catalogue.

QuadletLens never installs units, reloads systemd, invokes Podman, inspects the host, or converts
another container format. Cross-format conversion belongs to
[BoxFerry](https://github.com/Strukturpiloten/boxferry).

## Documentation

- [User guide](https://boxferry.dev/docs/libraries/quadlet-lens/)
- [Rust API](https://boxferry.dev/docs/api/quadlet-lens/)
- [Maintainer guide](docs/README.md)
- [Roadmap](docs/roadmap.md)
- [Changelog](CHANGELOG.md)

Repository instructions for coding agents are in [AGENTS.md](AGENTS.md).

## Project

QuadletLens is an independent, from-scratch implementation. It is maintained by
[Martin “Becks” Beckert](https://github.com/TheRealBecks) through
[Strukturpiloten OHG](https://www.strukturpiloten.de/) and licensed under the
[Mozilla Public License 2.0](LICENSE).
