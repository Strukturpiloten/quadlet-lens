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

## Bounded native value views

`container_ports()` and `pod_ports()` decode ordered `PublishPort=` directives, including
bracketed IPv6 addresses, omitted host ports, ranges, protocols, resets, and the primary
physical source-value segment span. Continued values retain their complete authored syntax in
the document; native views intentionally point to the first segment rather than inventing one
span across physical lines.
`container_mounts()` and `pod_mounts()` preserve ordered `Volume=` and long `Mount=` forms,
including unknown mount options and relative or Quadlet reference spellings. `container_commands()`
uses bounded lexical word tokenization for `Exec=`, plus JSON string-array or literal-executable
decoding for `Entrypoint=`. `NativeCommand::syntax()` retains the authored Entrypoint form. It
retains lexical command prefixes and separators as arguments; it does not claim
systemd executable, prefix, or semicolon execution semantics.

Each view keeps valid native evidence separate from malformed values and values that need systemd
specifier expansion. It returns recoverable diagnostics and does not select a Podman release,
expand a specifier, inspect a path, or decide whether a value can be represented portably. `%%` is
the one context-independent systemd escape decoded to a literal percent; every other `%` specifier
is retained as a deferred value until a manager context is available.
`build_environment()` applies the same redacted authored-environment decoder to `[Build]`
`Environment=` values; `container_environment()` retains its existing container-only behavior.
These source-only views take no Podman target. Capability evaluation remains a separate caller
action; native decoding makes no portability or target-support decision.

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
