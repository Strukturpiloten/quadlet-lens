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

Typed container keys are `AddHost`, `ContainerName`, `Image`, `Rootfs`, `Entrypoint`, `RunInit`, `StopSignal`, `StopTimeout`, `Pull`, `PidsLimit`, `HostName`, `ShmSize`, `DropCapability`, `AddCapability`, `Tmpfs`, `Sysctl`, `Ulimit`, `AddDevice`, `Memory`, `Exec`, `Environment`, `EnvironmentFile`, `Label`, `Secret`,
`User`, `Group`, `UserNS`, `GroupAdd`, `WorkingDir`, `ReadOnly`, `PublishPort`, `Volume`,
`Network`, `Pod`, `HealthCmd`, `HealthInterval`, `HealthRetries`, `HealthStartPeriod`,
`HealthTimeout`, `Notify`, and `PodmanArgs`. Typed pod keys are `AddHost`, `PodName`, `PublishPort`,
`Network`, `Volume`, `UserNS`, and `ShmSize`.

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

Exactly one of `Image` or `Rootfs` supplies a container workload. Both are singleton keys and they
conflict with each other. `Rootfs` is conservatively classified as a path, but QuadletLens does not
parse Podman's overlay-rootfs grammar or check the host directory and SELinux label.

The model does not expand `%h`, `~`, environment variables, or relative paths. It does not yet parse
systemd quoting, environment assignment lists, port ranges, mount options, health commands, or raw
Podman arguments. Those forms remain usable as authored text and must not be normalized implicitly.
Identity keys likewise retain exact values: no user or group lookup is performed, and namespace
and working-directory values are not validated against a host or container image. Label and secret
values remain exact Podman text; the model does not validate label naming conventions, read secret
contents, or verify runtime existence.

`ContainerName` is a typed singleton distinct from the Quadlet unit-file basename and generated
systemd service name. Its exact value is retained; QuadletLens does not invent a runtime name or
probe the host for collisions.

`Entrypoint` is a typed singleton distinct from `Exec`. Its exact executable or JSON command-array
text is retained; QuadletLens does not decode or normalize JSON and systemd quoting.

`RunInit` is a typed singleton whose exact authored one-line value remains text. Omission remains
the absence of a typed entry, while explicit `true` and `false` remain distinct values. The model
does not interpret raw noncanonical text as a boolean, and it does not select, mount, or inspect
Podman's container-init binary. The capability catalogue's `literal-true-or-false` form describes
the evidenced caller values rather than adding parser validation.

`StopSignal` and container `StopTimeout` are typed singletons whose exact one-line values remain
authored text, including `StopTimeout=0`. Named/numeric signals and non-negative integer seconds are
the currently evidenced supported caller forms, not parser or builder validation rules. Negative,
fractional, overflow-sized, or otherwise unusual authored values therefore remain recognized and
preserved without a claim that Podman accepts them. QuadletLens does not normalize signals, infer a
timeout from systemd, claim that zero sends a signal, or claim that another format's zero/default
semantics are equivalent. The separate pod `StopTimeout` key remains syntax-preserved but untyped.

`Pull` is a typed singleton whose exact one-line value remains authored text. Omission remains the
absence of a typed entry. The catalogue records `always`, `missing`, `never`, and `newer` as
evidenced caller forms; the model does not interpret or reject other raw values.

`PidsLimit` is a typed singleton whose exact one-line value remains authored text. Omission, `-1`,
positive integers, zero, overflow-sized text, and noncanonical values remain distinct and opaque;
the parser does not validate their semantics. The separate generation helper safely constructs
only documented `-1` unlimited or nonzero ASCII-decimal finite spellings. It preserves leading
zeros and arbitrary-precision digits without parsing into a machine integer; no portable target
maximum is claimed. Zero remains raw-preserved but is not capability-evidenced.

`HostName` is a typed singleton whose exact one-line value remains opaque authored text. Omission
remains omission, and QuadletLens does not apply Compose RFC-1123 validation or normalize native
values. Podman's documented behavior requires a private UTS namespace; an isolated container uses
the default private UTS namespace, while a container joining a pod with the default shared UTS
namespace uses the pod hostname instead. QuadletLens does not change UTS mode, model pod
`HostName`, or inspect the runtime hostname.

