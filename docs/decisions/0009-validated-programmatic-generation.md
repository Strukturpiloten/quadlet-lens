# ADR 0009: validated programmatic generation

- Status: accepted
- Date: 2026-08-03
- Additive amendments: 2026-08-03, 2026-08-05, 2026-08-06, and 2026-08-07

## Context

BoxFerry and other library consumers need to create native Quadlet documents without assembling
section names and key spellings themselves. QuadletLens 0.1.0 could parse and render authored
documents, but it exposed no supported construction path. Generating raw Quadlet text in an
adapter would put native syntax policy outside the library that owns it.

The existing typed model intentionally keeps most systemd and Podman value grammars opaque. A
first construction API must not imply that quoting, command arguments, environment assignments,
ports, or mount options have already been fully modeled.

## Decision

QuadletLens provides a programmatic document builder that:

- selects one supported native unit type;
- accepts typed native keys and open-ended generic systemd directives;
- accepts typed `Requires`, `Wants`, and `After` `[Unit]` directives for the evidence-backed
  dependency subset while retaining open-ended generic systemd construction;
- retains repeated supplementary-group entries while keeping the other execution-identity keys
  singleton;
- retains repeated container label assignments in insertion order;
- retains repeated container secret entries for distinct mounted-file and environment exposures;
- retains pod `UserNS` as a singleton distinct from container-level namespace selection;
- retains container `HostName` as an opaque singleton without adding cross-format hostname
  validation or normalization;
- retains separate container and pod `ShmSize` values as opaque singletons without selecting IPC
  modes or importing cross-format size grammar;
- retains repeated container `DropCapability` entries as exact opaque one-line values in insertion
  order without splitting, deduplication, lowercasing, or native capability validation;
- retains repeated container `AddCapability` entries, including empty native reset assignments, as
  exact opaque one-line values in insertion order without splitting, deduplication, lowercasing,
  or native capability validation;
- retains repeated container `Tmpfs` entries, including empty native reset assignments, as exact
  opaque one-line values in insertion order without splitting destination/options text,
  normalization, deduplication, mount-option validation, or conflation with `Volume`;
- retains repeated container `Sysctl` entries, including empty native reset assignments, as exact
  opaque one-line values in insertion order without parsing `name=value`, splitting whitespace,
  normalization, namespace validation, or runtime/kernel interpretation;
- retains repeated container `Ulimit` entries, including empty native reset assignments, as exact
  opaque one-line values in insertion order without splitting, unquoting, parsing
  `TYPE=SOFT[:HARD]`, normalization, resource-name validation, or runtime interpretation;
- retains repeated container `AddDevice` entries, including empty native reset assignments, as
  exact opaque one-line values in insertion order without splitting, unquoting, parsing host and
  container paths or permissions, interpreting whitespace tokenization or a leading `-`, checking
  device existence, or applying runtime semantics;
- retains container `Memory` as an opaque singleton without applying runtime, cgroup, swap,
  page-size, host-memory, rootless, or cross-format semantics;
- retains container `LogDriver` as an opaque singleton and repeated `LogOpt` entries, including
  empty resets, as exact opaque one-line values without parsing options, validating logging
  drivers/options, injecting defaults, or applying runtime or cross-format semantics;
- retains container `IP` and `IP6` as opaque singletons and repeated `NetworkAlias` entries,
  including empty resets, without address, alias, IPAM, DNS, network, runtime, or cross-format
  interpretation;
- retains the promoted repeatable networking, annotation, Mask, and Unmask keys as opaque
  physical-line-safe values in insertion order, including duplicates and reset assignments;
- retains repeated volume label entries, including empty native reset assignments, as exact opaque
  physical-line-safe values in insertion order without OCI parsing, reset application, duplicate
  collapse, sorting, validation, or runtime interpretation;
- retains repeated Build `Secret` entries as exact opaque physical-line-safe values in insertion
  order without comma splitting, argument, environment, or path parsing, secret materialization,
  or build/runtime interpretation;
