# Architecture decision records

## Status values

- `proposed` — under discussion
- `accepted` — current direction
- `superseded` — replaced by another ADR
- `rejected` — considered but not adopted

## Index

| ADR                                                           | Status     | Decision                                                                               |
| ------------------------------------------------------------- | ---------- | -------------------------------------------------------------------------------------- |
| [0001](0001-project-boundaries-and-origin.md)                 | accepted   | Independent Quadlet library, data-driven capabilities, and from-scratch implementation |
| [0002](0002-loss-aware-systemd-syntax.md)                     | accepted   | Loss-aware ordered physical-line systemd-style syntax kernel                           |
| [0003](0003-conservative-canonical-syntax-rendering.md)       | accepted   | Deterministic structural rendering without value normalization                         |
| [0004](0004-versioned-capability-catalogue.md)                | accepted   | Strict data-driven capability schema, finite coverage, and evidence levels             |
| [0005](0005-source-aware-native-typed-model.md)               | accepted   | Source-aware native model with conservative value classification                       |
| [0006](0006-rolling-support-window-and-generator-evidence.md) | accepted   | Podman 5.4 minimum, rolling current target, and exact ranged generator evidence        |
| [0007](0007-exact-name-document-set-resolution.md)            | accepted   | Exact-name document sets with explicit missing and ambiguous reference states          |
| [0008](0008-versioned-public-api-and-release-contract.md)     | superseded | Initial 0.1.x library API and trusted-publishing release contract                      |
| [0009](0009-validated-programmatic-generation.md)             | accepted   | Typed native keys, exact values, deterministic output, and parse-back validation       |
| [0010](0010-consolidated-0.2-public-api.md)                   | accepted   | Consolidated 0.2.x model API without compatibility-only re-exports                     |

Use the next four-digit number for new decisions. Include context, decision, consequences, and alternatives. Supersede accepted decisions with a new ADR rather than rewriting history.