Container and pod `ShmSize` are separate typed singletons whose exact one-line values remain opaque
authored text. Omission, zero, unit-bearing values, arbitrary-precision amounts, and noncanonical
raw values stay distinct; parsed native values are not forced through a Compose grammar. The
focused `ShmSize` constructor accepts only non-negative ASCII-decimal amounts with optional
lowercase `b`, `k`, `m`, or `g`, preserving exact spelling and leading zeros without parsing.
`ShmSize::unlimited()` produces explicit `0`, and `is_unlimited` distinguishes a zero amount from
omission. Podman documents unitless bytes, a `64m` omission default, zero as unlimited IPC memory,
and a host-IPC conflict. Pod `ShmSize` applies in the pod's default shared-IPC context. QuadletLens
does not change IPC mode, enforce or inspect shared memory, apply a default, or infer a
cross-format mapping.

Container `DropCapability` is a typed repeatable key whose exact one-line values remain opaque and
ordered. Omission remains distinct from one or more entries, and a space-separated authored value
remains one `EntryValue`: QuadletLens does not split it, deduplicate capability names, lowercase the
source text, or validate native capability tokens. The catalogue and generator evidence describe
the supported native forms and observed generated command, not runtime privilege state.

Container `AddCapability` is the corresponding typed repeatable addition key. Omission, an empty
native reset assignment, duplicates, entry order, case, and space-separated text remain exactly
authored through parsing and rendering. QuadletLens does not apply the generator's observed list
splitting or lowercasing and does not interpret `all`. Tagged Podman source and exact generator
output record that special behavior separately; they do not establish the runtime effective or
bounding capability sets.

Container `Tmpfs` is a typed repeatable key whose exact one-line values remain opaque. Omission,
empty native reset assignments, duplicates, order, case, destination spelling, and option spelling
remain distinct. QuadletLens does not split `CONTAINER-DIR[:OPTIONS]`, normalize paths or option
case, deduplicate destinations, validate Linux mount options, or reinterpret the entry as
`Volume`. Tagged Podman source and generator output describe `LookupAll` post-reset command
construction separately; they do not create or inspect a temporary filesystem.
`Tmpfs` is container-only; `PodKey` deliberately has no corresponding variant, so authored pod
`Tmpfs=` remains an unknown preserved entry rather than being assigned container semantics.

Container `Sysctl` is also a typed repeatable key with exact opaque one-line values. Omission,
empty resets, duplicates, insertion order, case, whitespace, systemd quoting/specifiers, and raw
text remain distinct. QuadletLens does not parse `name=value`, split space-separated lists,
normalize settings, validate namespaces, or infer kernel/runtime acceptance. Tagged Podman source
and generator output describe `LookupAllStrv` tokenization, reset, and command construction
separately; they do not prove runtime effects. `PodKey` deliberately has no `Sysctl` variant, so
authored pod `Sysctl=` remains an unknown preserved entry.

Container `Ulimit` is a typed repeatable key with the same opaque one-line boundary. Omission,
empty resets, duplicates, insertion order, case, systemd quotes/specifiers, and exact authored text
remain distinct. QuadletLens does not split or unquote values and does not validate
`TYPE=SOFT[:HARD]`. Tagged source and generator output describe `LookupAll` reset and command
construction separately; they do not establish runtime resource-limit behavior. `PodKey`
deliberately has no `Ulimit` variant, so authored pod `Ulimit=` remains an unknown preserved entry.

Container `AddDevice` is a typed repeatable key with the same opaque one-line boundary. Omission,
empty resets, duplicates, insertion order, case, systemd quotes/specifiers, whitespace-containing
lines, a leading `-`, and exact authored text remain distinct. QuadletLens does not split or
unquote values, parse host/container paths or permissions, check devices, or implement conditional
inclusion. Tagged source and generator output describe `LookupAllStrv` tokenization, reset, and
leading-minus handling separately; they do not establish runtime device access. `PodKey`
deliberately has no `AddDevice` variant, so authored pod `AddDevice=` remains unknown and preserved.

Container `Memory` is a typed singleton whose exact one-line value remains opaque. Omission,
duplicates, empty assignments, quoting, specifiers, zero, and vendor-defined spellings remain
available to diagnostics and preservation rendering without runtime interpretation. Duplicate
singletons produce the ordinary model diagnostic, while programmatic construction rejects the
second assignment. `Memory::new` is an additive safe path for positive ASCII-decimal amounts with
no suffix or one lowercase `b`, `k`, `m`, or `g`; it preserves leading zeros and arbitrary
precision without parsing. Pod `Memory` remains an unknown preserved entry.

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
| `QLM0002` | error    | A container has neither `Image=` nor `Rootfs=`           |
| `QLM0003` | warning  | A native section does not match the selected unit suffix |
| `QLM0004` | warning  | A first-conversion singleton key is repeated             |
| `QLM0005` | error    | An `Image=` entry is empty                               |
| `QLM0006` | error    | `Image=` and `Rootfs=` are both present                  |
| `QLM0007` | error    | A `Rootfs=` entry is empty                               |
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