- retains repeated Build `GroupAdd` entries as exact opaque physical-line-safe values in insertion
  order without group lookup, supplementary-group parsing, keep-groups exclusivity, rootless or
  user-namespace interpretation, runtime, build-execution, Compose privilege-equivalence, or
  conversion interpretation;
- retains repeated Build `DNS` entries as exact opaque physical-line-safe values in insertion
  order without resolver behavior, none compatibility, resolv.conf or host-DNS inspection,
  build-execution, Compose endpoint-mapping, or conversion interpretation;
- retains repeated Build `DNSOption` and `DNSSearch` entries as exact opaque physical-line-safe
  values in insertion order without model reset or special-dot semantics, option or domain parsing,
  resolver behavior, build execution, Compose mapping, or conversion interpretation;
- retains Build `AuthFile` as an opaque singleton without path validation or reads, credential
  parsing, content or sensitivity classification, generator-effective-last normalization, registry
  authentication, build-success, runtime, Compose, or conversion interpretation;
- retains Build `IgnoreFile` as an opaque singleton without path resolution or reads, ignore-rule
  parsing, `.containerignore` or `.dockerignore` default inference, relative-path normalization,
  generator-effective-last normalization, build-success, runtime, Compose, or conversion
  interpretation;
- retains repeated Build `Annotation` entries as exact opaque physical-line values in insertion
  order without splitting, unquoting, C-unescaping, OCI validation, reset application,
  duplicate-key collapse, sorting, image-metadata inference, build-success, runtime, Compose, or
  conversion interpretation;
- retains repeated Build `Environment` entries as exact opaque physical-line values in insertion
  order without splitting, unquoting, C-unescaping, host lookup, reset application, duplicate-name
  selection, sorting, build-success, runtime, Compose, or conversion interpretation;
- retains repeated Build `ContainersConfModule` entries as exact opaque physical-line values in
  insertion order without path parsing, module reads, configuration inspection, reset application,
  deduplication, tokenization, normalization, build-success, runtime, Compose, or conversion
  interpretation;
- retains Build `Arch` and `Variant` as opaque singletons without parsing platform grammar,
  selecting defaults, or applying effective-last behavior;
- retains Build `Pull` as an opaque singleton without policy validation, default selection,
  spelling normalization, effective-last behavior, Compose boolean inference, or runtime semantics;
- retains volume `Device` and `Type` as separate opaque singletons without path, filesystem,
  quote, specifier, generator-dependency, mount, runtime, or cross-format interpretation;
- retains volume `GID` as an opaque singleton without parsing or otherwise interpreting its value;
- retains AppArmor, no-new-privileges, seccomp, and SELinux-label keys as opaque singletons;
- performs no key-specific address, port, OCI, boolean, profile, SELinux, path, filesystem, host,
  runtime, or cross-format interpretation for these additions;
- rejects native keys from the wrong unit type and repeated singleton keys;
- retains repeatable entries in insertion order;
- emits sections in deterministic `[Unit]`, native, `[Service]`, `[Install]` order;
- rejects NUL bytes and physical line endings in values;
- provides a focused process-ID-limit helper that constructs only `-1` unlimited or nonzero
  ASCII-decimal finite spelling, retaining arbitrary precision without parsing while leaving the
  existing raw `EntryValue` boundary unchanged;
- provides a focused shared-memory-size helper for a non-negative ASCII-decimal amount with an
  optional lowercase `b`, `k`, `m`, or `g`, preserving exact arbitrary-precision spelling and
  explicitly representing zero as Podman's documented unlimited value while leaving parsed and
  raw values opaque;
- provides a focused container-memory helper for a positive ASCII-decimal amount with an optional
  lowercase `b`, `k`, `m`, or `g`, preserving leading zeros and arbitrary precision without
  narrowing the raw `EntryValue` boundary; and
- reparses generated text and returns it only when syntax and native-model validation succeed.

Values remain exact, already-semantic native values. The builder does not quote, split, expand, or
normalize key-specific value grammars. Focused value encoders may be added later when their rules
are protected by documentation and generator evidence.

## Consequences

