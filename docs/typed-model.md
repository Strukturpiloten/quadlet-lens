# Native typed model

The `model` module is the non-destructive bridge between QuadletLens's physical syntax and future
document-set, validation, rendering, and BoxFerry adapter layers. [ADR 0005](decisions/0005-source-aware-native-typed-model.md)
defines its representation boundary.

## Supported first-conversion units

`QuadletUnitType` currently accepts only these lowercase suffixes:

| Suffix       | Required section | Typed native boundary                                                                                                                                                                                                                                                                                                                               |
| ------------ | ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `.container` | `[Container]`    | Container keys listed below                                                                                                                                                                                                                                                                                                                         |
| `.pod`       | `[Pod]`          | Pod keys listed below                                                                                                                                                                                                                                                                                                                               |
| `.network`   | `[Network]`      | `NetworkName`, `Driver`, `Options`, `Label`, `Internal`, `IPv6`, `IPAMDriver`, `Subnet`, `Gateway`, `IPRange`, `ContainersConfModule`, `DisableDNS`, `DNS`, `GlobalArgs`, `InterfaceName`, `NetworkDeleteOnStop`, `PodmanArgs`, `ServiceName`                                                                                                       |
| `.volume`    | `[Volume]`       | `VolumeName`, `Driver`, `Options`, `Label`, `Device`, `Type`, `Copy`, `ContainersConfModule`, `GlobalArgs`, `PodmanArgs`, `User`, `Group`, `UID`, `GID`, `ServiceName`, `Image`                                                                                                                                                                     |
| `.build`     | `[Build]`        | repeatable `ImageTag`/`Network`/`Label`/`File`/`BuildArg`/`Secret`/`GroupAdd`/`DNS`/`DNSOption`/`DNSSearch`/`Annotation`/`Environment`/`ContainersConfModule`/`GlobalArgs`/`Volume`/`PodmanArgs`, singleton `SetWorkingDirectory`/`Target`/`Arch`/`Variant`/`Pull`/`Retry`/`RetryDelay`/`TLSVerify`/`ForceRM`/`AuthFile`/`IgnoreFile`/`ServiceName` |
| `.image`     | `[Image]`        | singleton `Image`, `ImageTag`, `ServiceName`, `AllTags`, `Arch`, `AuthFile`, `CertDir`, `Creds`, `DecryptionKey`, `OS`, `Policy`, `Retry`, `RetryDelay`, `TLSVerify`, `Variant`; repeatable `ContainersConfModule`, `GlobalArgs`, `PodmanArgs`                                                                                                      |
| `.kube`      | `[Kube]`         | required repeatable `Yaml`; repeatable `AutoUpdate`, `ConfigMap`, `ContainersConfModule`, `GlobalArgs`, `LogOpt`, `Network`, `PodmanArgs`, `PublishPort`, `RemapGid`, `RemapUid`; singleton `ExitCodePropagation`, `KubeDownForce`, `LogDriver`, `RemapUidSize`, `RemapUsers`, `ServiceName`, `SetWorkingDirectory`, `UserNS`                       |
| `.artifact`  | `[Artifact]`     | required singleton `Artifact`; repeatable `ContainersConfModule`, `GlobalArgs`, `PodmanArgs`; singleton `AuthFile`, `CertDir`, `Creds`, `DecryptionKey`, `Quiet`, `Retry`, `RetryDelay`, `ServiceName`, `TLSVerify`; shared `[Quadlet] DefaultDependencies`                                                                                         |

