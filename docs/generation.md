# Programmatic generation

`QuadletDocumentBuilder` constructs native Quadlet documents without requiring callers to assemble
section names or complete files as strings.

## Build a document

```rust
use quadlet_lens::{
    model::{ContainerKey, EntryValue, QuadletUnitType},
    render::QuadletDocumentBuilder,
    source::SourceId,
};

let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
builder.push_container(
    ContainerKey::Image,
    EntryValue::new("example.invalid/web:1")?,
)?;
builder.push_container(
    ContainerKey::Environment,
    EntryValue::new("APP_ENV=production")?,
)?;

let generated = builder.build(SourceId::new(1))?;
assert!(generated.text().contains("Image=example.invalid/web:1"));
# Ok::<(), Box<dyn std::error::Error>>(())
```

Successful output includes the generated text and its complete parsed result.

## Builder contract

The builder:

- places typed keys only in their owning native section;
- keeps repeatable entries in insertion order;
- rejects duplicate singleton keys;
- emits sections in deterministic order;
- accepts open-ended generic systemd directives;
- rejects NUL bytes and physical line endings in one-line values; and
- reparses generated text and rejects syntax or native-model errors.

It does not quote, split, expand, normalize, or validate every Podman and systemd value grammar.
`EntryValue` represents exact physical-line-safe text chosen by the caller.

## Focused values

Dedicated helpers provide stronger construction only where the project has an evidenced boundary.
Examples include process limits, shared-memory sizes, container memory, literal environment
assignments, assignment groups, and explicit environment resets.

These helpers are additive. Raw parsed values and raw `EntryValue` construction remain available,
and a helper does not imply runtime enforcement or cross-format equivalence.

## Environment plans

`ContainerEnvironmentPlan` preserves assignment, group, and reset directive order. It can answer
explicit per-name literal lookup where later assignments win and resets clear earlier values.

The plan renders its original directives. It does not expose a host-derived environment map, load
files or secrets, expand specifiers, or evaluate runtime state. Debug output redacts values.

For newly generated literal assignments, `ContainerEnvironmentPlan::sorted_by_name()` returns an
opt-in stable key order. It preserves reset boundaries and duplicate same-name order while expanding
groups into individual directives. Default generation and every parsed rendering keep authored
insertion order. See [environment and secret values](environment-and-secrets.md).

## Multiple files

Build each file independently with a distinct `SourceId`. Pair successful documents with validated
basenames and create a document set to verify exact references and dependency edges.

Generation does not select a Podman target. When output must support a version range, evaluate each
used representation through the [capability model](capability-model.md). BoxFerry owns
cross-format selection and loss policy.

The complete key and method surface belongs to the
[Rust API](https://boxferry.dev/docs/api/quadlet-lens/); exact compatibility claims belong to the
[catalogue](../catalogue/v1/podman-supported-range.toml).
