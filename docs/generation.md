# Programmatic generation

`QuadletDocumentBuilder` constructs native Quadlet documents without requiring callers to assemble
section names or complete files as strings.

## Build a document

```rust
use quadlet_lens::{
    model::{ContainerKey, QuadletUnitType},
    render::{EntryValue, QuadletDocumentBuilder},
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

## Protected inert extension sections

Use `render::XSectionName`, `render::ExtensionValue`, and
`QuadletDocumentBuilder::push_extension_entry` for native vendor sections. A section name starts
with ASCII `X-`, followed by an ASCII alphanumeric byte and then only ASCII alphanumeric bytes,
dots, underscores, or hyphens. Keys use nonempty ASCII alphanumeric names.

Extension values remain opaque. They reject NUL, CR, LF, and a final backslash, including a
backslash before trailing whitespace, so they cannot introduce another directive or accidentally
continue into another physical line. Validation errors contain no attempted input. This dedicated
protected type never routes payloads through the publicly printable `EntryValue` type.

The builder emits extension sections after its standard sections in lexical section-name order,
with repeated keys and insertion order preserved within each section. It reparses the complete
output before success. When extension entries are present, build also rejects final backslashes
in native or systemd values, preventing an earlier continuation from consuming an X-section header.
JSON framing, vendor schemas, and cross-format meaning belong to callers.

All parsed values in source containing any physical `X-*` header candidate are conservatively
sensitive, including unknown keys and continued values. This display-only over-redaction protects
malformed headers and headers consumed by a preceding continuation, without changing semantic
parsing or explicit access. Ordinary source without X-header candidates retains its existing
Debug behavior. Debug output protects generated values, builders, typed values, parse results, syntax,
and their source objects. Source Debug omits the whole raw text when it contains an X-section
header, including malformed source. Explicit `as_str`, source/value access, and rendering still
return original bytes and require caller-controlled disclosure.

The authored [generator fixture](../fixtures/generators/x-sections-inert/fixture.toml) checks
pass-through, repetition, and absence of payloads in active generated directives. Passing native
dry-run runs establish generator behavior only; they do not establish systemd runtime behavior.
See [ADR 0021](decisions/0021-protected-inert-extension-sections.md).

Decoded native views preserve this display protection: complete port specifications, environment-secret references,
and document-set reference names retain their originating entry's sensitivity. Their ordinary
Debug behavior remains available for unprotected source. Mount option names/values and authored
environment directive names/lookup literals are always redacted because their public enum variants
cannot carry source sensitivity without changing the API. Explicit fields, accessors, and rendered
source retain their original contents. Command arguments, mount paths, and environment-file paths
already use protected Debug representations. Diagnostics contain no copied payload text.