The typed boundary currently contains 90 container keys, twenty-five pod keys, eighteen network keys, sixteen
volume keys, twenty-eight build keys, eighteen image keys, nineteen Kube keys, thirteen experimental Artifact keys,
one shared Quadlet key, and nine reviewed systemd Unit relationship keys. Newly promoted Network, Image, Kube, Artifact, and Quadlet values are opaque;
their fixtures assert target command text, reset/order behavior, and documented version boundaries only. The exact key lists live in the
[specification coverage ledger](roadmap.md#specification-coverage-ledger).

`[Unit]`, `[Service]`, and `[Install]` are recognized as generic systemd sections. Parsed keys are
not restricted by a closed enum. `SystemdUnitKey` types `Requires`, `Wants`, `After`, `Requisite`,
`BindsTo`, `PartOf`, `Upholds`, `Conflicts`, and `Before`; every other systemd directive remains
generic and source-preserved. Other native sections and keys remain explicit `Unknown` entries.
Unsupported suffixes fail closed rather than implying complete typed support.

`QuadletDocument::container_environment()` is a separate authored semantic view for container
`Environment=` directives. It does not change `AuthoredValue` or source preservation. The view
keeps directive order, recognizes literal assignments, empty resets, and bare names after bounded
systemd word/quote/escape processing, and reports `%` values as deferred. Malformed names, quotes,
and escapes remain recoverable diagnostics. No environment-file, secret, manager, process, or
runtime expansion is attempted. Recognized Container and Build `Environment=` values are treated
as sensitive by repository-owned `TypedEntry` and document debug output, while explicit raw access
and preservation rendering remain available to callers that handle secrets safely.

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
- `UnitReference` identifies exact lowercase `.image`, `.build`, `.pod`, `.network`, `.volume`, and `.artifact`
  references where the recognized key permits them;
- `Opaque` retains everything else without decoding it.

The primary value preserves its authored continuation backslash. Continuation value segments remain
separate and ordered. Comments inside a continuation stay in the syntax document and never become
command arguments.

Exactly one of `Image` or `Rootfs` supplies a container workload. Both are singleton keys and they
conflict with each other. `Rootfs` is conservatively classified as a path, but QuadletLens does not
parse Podman's overlay-rootfs grammar or check the host directory and SELinux label.

`ContainersConfModule`, `GlobalArgs`, and `ImageVolume` are repeatable opaque Container keys. Their
physical values, empty reset lines, duplicate spellings, quoted text, continuations, and order stay
source-visible; the programmatic builder preserves their input order. `HealthLogDestination`,
`HealthMaxLogCount`, `HealthMaxLogSize`, `HealthStartupCmd`, `HealthStartupInterval`,
`HealthStartupRetries`, `HealthStartupSuccess`, `HealthStartupTimeout`, and `ServiceName` are opaque
Container singletons. QuadletLens does not parse health commands, numbers, sizes, durations, image
volume grammar, service identity, or health execution semantics. Duplicate singleton authored lines
receive `QLM0004`; generated construction rejects a second value. `ImageVolume` has no positive
native support range until versioned generator evidence is recorded.

Pod `ContainersConfModule`, `DNS`, `DNSOption`, `DNSSearch`, `GIDMap`, `GlobalArgs`, `Label`,
`NetworkAlias`, `PodmanArgs`, and `UIDMap` retain exact ordered physical values, including empty
reset lines and duplicates. `HostName`, `IP`, `IP6`, `SubGIDMap`, and `SubUIDMap` are opaque
singletons. Parsed duplicate singleton lines receive `QLM0004`; generated construction rejects a
second value. Pod `UserNS` combined with any direct or subordinate ID map is rejected with
`QLM0013`; direct and corresponding subordinate UID/GID maps are rejected with `QLM0014` and
`QLM0015`. These diagnostics use effective reset-aware values and source spans. The model neither
parses nor resolves DNS, labels, network aliases, addresses, ID maps, modules, or raw arguments.

Kube `Yaml` is required and repeatable. Missing `Yaml=` reports `QLM0017`; a blank `Yaml=`
resets the effective source list. `QLM0018` reports only when that reset-aware list has no source,
at the final effective-reset value span. Multiple effective `Yaml=` values combined with
`SetWorkingDirectory=yaml` report `QLM0019` because one YAML-relative directory cannot be selected;
an empty reset clears the effective list. `AutoUpdate`, `ConfigMap`, `ContainersConfModule`,
`GlobalArgs`, `LogOpt`, `Network`, `PodmanArgs`, and `PublishPort` retain ordered physical values,
including blank reset entries and duplicates. The remaining Kube keys are opaque singletons and use `QLM0004` when repeated. `Yaml`
and `ConfigMap` receive only lexical path classification; neither is read, normalized, or parsed.
An exact Kube `Network=NAME.network` establishes a document-set reference; all other network text
remains opaque. QuadletLens does not parse Kubernetes YAML, load ConfigMaps, execute `kube play` or
`kube down`, or make runtime claims.

Experimental Artifact `Artifact` is required. A missing Artifact section entry reports `QLM0021`;
the final authored `Artifact=` value being blank reports `QLM0022`. Earlier duplicate source text
remains preserved with the normal `QLM0004` singleton warning. `ContainersConfModule`, `GlobalArgs`,
and `PodmanArgs` remain ordered raw physical entries, including resets and duplicates; every other
Artifact value is an opaque singleton. `Creds` and `DecryptionKey` retain exact text for rendering
and explicit caller access but are redacted from repository-owned debug output. Artifact units are
capability-supported only from Podman 5.7.0 through 6.1.0. Container, Pod, and Build `Volume=`
source prefixes ending in `.artifact`, plus Container `Mount=type=artifact,source=` or `src=` exact
suffixes, resolve in document sets without parsing mount grammar; other suffixes remain opaque.
Shared `[Quadlet] DefaultDependencies` is an opaque singleton available for every typed unit type:
QuadletLens does not parse it as a boolean or infer systemd dependencies.

`ImageTag`, `Network`, `Label`, `File`, `BuildArg`, `Secret`, `GroupAdd`, `DNS`, `DNSOption`, `DNSSearch`, `Annotation`, `Environment`, `ContainersConfModule`, `GlobalArgs`, and `PodmanArgs` are typed repeatable Build keys: every authored physical value
stays ordered and opaque. `ImageTag` retains the first tag that Podman uses as a referenced build
artifact name; Build `DNSSearch` does not apply reset or special-dot semantics. Build `Annotation` preserves raw physical lines without tokenization, unquoting, C-unescaping, reset, duplicate-key selection, sorting, OCI validation, or image-metadata inference. Build `Environment` preserves raw physical lines without tokenization, unquoting, C-unescaping, reset, duplicate-name selection, sorting, or host lookup. Build `ContainersConfModule` preserves raw physical lines without path parsing, module reads, configuration inspection, reset, deduplication, tokenization, or normalization. Build `GlobalArgs` preserves raw physical lines without tokenization, reset, unquoting, C-unescaping, option validation, or inferred semantic, security, or runtime effects. Build `AuthFile` is an opaque singleton: the model preserves physical source lines and duplicate diagnostics without path validation or reads, credential parsing, content or sensitivity classification, or generator-effective-last normalization. Build `IgnoreFile` is likewise an opaque singleton without path resolution or reads, ignore-rule parsing, `.containerignore`/`.dockerignore` default inference, relative-path normalization, or generator-effective-last behavior. Build `ServiceName` is opaque singleton text without .service stripping, basename/default derivation, template handling, identity mutation, or generator-effective-last normalization. `Network` recognizes only an exact lowercase `.network` value as a Network reference
for document-set resolution, while network modes, options, and every other spelling remain opaque.
It does not adopt observed `.container` reference semantics. `File` does not classify paths, URLs,
or Containerfile forms, and does not select Podman's observed effective-last value. `Label` retains
each physical line without parsing `KEY=VALUE`, unquoting, choosing duplicate names, collapsing or
sorting a map, or validating label text. `BuildArg` does not parse `KEY=VALUE` text, expand or
resolve environments, read secrets, or infer bare/null meaning. `Secret` does not split comma-separated
text, parse argument names, resolve environment forms or paths, or materialize secret data. `PodmanArgs`
does not split or quote arguments, resolve contexts, paths, environments, images, or services, validate a CLI,
or infer build/runtime/cross-format behavior.
`GroupAdd` does not look up groups, parse supplementary-group grammar, interpret keep-groups
exclusivity, or resolve rootless or user-namespace behavior.
`DNS` does not resolve names or addresses, parse server grammar, interpret `none`, inspect
`resolv.conf` or host DNS, or define build/runtime behavior.
`SetWorkingDirectory`, `Target`, `Arch`, `Variant`, `Pull`, `Retry`, `RetryDelay`, `TLSVerify`, and
`ForceRM` are typed singletons whose exact text remains opaque;
QuadletLens does not resolve paths, URLs, build contexts, or generated systemd `WorkingDirectory`
precedence. A container
Duplicate authored singleton Build lines remain source-aware and receive the standard singleton diagnostic;
programmatic construction rejects a second value. QuadletLens does not validate build-stage names, platform grammar,
or architecture defaults. `Pull` does not validate policy spelling, select a default, normalize text,
or expose an effective-last value; it also makes no Compose boolean, image-pull, registry, or runtime claim.
`ForceRM` does not parse boolean text, select defaults, expose effective-last behavior, or make
cleanup, failure, execution, configuration, cache-equivalence, runtime, or conversion claims.
QuadletLens does not inspect a Containerfile or run a build. A container
`Image=name.build` is classified as a Build reference and resolves by exact basename in a document
set. Likewise, `Network=name.network` in a Build unit resolves only by exact basename in a document
set.

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
semantics are equivalent.

`ReloadCmd` and `ReloadSignal` are opaque, non-sensitive singleton physical-line values. The parser
retains every authored value, quote, specifier, continuation, blank, malformed line, and duplicate;
the builder rejects duplicate keys and either mutually exclusive pair. Parsed documents containing
both keys retain both entries but report `QLM0010`. QuadletLens neither tokenizes a reload command
nor parses a signal, chooses an effective value, derives a name or cidfile, exposes generated
`ExecReload` ordering, inspects a container, or triggers a reload.

`ExitPolicy` is an opaque, non-sensitive pod singleton. The parser retains every authored value,
quote, specifier, continuation, blank, malformed line, and duplicate; duplicates report `QLM0004`.
The builder rejects a duplicate. QuadletLens does not parse `continue` or `stop`, select an
effective or default value, derive names, or claim restart, runtime, CLI, or cross-format behavior.

Pod `StopTimeout` is an opaque, non-sensitive singleton. The parser retains every authored value,
quote, specifier, continuation, blank, malformed line, and duplicate; duplicates report `QLM0004`.
The builder rejects a duplicate. QuadletLens does not parse seconds or `-1`, inject a default,
select an effective value, calculate time, or claim systemd, restart, runtime, CLI, or cross-format
behavior.

Pod `ServiceName` is an opaque, non-sensitive singleton. The parser retains every authored value,
quote, specifier, continuation, blank, malformed line, and duplicate; duplicates report `QLM0004`.
The builder accepts exactly one physical-line-safe value and rejects a duplicate. QuadletLens does
not strip or require `.service`, derive defaults, normalize effective values, append suffixes,
evaluate templates or specifiers, or alter document/dependency identity, systemd, restart, runtime,
or cross-format behavior.

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

Container `IP` and `IP6` are opaque singletons. Container `NetworkAlias` is opaque, repeatable,
and resettable. The model preserves every physical value, singleton duplicates and diagnostics,
alias duplicates and order, empty resets, quotes, specifiers, and continuations without parsing or
validating addresses, aliases, IPAM, DNS, network configuration, runtime behavior, or cross-format
equivalence. These spellings remain unknown in other native scopes.

Network `Driver` is an opaque singleton and `Options` is opaque, repeatable, and resettable.
Omission, duplicate driver diagnostics, every physical option, resets, duplicate option keys,
quotes, specifiers, and continuations remain preserved. QuadletLens does not parse option text,
validate a driver or provider-specific options, or reproduce the generator's effective reset,
duplicate collapse, sorting, or version-specific bare-token behavior.

Network `Label` is opaque, repeatable, and resettable. Empty assignments, duplicates, bare
values, embedded equals signs, quotes, specifiers, continuations, and authored order remain
physical source data. QuadletLens does not tokenize labels, collapse duplicate names, sort them,
validate OCI label conventions, or reproduce version-specific bare-token behavior.

Volume `Label` is also opaque, repeatable, and resettable. Empty assignments, duplicates, bare
values, embedded equals signs, quotes, specifiers, continuations, and authored order remain
physical source data. QuadletLens does not tokenize labels, collapse duplicate names, sort them,
validate OCI label conventions, or reproduce version-specific bare-token behavior.

Volume `ContainersConfModule` is opaque and repeatable. Every physical value, including empty
reset assignments, duplicates, quotes, specifiers, and continuations, remains ordered source data.
QuadletLens does not parse paths, read modules or configuration, apply reset behavior, tokenize,
normalize, infer sensitivity, or define volume-creation, filesystem, lifecycle, runtime, Compose,
or conversion semantics.

Volume `GlobalArgs` is opaque and repeatable. Every physical value, including empty reset
assignments, duplicates, quotes, whitespace, specifiers, C-escapes, and continuations, remains
ordered source data. QuadletLens does not tokenize, unquote, C-unescape, omit malformed text,
apply resets, validate arguments, infer sensitivity, read modules, or define volume creation,
lifecycle, filesystem, runtime, Compose, or conversion semantics.

Volume `PodmanArgs` is opaque and repeatable. Every physical value, including empty reset
assignments, duplicates, quotes, whitespace, specifiers, C-escapes, and continuations, remains
ordered source data. QuadletLens does not tokenize, unquote, C-unescape, omit malformed text,
apply resets, deduplicate, validate options, infer sensitivity, or define CLI, volume creation,
lifecycle, filesystem, systemd, runtime, Compose, or conversion semantics.

Volume `User` is an opaque singleton. Every authored physical value, including numeric-looking,
name-like, blank, whitespace, quoted, specifier, continuation, and malformed-looking text remains
source-aware with the ordinary duplicate diagnostic; the model performs no UID/name parsing, host
lookup, last-value selection, defaulting, ownership, mount, filesystem, runtime, Compose, or conversion behavior.

Volume `GID` is an opaque singleton. Every authored physical value, including numeric-looking,
name-like, blank, whitespace, quoted, specifier, continuation, and malformed-looking text remains
source-aware with the ordinary duplicate diagnostic; the model does not parse or otherwise
interpret its value.

Volume `ServiceName` is an opaque singleton. Every authored physical value remains source-aware
with the ordinary duplicate diagnostic; the model does not tokenize, derive, or otherwise interpret it.

Volume `Image` is an opaque singleton except that an exact lowercase `.image` or `.build` basename
is classified as a native reference. Both resolve by exact basename when their corresponding typed
documents are present.

Image-unit `Image` is an opaque required singleton source, while `ImageTag`, `ServiceName`, `AllTags`, `Arch`, `AuthFile`, and `CertDir` are
opaque singletons. `ContainersConfModule` is repeatable physical-line text without module/configuration reads, reset, tokenization, CLI validation, or pull semantics. `AuthFile` has no path validation or reads, credential parsing, sensitivity classification, or registry-authentication semantics. `CertDir` has no path or certificate validation or reads, containers-certs.d default, remote-client policy, sensitivity, or registry-authentication semantics.
Their physical entries, duplicate diagnostics, quotes, continuations, and specifiers remain
source-aware; neither becomes an image-unit reference. Missing and blank/whitespace source values
remain errors only for `Image`. QuadletLens does not select service/resource names or defaults,
parse image transports, registries, tags, digests, archives, authentication, TLS, boolean/platform/default, pull, or substitution behavior.

Network `Internal` and `IPv6` are opaque singletons. Omission, literal true/false, duplicate
diagnostics, invalid or vendor-defined spellings, quotes, specifiers, continuations, and every
physical entry remain distinct. QuadletLens does not parse booleans, choose a last value, or import
Podman's invalid-as-false behavior. `Internal` remains driver-conditional; `IPv6` represents
dual-stack behavior only, so the model does not invent an IPv4-enable key.

Network `IPAMDriver` is an opaque singleton. `Subnet`, `Gateway`, and `IPRange` are opaque,
repeatable, resettable physical entries: empty assignments, duplicates, quotes, specifiers,
continuations, and authored inter-key order remain intact. The model does not apply resets, infer
subnets, parse addresses or ranges, or zip the three columns into effective target rows.

Container `DNS`, `DNSOption`, `DNSSearch`, `ExposeHostPort`, `Annotation`, `Mask`,
and `Unmask` are typed repeatable keys with opaque one-line values. Omission, resets, duplicates,
order, quoting, specifiers, malformed text, and exact spelling remain source-aware.

Container `AppArmor`, `NoNewPrivileges`, `SeccompProfile`, and the five
`SecurityLabel*` keys are typed opaque singletons. Duplicate authored lines produce `QLM0004`;
the builder rejects a second singleton.

QuadletLens does not parse addresses, ports, OCI assignments, booleans, profiles, SELinux values,
or paths for these keys. Unsupported scopes remain unknown and preserved, and generator evidence
does not imply host or runtime behavior.

## Document sets and dependency graph

Volume `Copy` is an opaque singleton. Every physical authored value, including blanks, duplicates,
matched or unmatched quotes, specifiers, and continuations, remains source-aware; duplicate lines
produce `QLM0004`. QuadletLens does not parse a boolean, select an effective duplicate, add an
`Image` model field, or infer copy-up, volume creation, image pulls, runtime, rootless, plugin, or
cross-format behavior.

`NamedQuadletDocument` pairs a typed document with a validated basename whose suffix must match the
selected unit type. `QuadletDocumentSet` requires unique `SourceId` values, retains duplicate
basenames for diagnostics, and resolves every native reference by exact basename. It never searches
the host filesystem or expands paths.

The graph retains all references. A reference is `Resolved`, `Missing`, or `Ambiguous`; only an
exactly resolved reference becomes a `DependencyEdge`. This distinction lets BoxFerry report an
incomplete application without silently discarding the authored relationship. Duplicate source
identities are construction errors because source-labelled diagnostics would otherwise be unsafe.

Reviewed `[Unit]` relationship values are decoded as systemd-style whitespace lists for graphing
only. Repetition and source order are retained, an empty assignment resets earlier values of the
same key, quotes and continuations group tokens, and malformed quoting creates no speculative edge.
Only exact `.container`, `.pod`, `.network`, `.volume`, `.build`, `.image`, `.kube`, and
`.artifact` basenames enter the native graph; ordinary `.service` and `.target` names stay opaque.
Each reference and edge exposes its originating `SystemdUnitKey`.

## Programmatic generation

The `render` module can construct the supported native document types with typed native keys,
typed dependency-ordering `[Unit]` directives, and open-ended generic systemd directives. It renders deterministic source and reparses it through the
same syntax and typed-model pipeline before returning a result. Values remain exact native
one-line text rather than being normalized by an incomplete systemd or Podman value parser. See
[programmatic generation](generation.md) and [ADR 0009](decisions/0009-validated-programmatic-generation.md).

## Diagnostics

Initial stable codes are:

| Code      | Severity | Meaning                                                                                                                                                                                                                                    |
| --------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `QLM0001` | error    | Required native section is missing                                                                                                                                                                                                         |
| `QLM0002` | error    | A container has neither `Image=` nor `Rootfs=`                                                                                                                                                                                             |
| `QLM0003` | warning  | A native section does not match the selected unit suffix                                                                                                                                                                                   |
| `QLM0004` | warning  | A first-conversion singleton key is repeated                                                                                                                                                                                               |
| `QLM0005` | error    | An `Image=` entry is empty                                                                                                                                                                                                                 |
| `QLM0006` | error    | `Image=` and `Rootfs=` are both present                                                                                                                                                                                                    |
| `QLM0007` | error    | A `Rootfs=` entry is empty                                                                                                                                                                                                                 |
| `QLM0008` | error    | An Image unit is missing `Image=`                                                                                                                                                                                                          |
| `QLM0009` | error    | An Image-unit `Image=` entry is blank                                                                                                                                                                                                      |
| `QLM0010` | error    | `ReloadCmd=` and `ReloadSignal=` are both present                                                                                                                                                                                          |
| `QLM0017` | error    | A Kube unit is missing required `Yaml=`                                                                                                                                                                                                    |
| `QLM0018` | error    | A Kube unit has no effective `Yaml=` source after reset processing                                                                                                                                                                         |
| `QLM0019` | error    | Multiple effective `Yaml=` sources use `SetWorkingDirectory=yaml`                                                                                                                                                                          |
| `QLM0020` | error    | Effective `UserNS=` conflicts with effective `RemapUid=`, `RemapGid=`, or `RemapUsers=`; both values are source-labelled. `RemapUidSize=` alone does not conflict.                                                                         |
| `QLM0023` | warning  | A container `Environment=` directive has incomplete continuation, malformed systemd quoting/escaping, or an unsupported variable name; the source remains preserved and the authored semantic view records recoverable unmodeled evidence. |
| `QLM0024` | warning  | A container `Environment=` assignment contains `%` specifier syntax; the authored semantic view classifies it as deferred and never expands it.                                                                                            |
| `QLG0001` | error    | A native unit reference has no matching document                                                                                                                                                                                           |
| `QLG0002` | error    | A native unit reference matches duplicate basenames                                                                                                                                                                                        |
| `QLG0003` | error    | The document set contains a duplicate basename                                                                                                                                                                                             |

Diagnostics are recoverable and source-labelled. A warning does not make the combined result
invalid; an error does.

## Deliberately deferred

- later Quadlet unit types and remaining Image keys
- parsing and canonical rendering of individual systemd/Podman value grammars
- dependency-cycle analysis and systemd runtime activation semantics
- target-version validation that combines documents with the capability catalogue
- mutation and preservation-oriented editing APIs
- key-specific typed value constructors and target-aware rendering