- Native section/key spelling and structural validation stay inside QuadletLens.
- BoxFerry can generate typed native documents without depending on parser internals or composing
  complete files as strings.
- Callers remain responsible for choosing a valid semantic representation for each value.
- Adding value-specific encoders is additive and does not require replacing this document-level
  boundary.
- The returned syntax and typed model share the same generated source and source identifier.

## Alternatives considered

### Construct raw text in BoxFerry

Rejected because it duplicates native spelling, ordering, repeatability, and validation policy in
the conversion layer.

### Expose mutable parser internals

Rejected because physical syntax mutation would couple consumers to representation details and
make source-span consistency difficult to guarantee.

### Wait for complete typed value grammars

Rejected because complete systemd and Podman value modeling is much larger than the safe first
conversion subset. The exact-value boundary permits useful generation without overstating what the
library understands.

## Follow-up: Volume `Copy`

`VolumeKey::Copy` is an opaque singleton. The builder rejects a second generated `Copy`, while
parsing retains every authored physical value and normal duplicate diagnostics. Its capability is
bounded to Podman 5.4.0–6.0.2 and its 20-unit fixture records dry-run command construction only. It
does not add `Image` to the Lens model or claim image pulls, volume creation, copy-up, runtime,
rootless, plugin, Compose, or BoxFerry semantics.

## Follow-up: Image `Image`, `ImageTag`, `ServiceName`, `AllTags`, `Arch`, `AuthFile`, `CertDir`, `ContainersConfModule`, and `OS`

`ImageKey::Image` is an opaque required singleton. The builder emits one `[Image]` `Image=` entry,
rejects a duplicate, and parse-back validation rejects missing or blank sources. It does not parse or
validate image references, transports, registries, tags, digests, authentication, TLS, platforms,
paths, pull behavior, service names, resource-name substitution, systemd behavior, runtime behavior,
Compose, or BoxFerry semantics.

`ImageKey::ImageTag` is an opaque singleton. The builder accepts one physical-line-safe value,
including empty text, and rejects a duplicate. It does not select a target resource name, apply
effective-last/default/quote behavior, substitute dependent units, or mutate graph identity.

`ImageKey::ServiceName` is an opaque singleton. The builder accepts one physical-line-safe value,
including empty text, and rejects a duplicate without deriving names, adding `.service`, expanding
templates/specifiers, or mutating document identity or graph edges.

`ImageKey::AllTags` is an opaque singleton. The builder accepts one physical-line-safe value,
including empty text, and rejects a duplicate without boolean parsing, default selection, pull-command
construction, or mutation of document identity or graph edges.

`ImageKey::Arch` is an opaque singleton. The builder accepts one physical-line-safe value,
including empty text, and rejects a duplicate without parsing architecture grammar, selecting a
host default, constructing a pull command, or mutating document identity or graph edges.

`ImageKey::AuthFile` is an opaque singleton. The builder accepts one physical-line-safe value,
including empty text, and rejects a duplicate without path validation or reads, credential or auth
JSON parsing, default or environment fallback, sensitivity inference, effective-last behavior,
registry authentication, pull/runtime behavior, or mutation of document identity or graph edges.

`ImageKey::CertDir` is an opaque singleton. The builder accepts one physical-line-safe value,
including empty text, and rejects a duplicate without path or certificate validation or reads,
containers-certs.d default or remote-client policy selection, sensitivity inference, effective-last
behavior, registry authentication, pull/runtime behavior, or mutation of document identity or graph edges.

`ImageKey::ContainersConfModule` is repeatable opaque physical-line text. The builder accepts every
physical-line-safe value in insertion order, including empty resets and duplicates, without path or module/configuration
reads, reset behavior, tokenization, unescaping, CLI validation, sensitivity inference, pull/runtime behavior, or mutation
of document identity or graph edges.

`ImageKey::OS` is an opaque singleton. The builder accepts one physical-line-safe value, including
empty text, and rejects a duplicate without operating-system grammar, host/default or platform
validation, tokenization, unescaping, CLI/runtime semantics, graph edges, effective-last behavior,
or cross-format mapping.
