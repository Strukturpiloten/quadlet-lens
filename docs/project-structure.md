# Project structure

QuadletLens is one library crate with clear module boundaries. The crate foundation exists; entries marked `planned` are created with their first behavior and tests. The versioned data catalogue begins with the capability-schema milestone.

```text
quadlet-lens/
├── .devcontainer/         # digest-pinned VS Code environment and feature lock
├── Cargo.toml
├── Cargo.lock
├── CHANGELOG.md
├── rust-toolchain.toml
├── rustfmt.toml
├── clippy.toml
├── deny.toml
├── .cargo/
│   └── config.toml        # canonical Cargo aliases
├── AGENTS.md
├── README.md
├── LICENSE
├── src/
│   ├── lib.rs
│   ├── source.rs           # source identifiers, spans, and line/column lookup
│   ├── syntax.rs           # ordered syntax plus preservation/canonical rendering
│   ├── path.rs             # lexical literal, relative, and systemd-specifier paths
│   ├── capability.rs       # strict catalogue loading and target-range evaluation
│   ├── model/
│   │   ├── mod.rs          # source-aware native documents, keys, paths, and references
│   │   └── document_set.rs # named documents, exact references, and dependency graph
│   ├── validation/         # planned: native and target-aware validation
│   ├── render/             # planned: typed target-aware rendering
│   └── diagnostic.rs       # stable structured syntax diagnostics
├── catalogue/
│   └── v1/
│       └── podman-supported-range.toml # finite evidence inside the rolling support window
├── tests/
│   ├── README.md           # suite ownership and introduction rules
│   ├── capabilities.rs     # schema and version-boundary behavior
│   ├── generators.rs       # matrix contract plus ignored container execution harness
│   ├── model.rs            # native typed subset, preservation, and diagnostics
│   ├── document_sets.rs    # exact cross-file resolution and dependency edges
│   ├── public_api.rs       # supported external 0.1.x consumer path
│   ├── syntax.rs           # preservation, recovery, canonical, property corpus
│   ├── repository_policy.rs # fixture and workflow-pin enforcement
│   └── support/            # private repository-test helpers
├── fixtures/
│   ├── README.md           # fixture location and safety rules
│   └── typed-model/        # authored container/pod/network/volume and graph cases
├── tools/
│   └── generator-matrix.toml # supported target, pinned images, commits, and builder
├── docs/
│   ├── api-stability.md    # supported pre-1.0 consumer contract
│   ├── releases/           # version-matched public release notes
│   └── fixture-format.md   # versioned fixture manifest contract
└── .github/
    ├── renovate.json
    └── workflows/
        ├── ci.yml
        ├── generator-matrix.yml
        └── release.yml
```

## Module placement rules

| Concern                                  | Owner                                  |
| ---------------------------------------- | -------------------------------------- |
| Entry order, comments, and continuations | `syntax`                               |
| Quadlet-native fields and value types    | `model`                                |
| Cross-file references                    | `model::document_set`                  |
| Version and fallback evidence            | `capability` plus `capabilities/` data |
| Target-aware correctness                 | `validation`                           |
| File output                              | `render`                               |
| External generator execution             | test/tooling support                   |

Do not encode a version matrix in model enums, use unordered maps as the source syntax tree, or put BoxFerry conversion rules into the library.
