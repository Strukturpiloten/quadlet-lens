# Target project structure

QuadletLens begins as one public crate with a versioned data catalogue and clear module boundaries.

```text
quadlet-lens/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── AGENTS.md
├── README.md
├── LICENSE
├── src/
│   ├── lib.rs
│   ├── source/             # source identifiers, spans, and diagnostics
│   ├── syntax/             # ordered systemd-style unit syntax
│   ├── model/              # typed native Quadlet documents and values
│   ├── document_set/       # references and dependency graph
│   ├── capability/         # catalogue schema, loading, and range evaluation
│   ├── validation/         # native and target-aware validation
│   ├── render/             # preservation and canonical rendering
│   └── diagnostic/         # stable structured diagnostics
├── capabilities/
│   ├── schema.json
│   ├── podman/
│   │   ├── 5.4.toml
│   │   ├── 5.5.toml
│   │   └── ...
│   └── systemd/
├── tests/
│   ├── syntax/
│   ├── roundtrip/
│   ├── capabilities/
│   ├── generators/
│   └── real-world/
├── fixtures/
│   └── README.md
├── tools/
│   └── catalogue/          # extraction/diff helpers; never sole source of truth
├── docs/
└── .github/
    ├── workflows/
    └── ISSUE_TEMPLATE/
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
