# QuadletLens

QuadletLens is the Quadlet document library behind BoxFerry. Use it directly when a Rust program needs to inspect, validate, generate, or render Quadlet while keeping Podman and systemd compatibility explicit.

## What the library owns

- loss-aware parsing and source locations for Quadlet files;
- typed keys for every supported Quadlet unit type and shared systemd sections;
- multi-file relationship graphs;
- deterministic and preservation-oriented rendering;
- a versioned capability catalogue backed by reviewed evidence; and
- typed generation that validates its output before returning it.

QuadletLens does not install units, reload systemd, run Podman, inspect the host, or convert another container format. Cross-format conversion belongs to [BoxFerry](https://boxferry.dev/docs/).

## Choose a topic

- [Model](model/) explains documents, unit types, and document sets.
- [Parsing and rendering](parsing-rendering/) covers preserved, canonical, and generated output.
- [Diagnostics](diagnostics/) covers codes, source spans, and partial results.
- [Compatibility](compatibility/) explains targets and the downloadable capability catalogue.
- [Rust API](https://boxferry.dev/docs/api/quadlet-lens/) lists every public item.

Add the library with `quadlet-lens = "0.2"`. Rust 1.85.0 or newer is required.
