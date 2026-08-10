# Software architecture

## Purpose

QuadletLens provides a native Quadlet representation and target-aware validation without invoking Podman or systemd during normal parsing and rendering.

## Layers

```text
source files ───────────────────────────────────────────────────┐
                                                               ▼
native generated values ──▶ document builder ──▶ unit syntax documents ──▶ typed Quadlet documents ──▶ document set and graph
                                │                     │                            │                              │
                                ├──▶ typed keys       ├──▶ comments/order          ├──▶ native value types        ├──▶ references
                                ├──▶ exact values     ├──▶ repeated keys           ├──▶ unknown fields            ├──▶ dependencies
                                └──▶ parse-back       ├──▶ source spans            │                              │
                                                      │                            │                              │
                                                      ├────────────────────────────┼──────────────────────────────┼──▶ renderer
                                                      │                            │                              │
                                                      └────────────────────────────┴──────────────────────────────┤
target profile ──▶ capability catalogue ─────────────────────────────────────────────────────────────────────────┴──▶ validation report
```

### Unit syntax

The syntax layer represents sections and ordered entries rather than flattening them into maps. It owns comments, blank lines, continuations, quoting, repeated keys, reset behavior, and source spans.

The initial dependency-free grammar stores immutable source text and classifies every physical line
as blank, comment, section, entry, continuation, or recoverable invalid input. It retains repeated
keys, comment markers, continuation context, line endings, source spans, and specifier-shaped text
without decoding values. [ADR 0002](decisions/0002-loss-aware-systemd-syntax.md) defines this
boundary. Supporting every systemd file type and every systemd parser extension is not required.

Valid parse results can also render a conservative canonical form. It normalizes structural
indentation, assignment spacing, and line endings while retaining order, repetition, comments,
continuations, raw value spelling, and specifiers. It refuses invalid syntax and does not perform
typed normalization. [ADR 0003](decisions/0003-conservative-canonical-syntax-rendering.md) defines
that boundary.

### Typed Quadlet model

Typed documents represent native Quadlet unit types, including container, pod, network, volume, image, build, kube, and artifact units as supported by target versions.

Generic systemd sections and unknown Quadlet entries remain attached to the document. Typed conversion cannot be destructive.

