# Architecture

QuadletLens separates physical syntax, native Quadlet meaning, cross-file relationships, target
support, and output. None of these layers performs runtime deployment.

## Data flow

```text
source text ──▶ syntax document ──▶ typed document ──▶ document set
                    │                    │                  │
                    ├── preserved output ├── canonical output
                    │                    │                  │
generated values ──▶ document builder ───┘                  │
                                                           ▼
target profile ──▶ capability catalogue ───────────▶ compatibility result
```

Each arrow is an explicit API boundary. Parsing one file does not discover neighboring files, and
target evaluation does not inspect the development machine.

## Layers

### Source and syntax

The syntax layer owns physical lines, comments, blank lines, section order, repeated entries,
continuations, line endings, invalid recoverable input, and byte spans. It deliberately does not
decode Podman or systemd value grammar.

Preservation rendering reproduces untouched syntax. Canonical rendering normalizes structural
presentation while retaining authored values, order, repetition, comments, and specifiers.

### Native model

The model recognizes the supported `.container`, `.pod`, `.network`, `.volume`, `.image`,
`.build`, `.kube`, and `.artifact` units. Generic `[Unit]`, `[Service]`, and `[Install]`
directives remain open-ended because systemd owns their vocabulary. Unknown Quadlet entries remain
attached instead of being discarded.

Most values are intentionally opaque. A typed key establishes its section, cardinality, source
location, and conservative value kind; it does not imply complete semantic validation.

### Document sets

A document set associates typed documents with validated unit-file basenames. It resolves native
references by exact name, retains missing and ambiguous relationships, and produces deterministic
dependency edges without reading a directory.

### Capability catalogue

The catalogue describes evidenced support over explicit Podman ranges and optional systemd
requirements. It is independent of parsing: recognizing a key does not prove that every target
supports it.

### Generation

The builder accepts typed native keys and open-ended generic systemd directives, enforces structural
rules, emits deterministic text, and reparses the result. Callers remain responsible for choosing
an evidenced native value representation.

## Module ownership

| Concern                                      | Module or data                                 |
| -------------------------------------------- | ---------------------------------------------- |
| Source identifiers and spans                 | `source`                                       |
| Ordered physical syntax and syntax rendering | `syntax`                                       |
| Native documents and typed keys              | `model`                                        |
| Exact cross-file relationships               | `model::document_set`                          |
| Conservative path classification             | `path`                                         |
| Programmatic construction                    | `render`                                       |
| Structured diagnostics                       | `diagnostic`                                   |
| Target ranges and evidence                   | `capability` and `catalogue/`                  |
| Exact Podman generator execution             | test support and `tools/generator-matrix.toml` |

## Dependency rules

- Syntax does not depend on typed Quadlet models.
- Typed models may carry source references but never parser internals.
- Capability data does not depend on model enums or BoxFerry.
- Exact generator execution is test tooling, not library behavior.
- Cross-format policy belongs to BoxFerry.
- QuadletLens never starts, installs, enables, or applies generated output.

Representation decisions are recorded in the [ADR index](decisions/README.md). Current supported
surface and evidence come from the [machine-readable sources](README.md#sources-of-truth).
