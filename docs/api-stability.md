# API stability policy

QuadletLens is pre-1.0, but version 0.2 establishes the current supported integration line for
BoxFerry and independent tools that need native Quadlet parsing, modeling, document-set resolution,
and evidence-backed Podman capability queries. This policy is recorded by
[ADR 0010](decisions/0010-consolidated-0.2-public-api.md).

## The 0.2.x contract

Within the 0.2.x line:

- patch releases preserve source compatibility for the supported public entry points;
- public APIs use QuadletLens-owned types;
- the module paths exercised by `tests/public_api.rs` remain available;
- diagnostic code strings remain machine-readable contracts;
- preservation rendering stays byte-identical and canonical syntax rendering remains
  deterministic for the same valid input;
- parsing, modeling, document-set resolution, capability evaluation, and rendering perform no
  filesystem discovery, process execution, unit installation, or runtime mutation; and
- all supported public APIs compile on Rust 1.85.0 or newer.

Bug fixes may change a result that contradicted these contracts or retained conformance evidence.
Such a change needs a regression test and changelog entry. A patch release must not silently drop
authored syntax, flatten repeated entries, expand systemd specifiers, or broaden capability
evidence beyond reviewed ranges.

## Supported entry points

| Concern                                  | Public modules         |
| ---------------------------------------- | ---------------------- |
| Source and diagnostics                   | `source`, `diagnostic` |
| Loss-aware syntax and rendering          | `syntax`               |
| Native typed documents and document sets | `model`, `path`        |
| Validated programmatic generation        | `render`               |
| Versioned Podman capability evidence     | `capability`           |

The compile-and-behavior contract in `tests/public_api.rs` exercises these stages as an external
crate consumer would. The modules remain separate: QuadletLens does not hide source loading,
target selection, filesystem lookup, Podman execution, or systemd operations behind a convenience
API.

## Changes before 1.0

An intentional public break requires the next 0.x minor version, migration guidance in the release
notes, and an ADR when the architecture changes. Consumers that cannot absorb that cadence should
use an exact dependency requirement or commit their lockfile.

Enums intended to grow are marked `#[non_exhaustive]` before the first release. Existing public
enums without that marker require a breaking release when adding a variant would break exhaustive
matches.

Public unit-only enums also expose their implicit numeric discriminants through Rust casts. New
variants are therefore appended; existing variants are never reordered. The public API regression
test records the published key-enum values, and normal CI compares the candidate API with the
latest published crate before changes can reach the release workflow.

## Not promised by 0.2

The 0.2 contract does not claim:

- typed coverage of every open-ended generic systemd directive or complete systemd value grammar;
- key-specific quoting, value parsing, and target-aware value rendering;
- runtime, rootless/rootful, SELinux, cgroup, network, or systemd activation behavior;
- support evidence beyond the finite catalogue range or its capability-specific evidence; or
- long-term 1.x compatibility.

Before 1.0, the project will define supported release lifetimes, deprecation periods, and the 1.x
diagnostic-code policy through a superseding ADR.