The first implemented subset covers `.container`, `.pod`, `.network`, `.volume`, and the minimal
`.build` core. Build `ImageTag`, `Network`, `Label`, `File`, `BuildArg`, `Secret`, `GroupAdd`, `DNS`, `DNSOption`, `DNSSearch`, `Annotation`, `Environment`, `ContainersConfModule`, `GlobalArgs`, and `PodmanArgs` remain repeatable and source ordered, while
`SetWorkingDirectory`, `Target`, `Arch`, `Variant`, `Pull`, `Retry`, `RetryDelay`, `TLSVerify`, `ForceRM`, `AuthFile`, `IgnoreFile`, and `ServiceName` remain opaque singletons. `File` stays unclassified and the model does
not apply Podman's observed effective-last behavior. `Pull` does not validate policy spelling, inject a default,
normalize text, or expose effective-last behavior. `Label`, `BuildArg`, `Secret`, `GroupAdd`, `DNS`, `DNSOption`, `DNSSearch`, and `PodmanArgs` are opaque
physical-line text: they are not parsed as `KEY=VALUE`, unquoted, selected by duplicate name,
map-collapsed, sorted, or validated. Build `Secret` additionally does not parse commas, arguments,
environment forms, or paths, and never materializes secret data. Build `DNSSearch` does not apply reset or special-dot semantics. Build `Annotation` preserves raw ordered physical lines without tokenization, unquoting, C-unescaping, reset, duplicate-key collapse, sorting, OCI validation, or image-metadata inference. Build `Environment` likewise preserves raw ordered physical lines without tokenization, unquoting, C-unescaping, reset, duplicate-name selection, sorting, or host lookup. Build `ContainersConfModule` preserves raw ordered physical lines without path parsing, module reads, configuration inspection, reset, deduplication, tokenization, or normalization. Build `GlobalArgs` preserves raw ordered physical lines without tokenization, reset, unquoting, C-unescaping, option validation, or inferred semantic, security, or runtime effects. Build `AuthFile` is neither read nor path-validated, and its text is not classified as credential content or sensitive data. Build `IgnoreFile` is neither resolved nor read, parsed as ignore rules, defaulted from `.containerignore` or `.dockerignore`, relative-path-normalized, or assigned generator-effective-last semantics. `PodmanArgs` does not split or quote
arguments, resolve contexts, paths, environments, images, or services, validate a CLI, or imply build/runtime behavior.
Exact `.container` Image-to-`.build` and
`.build` Network-to-`.network` references resolve only in document sets. It classifies the
native keys needed by the first conversion, keeps repeated section occurrences and entries in
source order, and owns the authored text plus source span for every typed name and value segment.
That boundary includes container execution identity/context and pod-level user-namespace selection
without resolving host users, groups, paths, or namespaces, plus container stop signal, timeout,
image pull policy, process-ID limit, and hostname values without normalizing their native spelling.
Container and pod shared-memory sizes are likewise recognized as opaque singleton values. Focused
process-ID-limit and shared-memory-size construction helpers are additive to, and do not narrow,
the raw value boundary. Container `DropCapability` and `AddCapability` are recognized as separate
opaque repeatable keys; their authored entries, empty reset assignments, and space-separated text
are neither merged nor normalized. Container `Tmpfs` is a third opaque repeatable boundary:
omission, empty resets, duplicates, order, case, destination/options text, and its distinction from
`Volume` remain intact rather than being interpreted as mounts by the model.
Container `Sysctl` is a fourth opaque repeatable boundary. Each authored one-line entry, including
empty resets, whitespace, quoting, specifiers, duplicates, and case, remains intact; the model does
not parse `name=value` assignments or import Podman/kernel namespace rules. Pod `Sysctl` remains an
unknown preserved entry.
Container `Ulimit` is a fifth opaque repeatable boundary. Omission, every authored one-line value,
empty resets, duplicates, order, case, quoting, and specifiers remain intact. The model does not
split, unquote, or validate `TYPE=SOFT[:HARD]`; Pod `Ulimit` remains an unknown preserved entry.
Container `AddDevice` is a sixth opaque repeatable boundary. Omission, every physical value,
empty resets, duplicates, order, case, quotes/specifiers, whitespace-token-containing lines, and
a leading `-` remain intact without splitting, unquoting, or device validation. Pod `AddDevice`
remains an unknown preserved entry. Tagged generator behavior is evidence, not native-model logic.
Container `Memory` is an opaque singleton boundary introduced natively in Podman 5.5.0. Omission,
duplicates, empty assignments, quoting, specifiers, and exact one-line values remain source-aware;
the model does not infer runtime limits or cross-format equivalence. A focused positive-decimal
construction helper is additive to the unchanged raw-value boundary. Pod `Memory` remains unknown.
Container `LogDriver` is an opaque singleton, while `LogOpt` is opaque, repeatable, and resettable.
Both preserve physical values, quotes, specifiers, and authored order without driver/option
validation, key/value parsing, default injection, or runtime interpretation. Other native scopes
remain unknown and preserved.
Container `IP` and `IP6` are opaque singletons, while `NetworkAlias` is opaque, repeatable, and
resettable. Physical values, duplicates, order, quotes, specifiers, continuations, and standard
cardinality diagnostics remain source-aware without address, IPAM, DNS, network, runtime, or
cross-format interpretation. These keys are typed only for containers.
Network `Driver` is an opaque singleton and `Options` is opaque, repeatable, and resettable.
Every authored physical option remains available to callers; the model does not apply Podman's
effective reset, tokenization, duplicate-key collapse, sorting, or version-specific bare-token
handling, and does not validate driver availability or provider-specific options.
Network `Label` is likewise opaque, repeatable, and resettable. Its physical entries preserve
empty resets, duplicates, bare values, embedded equals signs, quotes, specifiers, continuations,
and source order; target tokenization, map collapse, sorting, and bare-token behavior stay
generator evidence rather than model semantics.
Network `Internal` and `IPv6` are separate opaque singletons. Omission, literal true/false,
duplicates, invalid or vendor-defined spelling, quotes, specifiers, and continuations remain
source-aware; the model does not parse booleans or adopt Podman's last-value and invalid-as-false
lookup behavior. `Internal` remains driver-conditional, while `IPv6` describes dual-stack behavior;
neither key implies an IPv4-enable spelling.
`IPAMDriver` is likewise an opaque singleton, while `Subnet`, `Gateway`, and `IPRange` are
opaque repeatable physical entries. QuadletLens preserves their resets, duplicates, quoting,
specifiers, continuations, and authored order without calculating the generator's indexed
subnet/gateway/range groups, inferring IPv4 or IPv6 behavior, or creating a network.
Volume `Driver` and `Options` are opaque singletons. `Options` is one raw mount-option string,
not a repeatable network-style option map: duplicates, blank values, bare text, quotes,
specifiers, and continuations remain physical source entries, while generated construction rejects
duplicate singletons. Tagged-source and generator evidence describe the target's last-value,
blank-omission, Device prerequisite, and quote-boundary behavior without importing it into the
model or claiming driver/plugin, mount, rootless, runtime, or cross-format semantics.
Volume `Label` is opaque, repeatable, and resettable. Its physical entries preserve empty resets,
duplicates, bare values, embedded equals signs, quotes, specifiers, continuations, and source
order; target tokenization, map collapse, sorting, and bare-token behavior stay generator evidence
rather than model semantics.
Volume `ContainersConfModule` is opaque and repeatable. Its physical entries preserve empty
resets, duplicates, quotes, specifiers, continuations, and source order; target reset,
continuation presentation, and `--module` construction remain generator evidence rather than
model semantics. QuadletLens does not parse paths, read modules or configuration, infer
sensitivity, or establish volume creation, filesystem, lifecycle, security, runtime, Compose, or
conversion behavior.
Image `GlobalArgs` is opaque and repeatable. Its physical entries preserve empty resets,
duplicates, quotes, whitespace, specifiers, C-escapes, continuations, and source order; target
tokenization, unquoting, C-unescaping, malformed-line omission, reset, and placement before
`image pull` stay generator evidence rather than model semantics. QuadletLens does not parse or
validate arguments, infer sensitivity, or establish image-pull, runtime, Compose, or conversion
behavior.
Volume `GlobalArgs` is opaque and repeatable. Its physical entries preserve empty resets,
duplicates, quotes, whitespace, specifiers, C-escapes, continuations, and source order; target
tokenization, unquoting, C-unescaping, malformed-line omission, reset, and command placement stay
generator evidence rather than model semantics. QuadletLens does not parse or validate arguments,
infer sensitivity, or establish volume creation, lifecycle, filesystem, runtime, Compose, or
conversion behavior.
Volume `PodmanArgs` is opaque and repeatable. Its physical entries preserve empty resets,
duplicates, quotes, whitespace, specifiers, C-escapes, continuations, and source order; target
tokenization, unquoting, C-unescaping, malformed-line omission, reset, and terminal placement
remain generator evidence rather than model semantics. QuadletLens does not parse a CLI, assign
dedicated-key behavior, infer sensitivity, or establish volume creation, lifecycle, filesystem,
systemd, runtime, Compose, or conversion behavior.
Volume `User` is an opaque singleton: authored physical lines and the ordinary duplicate diagnostic
remain source-aware, while UID/name parsing, host lookup, generator defaults, ownership, mount,
filesystem, runtime, Compose, and conversion behavior remain outside the model.
Volume `GID` is likewise an opaque singleton: authored physical lines and the ordinary duplicate
diagnostic remain source-aware, without parsing or otherwise interpreting the value.
Volume `ServiceName` is likewise an opaque singleton: authored physical lines and the ordinary
duplicate diagnostic remain source-aware, without naming or identity interpretation.
Volume `Image` is an opaque singleton with only exact `.image` and `.build` reference classification;
both resolve when their corresponding typed documents are present. The minimal native Image unit
types its required opaque `Image` source, opaque singleton `ImageTag`/`ServiceName`/`AllTags`/`Arch`/`AuthFile`/`CertDir`/`Creds`/`DecryptionKey`/`OS`, and repeatable `ContainersConfModule`/`GlobalArgs`; target
resource-name/service-name substitution, boolean/platform/auth-file/certificate-directory/operating-system/default handling, pull commands, credentials, certificates, and generated dependencies remain outside the model.
Cross-format prefix-complete mapping policy remains BoxFerry-owned.
Container `DNS`, `DNSOption`, `DNSSearch`, `ExposeHostPort`, `Annotation`, `Mask`,
and `Unmask` are opaque repeatable boundaries. They preserve every physical value, reset
assignment, duplicate, and source order without key-specific parsing or runtime interpretation.

