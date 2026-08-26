# Typed model

The typed model adds native Quadlet structure without replacing the loss-aware syntax document.
Use it when a caller needs typed keys, diagnostics, or multi-file relationships and must still be
able to explain or preserve the source.

## Parse one document

```rust
use quadlet_lens::{
    model::{QuadletDocument, QuadletUnitType},
    source::SourceId,
};

let result = QuadletDocument::parse(
    QuadletUnitType::Container,
    SourceId::new(7),
    "[Container]\nImage=example.invalid/api:1\n",
)
.expect("recognized unit type");

assert!(result.is_valid());
```

The result contains:

- the typed `QuadletDocument`;
- the complete syntax parse result;
- syntax diagnostics; and
- native-model diagnostics.

`is_valid()` is false when either diagnostic layer contains an error. Recovery can still return
source evidence that a caller may display or preserve.

## Unit and section boundary

Quadlet unit type comes from the caller, normally from a validated filename suffix. Supported
native sections expose typed key enums. Generic systemd sections remain open, and unknown native
sections or keys stay explicit.

Repeated sections and entries stay repeated. Typed entries retain their authored key, value
segments, source span, and physical order. Comments and invalid lines remain in the syntax document
rather than being duplicated in the model.

## Value kinds

The model makes only conservative lexical claims:

| Kind            | Meaning                                                                |
| --------------- | ---------------------------------------------------------------------- |
| `Path`          | Absolute, unit-relative, other relative, or systemd-specifier spelling |
| `UnitReference` | An exact supported Quadlet filename reference                          |
| `Opaque`        | Authored text with no stronger claim                                   |

The model does not expand `%h`, `~`, environment variables, or relative paths. It does not inspect
images, users, groups, devices, secrets, networks, filesystems, or the host.

Some keys have focused value helpers or cross-field diagnostics. Those additions do not narrow raw
authored input: unusual values remain preserved unless the physical syntax itself is invalid.

## Environment view

`QuadletDocument::container_environment()` provides a bounded semantic view over container
`Environment=` directives. It preserves directive order and recognizes literal assignments, bare
names, and resets after systemd-compatible word processing.

Deferred specifiers and malformed names or quoting produce recoverable diagnostics. The view never
loads environment files or secrets and never performs manager, process, or runtime expansion.
Repository-owned debug output redacts recognized environment values; explicit source access remains
the caller's responsibility.

`QuadletDocument::container_environment_sources()` adds source-located `EnvironmentFile=` and
environment-exposing `Secret=` references without acquiring their values. Resolution accepts only
decoded values explicitly authorized by the caller. See
[environment and secret values](environment-and-secrets.md) for trust and BoxFerry boundaries.

## Document sets

Use `NamedQuadletDocument` to pair a document with a validated basename, then build a
`QuadletDocumentSet`. The set:

1. rejects duplicate source identities and duplicate basenames;
2. classifies exact native references;
3. records resolved, missing, and ambiguous relationships; and
4. derives deterministic dependency edges.

Resolution is in memory. The caller chooses which documents belong to the set and retains the
mapping from `SourceId` to filename and source text.

## Diagnostics and privacy

Branch on diagnostic codes and typed severities, not display text. Keep the source text available
when rendering labels, but avoid raw excerpts when a key may contain credentials or environment
values.

For exact item signatures and supported key enums, use the
[Rust API](https://boxferry.dev/docs/api/quadlet-lens/). For target support, evaluate the
[capability catalogue](capability-model.md) separately.
