# Native typed model

The `model` module is the non-destructive bridge between QuadletLens's physical syntax and future
document-set, validation, rendering, and BoxFerry adapter layers. [ADR 0005](decisions/0005-source-aware-native-typed-model.md)
defines its representation boundary.

## Supported first-conversion units

`QuadletUnitType` currently accepts only these lowercase suffixes:

| Suffix       | Required section | Typed native boundary       |
| ------------ | ---------------- | --------------------------- |
| `.container` | `[Container]`    | Container keys listed below |
| `.pod`       | `[Pod]`          | Pod keys listed below       |
| `.network`   | `[Network]`      | `NetworkName`               |
| `.volume`    | `[Volume]`       | `VolumeName`                |

Typed container keys are `AddHost`, `Image`, `Exec`, `Environment`, `EnvironmentFile`,
`User`, `Group`, `UserNS`, `GroupAdd`, `WorkingDir`, `ReadOnly`, `PublishPort`, `Volume`,
`Network`, `Pod`, `HealthCmd`, `HealthInterval`, `HealthRetries`, `HealthStartPeriod`,
`HealthTimeout`, `Notify`, and `PodmanArgs`. Typed pod keys are `AddHost`, `PodName`, `PublishPort`,
`Network`, and `Volume`.

`[Unit]`, `[Service]`, and `[Install]` are recognized as generic systemd sections. Parsed keys are
not restricted by a closed enum. Programmatic generation additionally offers typed `Requires`,
`Wants`, and `After` `[Unit]` directives for the evidence-backed dependency subset. Other sections
and keys remain explicit `Unknown` entries.
Unsupported suffixes, including `.image` and `.build`, currently fail closed rather than implying
complete typed support.

## Parse result

`QuadletDocument::parse` returns a `QuadletParseResult` containing:

- the complete loss-aware syntax result, including preserved source and syntax diagnostics;
- the ordered typed document;
- separate native-model diagnostics.

`QuadletParseResult::is_valid` requires both diagnostic layers to contain no errors. Callers that
need to report details should retain both layers rather than treating typed interpretation as a
replacement for syntax parsing.

Repeated sections and entries remain repeated. Each typed section name, key, primary value, and
continuation segment has owned authored text plus a `SourceSpan`. The zero-based physical line index
is also available for stable ordering. Comments and invalid physical lines remain in the syntax
document; the typed view does not duplicate them.

## Value boundary

`ValueKind` makes only conservative lexical claims:

- `Path` distinguishes absolute, unit-relative, other relative, and systemd-specifier spellings;
- `UnitReference` identifies exact lowercase `.image`, `.build`, `.pod`, `.network`, and `.volume`
  references where the recognized key permits them;
- `Opaque` retains everything else without decoding it.

The primary value preserves its authored continuation backslash. Continuation value segments remain
separate and ordered. Comments inside a continuation stay in the syntax document and never become
command arguments.

The model does not expand `%h`, `~`, environment variables, or relative paths. It does not yet parse
systemd quoting, environment assignment lists, port ranges, mount options, health commands, or raw
Podman arguments. Those forms remain usable as authored text and must not be normalized implicitly.
Identity keys likewise retain exact values: no user or group lookup is performed, and namespace
and working-directory values are not validated against a host or container image.

## Document sets and dependency graph

`NamedQuadletDocument` pairs a typed document with a validated basename whose suffix must match the
selected unit type. `QuadletDocumentSet` requires unique `SourceId` values, retains duplicate
basenames for diagnostics, and resolves every native reference by exact basename. It never searches
the host filesystem or expands paths.

The graph retains all references. A reference is `Resolved`, `Missing`, or `Ambiguous`; only an
exactly resolved reference becomes a `DependencyEdge`. This distinction lets BoxFerry report an
incomplete application without silently discarding the authored relationship. Duplicate source
identities are construction errors because source-labelled diagnostics would otherwise be unsafe.

## Programmatic generation

The `render` module can construct the supported native document types with typed native keys,
typed dependency-ordering `[Unit]` directives, and open-ended generic systemd directives. It renders deterministic source and reparses it through the
same syntax and typed-model pipeline before returning a result. Values remain exact native
one-line text rather than being normalized by an incomplete systemd or Podman value parser. See
[programmatic generation](generation.md) and [ADR 0009](decisions/0009-validated-programmatic-generation.md).

## Diagnostics

Initial stable codes are:

| Code      | Severity | Meaning                                                  |
| --------- | -------- | -------------------------------------------------------- |
| `QLM0001` | error    | Required native section is missing                       |
| `QLM0002` | error    | A container native section has no `Image=` entry         |
| `QLM0003` | warning  | A native section does not match the selected unit suffix |
| `QLM0004` | warning  | A first-conversion singleton key is repeated             |
| `QLM0005` | error    | An `Image=` entry is empty                               |
| `QLG0001` | error    | A native unit reference has no matching document         |
| `QLG0002` | error    | A native unit reference matches duplicate basenames      |
| `QLG0003` | error    | The document set contains a duplicate basename           |

Diagnostics are recoverable and source-labelled. A warning does not make the combined result
invalid; an error does.

## Deliberately deferred

- typed `.image`, `.build`, and later Quadlet unit types
- parsing and canonical rendering of individual systemd/Podman value grammars
- dependency-cycle analysis and systemd runtime activation semantics
- target-version validation that combines documents with the capability catalogue
- mutation and preservation-oriented editing APIs
- key-specific typed value constructors and target-aware rendering