Container `AppArmor`, `NoNewPrivileges`, `SeccompProfile`, and the five
`SecurityLabel*` keys are opaque singletons. Duplicate authored assignments receive the standard
singleton diagnostic; programmatic construction rejects a second value.

These keys are typed only in their documented `[Container]` scope. Other scopes stay unknown and
preserved. Capability and generator evidence describe target command construction separately from
the source-aware model.

Generic `[Unit]`, `[Service]`, and `[Install]` entries remain open-ended. Unknown sections and keys
remain explicit entries rather than validation losses.

Value interpretation is deliberately conservative. Environment-file and mount sources receive
lexical path classifications, and `.image`, `.build`, `.pod`, `.network`, and `.volume` references
receive explicit reference kinds. Systemd command lines, environment assignments, ports, health
commands, and raw Podman arguments remain opaque until their behavior is protected by focused
parsers and exact generator evidence. [ADR 0005](decisions/0005-source-aware-native-typed-model.md)
defines this boundary.

### Document set and dependency graph

A Quadlet application commonly spans multiple files. The implemented document set pairs every
typed document with a validated unit-file basename, requires unique source identities, and resolves
native references by exact basename without consulting the filesystem. The graph retains resolved,
missing, and ambiguous references; resolved relationships become deterministic dependency edges.
Source-labelled diagnostics report missing targets, ambiguous targets, and duplicate basenames.

