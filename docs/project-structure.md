# Project structure

QuadletLens is one library crate with clear module boundaries. The crate foundation exists; entries marked `planned` are created with their first behavior and tests. The versioned data catalogue begins with the capability-schema milestone.

```text
quadlet-lens/
├── Cargo.toml
├── Cargo.lock
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
│   ├── source/             # planned: source identifiers, spans, and diagnostics
│   ├── syntax/             # planned: ordered systemd-style unit syntax
│   ├── model/              # planned: typed native Quadlet documents and values
│   ├── document_set/       # planned: references and dependency graph
│   ├── capability/         # planned: catalogue loading and range evaluation
│   ├── validation/         # planned: native and target-aware validation
│   ├── render/             # planned: preservation and canonical rendering
│   └── diagnostic/         # planned: stable structured diagnostics
├── capabilities/           # planned with the capability-schema milestone
│   ├── schema.json
│   ├── podman/
│   │   ├── 5.4.toml
│   │   ├── 5.5.toml
│   │   └── ...
│   └── systemd/
├── tests/
│   ├── README.md           # suite ownership and introduction rules
│   ├── repository_policy.rs # fixture and workflow-pin enforcement
│   └── support/            # private repository-test helpers
├── fixtures/
│   └── README.md           # fixture location and safety rules
├── tools/                  # planned when catalogue automation is justified
│   └── catalogue/          # extraction/diff helpers; never sole source of truth
├── docs/
│   └── fixture-format.md   # versioned fixture manifest contract
└── .github/
    ├── renovate.json
    └── workflows/
        └── ci.yml
```

## Module placement rules

| Concern                                  | Owner                                  |
| ---------------------------------------- | -------------------------------------- |
| Entry order, comments, and continuations | `syntax`                               |
| Quadlet-native fields and value types    | `model`                                |
| Cross-file references                    | `document_set`                         |
| Version and fallback evidence            | `capability` plus `capabilities/` data |
| Target-aware correctness                 | `validation`                           |
| File output                              | `render`                               |
| External generator execution             | test/tooling support                   |

Do not encode a version matrix in model enums, use unordered maps as the source syntax tree, or put BoxFerry conversion rules into the library.
