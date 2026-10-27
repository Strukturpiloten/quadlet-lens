# Programmatic generation

`QuadletDocumentBuilder` constructs native Quadlet documents, validates their structure, and
reparses the generated text before returning the complete syntax/model result.

## Build a document

```rust
use quadlet_lens::{
    model::{ContainerKey, QuadletUnitType},
    render::{EntryValue, EnvironmentAssignment, QuadletDocumentBuilder},
    source::SourceId,
};

let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
builder.push_container(ContainerKey::Image, EntryValue::new("example.invalid/web:1")?)?;
builder.push_container_environment(EnvironmentAssignment::new("APP_ENV", "production")?)?;
let generated = builder.build(SourceId::new(1))?;
assert!(generated.text().contains("Image=example.invalid/web:1"));
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Builder contract

The builder enforces native section ownership and singleton cardinality, preserves repeatable-entry
order, emits deterministic sections, and accepts generic systemd directives. Physical-line values
reject NUL and line endings. Protected generated boundaries reject preceding continuations.

`EntryValue` is exact caller-selected spelling; it does not validate every native value grammar.
Generation never expands expressions, acquires runtime values, or applies output.

## Focused values

Dedicated helpers strengthen construction for evidenced representations such as resource limits,
environment assignments, groups, and resets. Raw source and `EntryValue` remain available; helper
acceptance does not imply runtime enforcement, target compatibility, or cross-format equivalence.

## Environment plans

`ContainerEnvironmentPlan` preserves assignments, groups, and resets in order. Explicit per-name
lookup applies later assignments and resets without loading files, secrets, or a host environment.
Debug redacts values. The plan renders its original directives rather than a host-derived map.

`sorted_by_name()` optionally orders newly generated literals while preserving reset boundaries
and duplicate-name order. Default generation and parsed rendering preserve insertion order. See
[environment and secret values](environment-and-secrets.md).

## Multiple files

Build files with distinct `SourceId` values, then pair their documents with validated basenames
and use a document set to check exact references. Evaluate the [capability catalogue](capability-model.md)
separately for target ranges. BoxFerry owns cross-format selection and loss policy.

## Protected inert extension sections

Use `XSectionName`, `ExtensionValue`, and `push_extension_entry` for opaque native vendor metadata.
Section names use ASCII `X-` followed by an alphanumeric byte and then alphanumeric/dot/underscore/
hyphen bytes; keys are nonempty ASCII alphanumeric names. Values reject NUL, CR, LF, and final
backslashes, including before trailing whitespace. Errors never retain attempted input.

Extension sections follow standard sections in lexical name order, retaining repeated-entry order.
Protected storage avoids printable `EntryValue` conversions. JSON/vendor semantics remain outside
QuadletLens. See [ADR 0021](decisions/0021-protected-inert-extension-sections.md) and the
[inert-section fixture](../fixtures/generators/x-sections-inert/fixture.toml).

## Defer explicitly selected service references

Use `DeferredWord` with ordered `DeferredWordPart::{Literal, Reference}` parts. Empty words produce
empty assignment values. Service references/providers use strict ASCII `ServiceEnvironmentName`.
Literal container keys use protected `ContainerEnvironmentName`: nonempty, without equals,
controls/NUL, or leading whitespace. Dotted and literal dollar/percent keys are supported for
construction; broader reverse names remain `Unmodeled` with diagnostics.

`push_container_environment_deferred` escapes literal key/value dollars and percents and quotes
the assignment word. Only reference parts emit canonical `${NAME}` syntax. `Volume=` instead uses
raw field text: its quotes/backslashes are literal bytes, and the generator owns argv quoting.
No shell, defaults, commands, recursive expansion, or provider evaluation is introduced.

`push_container_bind_source_deferred` accepts an absolute literal target and native option tokens.
The source must be pure absolute literals or one direct leading reference followed only by literals.
It rejects source/target colons, option delimiter injection, and authored `.volume`/`.artifact`
source suffixes that generators could rewrite before expansion; `.image` remains supported.

Pure absolute literal sources reject dollars, whitespace, quotes, and backslashes because older
inferred mount dependencies cannot preserve them. Reference-leading suffixes support those literal
bytes without inferred source dependencies. Targets reject trailing whitespace/backslashes to
avoid raw-line loss. Runtime providers must yield nonempty absolute paths without colons, NUL,
or controls; QuadletLens never acquires or validates those runtime values.

Use `ServiceEnvironmentAssignment` with `push_service_environment_literal` for protected literal
service providers: dollars stay literal and percents are escaped. `ServiceEnvironmentFile` with
`push_service_environment_file` declares ordered absolute literal paths without globs, mandatory
by default; `.with_optional(true)` explicitly permits missing files. Files override inline service
assignments, with later files overriding earlier files. Container/service scopes and empty values
remain distinct. No provider file is read, created, or resolved.

## Source evidence and privacy

Environment/mount views decode paired dollars/percents once; unescaped braced references stay
nonexact `Deferred` evidence. Short Volume words starting with an unescaped single dollar are also
deferred; embedded unbraced dollars stay literal in assignments and long Mount syntax. Raw Volume
bytes are not word-unquoted/C-decoded; long Mount retains its separate word-then-CSV boundary.

Re-export decoded dollar-bearing literals through `DeferredWordPart::Literal`. Historical literal
construction APIs keep their spelling behavior; service inline/caller-decoded provider values are
never exec-decoded. No reverse provider policy is reconstructed.

Debug protects helpers, generated storage, syntax, typed values, and derived views. Physical
`Environment`, `EnvironmentFile`, `Volume`, `Mount`, or X-header candidates conservatively protect
source values even in malformed/consumed syntax. Explicit access and rendering retain bytes.
Detailed redaction rules and native limitations live in
[ADR 0022](decisions/0022-typed-deferred-service-environment.md).

The [deferred fixture](../fixtures/generators/deferred-service-environment/fixture.toml) records
passing exact Podman 5.4.0/6.1.2 endpoints and runs in every selected active generator lane.
Endpoint acceptance proves native text/argument preservation, not full-matrix observations,
provider availability, manager expansion, or runtime behavior. Native tests never execute units.

See the [Rust API](https://boxferry.dev/docs/api/quadlet-lens/) for item signatures and the
[machine catalogue](../catalogue/v1/podman-supported-range.toml) for exact capability evidence.