### Capability catalogue

The catalogue describes when a unit type, section, key, value form, or fallback is available. It is
strict versioned TOML, data-driven, and independently validated. Coverage ranges are separate from
the rolling product support window and upstream introduction claims, known patch bugs override
native support, and documentation evidence remains distinguishable from exact generator execution.
Catalogue `value_forms` identify evidenced and supported caller representations; they do not add
key-specific semantic validation to the source-aware model or the physical-line-safe builder.
The built-in catalogue starts at Podman 5.4.0 and currently verifies the first-conversion subset
through current Podman 6.0.2. [ADR 0004](decisions/0004-versioned-capability-catalogue.md) defines the core schema;
[ADR 0006](decisions/0006-rolling-support-window-and-generator-evidence.md) defines ranged evidence
and the rolling upper target.

### Target validator

Validation combines documents, a target profile, and catalogue data. It reports syntax validity, native-model constraints, cross-file references, target support, deprecation, known issues, and fallback availability.

### Renderer

The implemented programmatic renderer accepts typed native keys, exact one-line native values, and
open-ended generic systemd directives. It produces deterministic section order and reparses every
generated document before returning it. It deliberately does not invent quoting or normalization
rules for opaque systemd and Podman value grammars. [ADR 0009](decisions/0009-validated-programmatic-generation.md)
defines this boundary.

The broader renderer direction supports:

- preservation-oriented edits based on syntax documents
- deterministic canonical files based on typed documents
- target-aware rendering when value syntax differs by version

Rendering never installs or executes the result.

## Target profile

A target profile may include:

- Podman minimum and optional maximum version
- exact detected Podman version
- systemd minimum or detected version
- rootless or rootful mode
- allowed fallback policy
- explicit capability overrides for distribution backports

Output claimed compatible with a range must validate across the entire range.

## Dependency rules

- Syntax does not depend on typed Quadlet models.
- Typed models may carry source references but not parser internals.
- Capability data does not depend on BoxFerry.
- Validation depends on models and capability interfaces.
- External generator verification is an optional test/tooling layer, not parser behavior.
- QuadletLens never depends on BoxFerry.
