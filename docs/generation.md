# Programmatic generation

QuadletLens 0.1.1 adds a validated construction boundary for tools that generate Quadlet files.
[ADR 0009](decisions/0009-validated-programmatic-generation.md) records its scope.

## Document construction

`QuadletDocumentBuilder` is created for one `QuadletUnitType`. Typed `push_container`, `push_pod`,
`push_network`, `push_volume`, `push_build`, and `push_image` methods prevent native keys from being written into the wrong
section. `push_systemd` adds open-ended directives to `[Unit]`, `[Service]`, or `[Install]`.
`push_systemd_unit` provides typed `Requires`, `Wants`, `After`, `Requisite`, `BindsTo`, `PartOf`,
`Upholds`, `Conflicts`, and `Before` spellings protected by capability and real-generator evidence.
All remain repeatable and accept exact physical-line-safe systemd unit-list text.

Container `ReloadCmd` and `ReloadSignal` are opaque singleton entries. The builder rejects a
duplicate of either key and rejects either insertion order of their mutually exclusive pair with a
key-only `RenderError::ConflictingSingletons`; it does not tokenize commands, validate signals, or
construct a reload action.

Pod `ExitPolicy` is an opaque singleton entry. The builder rejects a duplicate but does not parse
or validate its value, choose a default, or construct lifecycle behavior.

Pod `StopTimeout` is an opaque singleton entry. The builder rejects a duplicate but does not parse
seconds or `-1`, select or inject a timeout, or construct a stop action.

Pod `ServiceName` is an opaque singleton entry. The builder accepts blank, quoted, and
specifier-bearing physical-line-safe text and rejects a duplicate without extension handling,
name derivation, template/specifier evaluation, identity mutation, or restart semantics.

Pod `ContainersConfModule`, `DNS`, `DNSOption`, `DNSSearch`, `GIDMap`, `GlobalArgs`, `Label`,
`NetworkAlias`, `PodmanArgs`, and `UIDMap` are repeatable opaque physical entries. The builder
retains authored order and accepts empty source-safe entries, but does not apply Podman's reset,
tokenize arguments, parse mappings or labels, inspect DNS, allocate addresses, or interpret
network/runtime behavior. `HostName`, `IP`, `IP6`, `SubGIDMap`, and `SubUIDMap` are opaque Pod
singletons. Construction rejects a duplicate without hostname, IPAM, address, ID, namespace, or
host interpretation. Direct and subordinate map forms remain mutually exclusive; a builder emits
the same source-spanned mapping-conflict diagnostics as parsed input.

Repeated native keys retain insertion order. Native keys classified as singletons are rejected
when repeated. Generic systemd directives may repeat because their list and reset semantics are
directive-specific.

Container `ContainersConfModule`, `GlobalArgs`, and `ImageVolume` are repeatable opaque physical
entries. The builder retains their order and accepts empty source-safe entries; it does not apply
Podman's reset, tokenization, module-loading, or image-volume semantics. `HealthLogDestination`,
`HealthMaxLogCount`, `HealthMaxLogSize`, `HealthStartupCmd`, `HealthStartupInterval`,
`HealthStartupRetries`, `HealthStartupSuccess`, `HealthStartupTimeout`, and `ServiceName` are
opaque singletons: the builder rejects a duplicate but does not parse a path, number, size,
duration, command, service identity, or health behavior. Versioned generator evidence covers the
first eleven native keys through 6.0.2; `ImageVolume` intentionally has no positive support claim.

A generated container requires exactly one workload source. `ContainerKey::Image` selects an image
or native image/build reference; `ContainerKey::Rootfs` selects a Podman root filesystem. Building
a document with neither or both returns structured typed-model diagnostics.

For `.build` documents, `BuildKey::ImageTag`, `BuildKey::Network`, `BuildKey::Label`, `BuildKey::File`,
`BuildKey::BuildArg`, `BuildKey::Secret`, `BuildKey::GroupAdd`, `BuildKey::DNS`, `BuildKey::DNSOption`, `BuildKey::DNSSearch`, `BuildKey::Annotation`, `BuildKey::Environment`, `BuildKey::ContainersConfModule`, `BuildKey::GlobalArgs`, `BuildKey::Volume`, and `BuildKey::PodmanArgs` retain every exact physical-line value in insertion order. `BuildKey::Arch`,
`BuildKey::Variant`, `BuildKey::Pull`, `BuildKey::Retry`, `BuildKey::RetryDelay`, `BuildKey::TLSVerify`, `BuildKey::ForceRM`, `BuildKey::AuthFile`, `BuildKey::IgnoreFile`, and `BuildKey::ServiceName` are opaque singletons: construction rejects a second value, without parsing platform, pull-policy, integer, duration, boolean, auth-file path, ignore-file, or service-name grammar,
selecting defaults, or applying generator-effective-last behavior. Build `DNSSearch` does not apply reset or special-dot semantics. Build `Annotation` is not tokenized, unquoted, C-unescaped, reset, collapsed, sorted, OCI-validated, or image-metadata-inferred. Build `Environment` is not tokenized, unquoted, C-unescaped, reset, duplicate-name-selected, sorted, or host-looked-up. Build `ContainersConfModule` is not path-parsed, module-read, configuration-inspected, reset, deduplicated, tokenized, or normalized. Build `GlobalArgs` is not tokenized, reset, unquoted, C-unescaped, option-validated, or given semantic, security, or runtime effects. Build `AuthFile` is not read or path-validated, and its text is not credential-parsed or sensitivity-classified. Build `IgnoreFile` is not resolved or read, parsed as rules, defaulted, relative-path-normalized, or given effective-last behavior. Build `ServiceName` is not extension-stripped, defaulted from a basename, made template-aware, or used to mutate document, reference, dependency, or resource identity. `Network` is opaque to construction: callers
choose network modes, options, or exact `.network` basenames, and the builder neither parses nor
normalizes them. `File` is not path- or URL-classified, and construction does not apply the
generator's observed effective-last behavior. `Label` does not parse `KEY=VALUE`, unquote, select
duplicate names, collapse or sort a map, or validate label text. `BuildArg` does not parse key/value
text or resolve environments or secrets. `Secret` does not split comma-separated text, parse
argument names, resolve environment forms or paths, or materialize secret data. `PodmanArgs` does not split
or quote arguments, lower Compose `additional_contexts` or `service:` forms, resolve contexts, paths,
environments, images, or services, validate a CLI, or infer build/runtime behavior. `SetWorkingDirectory`
and `Target` are Build singletons. `Pull` neither applies a default nor normalizes spelling or
exposes effective-last behavior; it does not imply Compose boolean, registry, image-pull, build, or runtime semantics.
`Target` receives no build-stage grammar validation.

`ContainerKey::ContainerName` optionally selects the exact Podman runtime name. It is a singleton
and remains separate from the Quadlet filename and generated service identity.

`ContainerKey::Entrypoint` optionally overrides the image entrypoint and is a singleton distinct
from `ContainerKey::Exec`. Multiple entrypoint arguments use the documented JSON command-array
text; `EntryValue` retains that spelling exactly.

`ContainerKey::RunInit` is a singleton that carries exact authored one-line text. Leaving the key
out, writing `true`, and writing `false` remain three distinct builder results; raw noncanonical
text is preserved rather than interpreted as a boolean. Across the evidenced Podman
5.4.0-through-6.0.2 generator range, authored `true` emits exactly one `--init` argument and
authored `false` emits exactly one `--init=false` argument. This generator observation does not
inspect the init binary or establish runtime signal-forwarding and child-reaping behavior.

`ContainerKey::StopSignal` and `ContainerKey::StopTimeout` are singletons. Named/numeric signals and
non-negative integer seconds are the caller forms backed by current capability and generator
evidence; in particular, the builder preserves and emits `StopTimeout=0` without substituting a
default. The keys still use the shared raw-value boundary rather than dedicated semantic
validators.

`ContainerKey::Pull` is a singleton carrying exact authored one-line text. Omission stays absent;
the evidenced caller forms are `always`, `missing`, `never`, and `newer`, but the shared builder
does not reject other raw values or claim Podman accepts them.

`ContainerKey::PidsLimit` is a singleton carrying exact authored one-line text. Raw `EntryValue`
construction preserves omission, zero, overflow-sized text, and noncanonical values without
claiming they are accepted. `PidsLimit::unlimited()` and `PidsLimit::finite` are the stronger
additive construction path: they emit `-1` or exact nonzero ASCII-decimal spelling, rejecting
empty, nondecimal, and all-zero text. Finite values retain leading zeros and arbitrary-precision
digits without parsing into a machine integer, so this layer cannot overflow or silently normalize
the caller's value. The supported-window evidence does not establish a portable target maximum;
current generator evidence covers positive `127` and unlimited `-1`, not zero or runtime cgroup
enforcement.

`ContainerKey::HostName` is a singleton carrying exact authored one-line text. Omission stays
absent, and the shared `EntryValue` boundary rejects only NUL bytes and physical line endings; it
does not apply Compose RFC-1123 validation or normalize native values. For an isolated container,
Podman's default private UTS namespace lets the hostname apply. When a container joins a pod with
the default shared UTS namespace, the pod hostname wins. The generator fixture proves one emitted
logical `--hostname app.example` argument only, not runtime hostname behavior or pod precedence.

`ContainerKey::ShmSize` and `PodKey::ShmSize` are distinct singletons carrying exact authored
one-line text. The stronger `ShmSize::new` path accepts a non-negative ASCII-decimal amount with no
unit or one lowercase `b`, `k`, `m`, or `g`, preserving leading zeros and arbitrary precision
without parsing. `ShmSize::unlimited()` emits explicit `0`, distinct from omission and the
documented `64m` default. The builder does not select IPC mode or inspect `/dev/shm`; pod values
belong to the pod's shared-IPC context.

`ContainerKey::DropCapability` is repeatable. Each `EntryValue` stays one exact authored physical
line, including any space-separated list, and repeated entries retain insertion order. The builder
does not split, deduplicate, lowercase, or validate capability tokens. Podman's generator performs
its own observed lowercase expansion into `--cap-drop` arguments; callers must not treat that
generated-command observation as proof of runtime privilege state.

`ContainerKey::AddCapability` is repeatable with the same raw boundary. Omission stays absent;
empty native reset assignments, duplicates, order, case, and space-separated text are retained
exactly. QuadletLens does not apply the generator's observed splitting and lowercasing or interpret
`all`. Tagged Podman source and generated commands record those behaviors separately from this
builder and from runtime privilege state.

`ContainerKey::Tmpfs` is repeatable and retains each `EntryValue` as one exact physical line.
Omission, empty native reset assignments, duplicates, insertion order, case, destination spelling,
and options remain unchanged. The builder does not split or normalize
`CONTAINER-DIR[:OPTIONS]`, deduplicate destinations, validate target mount options, or route these
values through `ContainerKey::Volume`.

`ContainerKey::Sysctl` is repeatable and retains each `EntryValue` as one exact physical line.
Omission, empty native resets, duplicates, insertion order, case, whitespace, systemd
quoting/specifiers, and authored text remain unchanged. The builder does not parse `name=value`,
split a space-separated list, normalize assignments, validate namespaces, or apply kernel/runtime
rules. `PodKey` intentionally has no `Sysctl` variant.

`ContainerKey::Ulimit` is repeatable and retains each `EntryValue` as one exact physical line.
Omission, empty native resets, duplicates, insertion order, case, systemd quoting/specifiers, and
authored text remain unchanged. The builder does not split, unquote, or validate
`TYPE=SOFT[:HARD]`, infer defaults, or apply runtime resource-limit rules. `PodKey` intentionally
has no `Ulimit` variant.

`ContainerKey::AddDevice` is repeatable and retains each `EntryValue` as one exact physical line.
Omission, empty native resets, duplicates, insertion order, case, systemd quoting/specifiers,
whitespace-token-containing lines, a leading `-`, and authored text remain unchanged. The builder
does not split, unquote, parse host/container paths or permissions, check devices, or apply
conditional inclusion. `PodKey` intentionally has no `AddDevice` variant.

`ContainerKey::Memory` is a singleton with the same raw `EntryValue` preservation boundary.
`Memory::new` provides a stronger additive path for a positive ASCII-decimal amount with no unit
or one lowercase `b`, `k`, `m`, or `g`, retaining leading zeros and arbitrary precision without
parsing. It does not accept zero or make runtime, cgroup, page-size, swap, host-memory, rootless, or
cross-format claims. `PodKey` intentionally has no `Memory` variant.

`ContainerKey::LogDriver` is an opaque singleton. `ContainerKey::LogOpt` is opaque and
repeatable, including empty native reset assignments. Their physical values, duplicates, order,
quotes, and systemd specifiers remain exact; the builder does not split or parse options, validate
drivers/options, inject defaults, or claim runtime or cross-format behavior. Other native key
enums intentionally expose no logging variants in this slice.

`ContainerKey::IP` and `ContainerKey::IP6` are opaque singletons.
`ContainerKey::NetworkAlias` is opaque and repeatable, including empty native reset assignments.
The builder retains exact physical-line-safe values, duplicates, order, quotes, and specifiers;
it does not parse or validate addresses, aliases, IPAM, DNS, networks, runtime behavior, or
cross-format equivalence.

`NetworkKey::Driver` is an opaque singleton and `NetworkKey::Options` is opaque and repeatable,
including empty native reset assignments. The builder retains exact physical-line-safe values,
duplicates, order, quotes, and specifiers; it does not parse option keys or values, validate
driver availability or provider semantics, or apply generator reset, duplicate-collapse, sorting,
or version-specific bare-token behavior.

`NetworkKey::Label` is opaque and repeatable, including empty native reset assignments. The
builder retains exact physical-line-safe values, duplicates, order, bare values, embedded equals
signs, quotes, and specifiers; it does not tokenize label text, apply resets, collapse duplicate
keys, sort values, validate OCI labels, or adopt version-specific bare-token behavior.

`VolumeKey::Label` is likewise opaque and repeatable, including empty native reset assignments.
The builder retains exact physical-line-safe values, duplicates, order, bare values, embedded
equals signs, quotes, and specifiers; it does not tokenize label text, apply resets, collapse
duplicate keys, sort values, validate OCI labels, or adopt version-specific bare-token behavior.

`VolumeKey::ContainersConfModule` is opaque and repeatable, including empty native reset
assignments. The builder retains exact physical-line-safe values, duplicates, order, quotes,
specifiers, and continuations; it does not parse paths, read modules or configuration, apply
resets, tokenize, normalize, classify sensitivity, validate CLI options, or establish volume
creation, filesystem, lifecycle, security, runtime, Compose, or conversion behavior.

`VolumeKey::GlobalArgs` is opaque and repeatable, including empty native reset assignments. The
builder retains exact physical-line-safe values, duplicates, order, quotes, whitespace, specifiers,
C-escapes, and continuations; it does not tokenize, unquote, C-unescape, omit malformed values,
apply resets, validate arguments, infer sensitivity, read modules or configuration, or establish
volume creation, lifecycle, filesystem, security, runtime, Compose, or conversion behavior.

`VolumeKey::PodmanArgs` is opaque and repeatable, including empty native reset assignments. The
builder retains exact physical-line-safe values, duplicates, order, quotes, whitespace, specifiers,
C-escapes, and continuations; it does not tokenize, unquote, C-unescape, omit malformed values,
apply resets, deduplicate, validate options or a CLI, infer sensitivity, or establish dedicated-key,
volume-creation, lifecycle, filesystem, systemd, security, runtime, Compose, or conversion
behavior.

`VolumeKey::User` is an opaque singleton. The builder accepts one exact physical-line-safe value,
including empty text, and rejects a duplicate without parsing UID/name spelling, looking up a host
user, selecting defaults, or interpreting ownership, mounts, filesystems, lifecycle, runtime,
Compose, or conversion behavior.

`VolumeKey::Group` is an opaque singleton. The builder accepts one exact physical-line-safe value,
including empty text, and rejects a duplicate without parsing GID/name spelling, looking up an
account, selecting defaults, or interpreting ownership, mounts, filesystems, lifecycle, runtime,
Compose, or conversion behavior.

`VolumeKey::GID` is an opaque singleton. The builder accepts one exact physical-line-safe value,
including empty text, and rejects a duplicate without parsing or otherwise interpreting it.

`VolumeKey::ServiceName` is an opaque singleton. The builder accepts one exact physical-line-safe
value, including empty text, and rejects a duplicate without deriving or otherwise interpreting it.

`VolumeKey::Image` is an opaque singleton. Exact `.image` and `.build` values are reference-classified
after parsing and resolve by exact basename when their typed documents are present. Construction
retains one physical-line-safe value and rejects a duplicate.

`ImageKey::Image` is an opaque required singleton image source. Construction emits `[Image]` with
one `Image=` entry, rejects duplicates, and rejects missing or blank source text at parse-back
validation. It does not validate or interpret image-reference, transport, registry, tag/digest,
authentication, TLS, platform, path, pull, systemd/runtime, service naming, or substitution semantics.

`ImageKey::ImageTag` is an opaque singleton. Construction accepts one physical-line-safe value,
including empty text, and rejects a duplicate; it neither selects a resource name nor changes a
document-set reference or dependency.

`ImageKey::ServiceName` is an opaque singleton. Construction accepts one physical-line-safe value,
including empty text, and rejects a duplicate without deriving, normalizing, or applying it to identity.

`ImageKey::AllTags` is an opaque singleton. Construction accepts one physical-line-safe value,
including empty text, and rejects a duplicate without parsing booleans, selecting defaults, or
constructing pull commands.

`ImageKey::Arch` is an opaque singleton. Construction accepts one physical-line-safe value,
including empty text, and rejects a duplicate without parsing platform grammar, selecting host defaults,
or constructing pull commands.

`ImageKey::AuthFile` is an opaque singleton. Construction accepts one physical-line-safe value,
including empty text, and rejects a duplicate without validating or reading paths, parsing credentials,
classifying sensitivity, selecting defaults, or constructing pull commands.

`ImageKey::CertDir` is an opaque singleton. Construction accepts one physical-line-safe value,
including empty text, and rejects a duplicate without validating or reading paths or certificates,
selecting containers-certs.d defaults or remote-client policy, classifying sensitivity, or constructing pull commands.

`ImageKey::ContainersConfModule` is repeatable opaque physical-line text. Construction preserves every
value in insertion order, including empty resets and duplicates, without path parsing, module/configuration reads,
reset behavior, tokenization, CLI validation, or pull construction.

`ImageKey::Creds` is an opaque singleton. Construction accepts one physical-line-safe value,
including empty text, and rejects a duplicate. The keyed builder redacts this value in its own
debug output, but standalone `EntryValue` remains raw and unclassifiable until paired with the
`Creds` key; rendering and explicit text access remain exact. It does not split, parse, validate,
read, default, authenticate, or construct a pull command from credentials.

`ImageKey::DecryptionKey` is an opaque singleton. Construction accepts one physical-line-safe
value, including empty text, and rejects a duplicate. The keyed builder redacts this value in its
own debug output, while standalone `EntryValue` remains raw and unclassifiable until paired with
the `DecryptionKey` key; rendering and explicit text access remain exact. It does not split key or
passphrase text, validate or read files, decrypt, select defaults, authenticate, or construct a
pull command.

`ImageKey::GlobalArgs` is repeatable opaque physical-line text. Construction preserves every
physical-line-safe value in insertion order, including empty resets and duplicates, without
tokenization, reset behavior, unquoting, C-unescaping, CLI validation, sensitivity inference, or
pull construction.

`ImageKey::OS` is an opaque singleton. Construction accepts one physical-line-safe value,
including empty text, and rejects a duplicate without operating-system grammar, host/default or
platform validation, tokenization, unescaping, pull construction, or graph/runtime semantics.

`VolumeKey::Device` and `VolumeKey::Type` are separate opaque singletons. The builder retains
their exact physical-line-safe text, including blanks, quotes, and specifiers, and rejects a
second generated value. It does not apply Podman's last-value lookup, quote handling, source-path
or filesystem validation, Type/Device prerequisites, generated dependency rules, mount behavior,
or cross-format policy.

`NetworkKey::IPAMDriver` is an opaque singleton. `NetworkKey::Subnet`, `Gateway`, and `IPRange`
are opaque repeatable keys, including empty native reset assignments. The builder retains exact
physical-line-safe values, duplicates, order, quotes, and specifiers; it does not parse addresses
or ranges, infer a subnet, apply resets, zip target IPAM columns, validate IPAM drivers/defaults,
or make network/runtime/cross-format claims.

`DNS`, `DNSOption`, `DNSSearch`, `ExposeHostPort`, `Annotation`, `Mask`, and
`Unmask` are repeatable `ContainerKey` variants. The builder preserves each physical-line-safe
`EntryValue` in insertion order, including duplicates and reset assignments.

`AppArmor`, `NoNewPrivileges`, `SeccompProfile`, and the five `SecurityLabel*` variants are
singletons. Their values remain opaque and a second assignment is rejected.

The builder performs no address, port, OCI, boolean, profile, SELinux, path, filesystem, host, or
runtime validation. Unsupported scopes expose no typed construction key.

`ContainerKey::Secret` is repeatable. Its exact value may select mounted-file or environment
exposure and carry target, UID, GID, and mode options; the builder preserves those options without
reading the referenced Podman secret.

`ContainerKey::Label` is repeatable. Each value remains an exact native `key=value` assignment;
the builder preserves insertion order and does not enforce reverse-DNS naming recommendations or
merge duplicate label names.

`AutoUpdate`, `CgroupsMode`, `EnvironmentHost`, `HttpProxy`, `ReadOnlyTmpfs`, `Retry`,
`RetryDelay`, `StartWithPod`, `SubGIDMap`, `SubUIDMap`, `Timezone`, and `HealthOnFailure` are
opaque singleton values. Repeatable `GIDMap`, `UIDMap`, and `Mount` retain exact physical-line
text in insertion order. Generated documents retain authored reset assignments; only the target
generator's effective-value behavior is asserted by isolated generator fixtures. `Retry`,
`RetryDelay`, and `HttpProxy` have target-version capability boundaries, while `StartWithPod`
has a target generator command boundary at Podman 5.7.0. The builder never reads process/proxy
environments, host mapping or timezone files, image registries, or the filesystem; it neither
performs pulls nor starts, mounts, retries, or executes health checks. `Mount` remains opaque
because the current document-set grammar has no evidenced native `--mount` reference extractor.

`PodKey::UserNS` configures the namespace shared by pod members and is a singleton. It is distinct
from `ContainerKey::UserNS`: Podman ignores container-level namespace selection after a container
joins a pod.

`build` emits deterministic text, reparses it through the normal syntax and typed-model pipeline,
and fails if that result contains an error. Successful output exposes the source text, typed
document, and complete parse result.

## Value boundary

`EntryValue` is exact authored text on one physical line. It rejects NUL bytes and line endings
but does not validate, quote, or normalize key-specific grammar. Callers select the appropriate
native spelling for the typed keys in the
[specification coverage ledger](roadmap.md#specification-coverage-ledger).

This is an explicit boundary, not a claim that all value forms are interchangeable. Future
key-specific constructors can add stronger guarantees once exact Podman-version behavior and
systemd escaping rules are covered by evidence. In particular, raw noncanonical `RunInit` text and
raw one-line negative, fractional, or overflow-sized timeout text is preserved rather than rejected
by `EntryValue`; that preservation is not a claim that Podman accepts or gives useful runtime
meaning to those values.

## Example

```rust
use quadlet_lens::{
    model::{ContainerKey, QuadletUnitType},
    render::{EntryValue, Memory, PidsLimit, QuadletDocumentBuilder, ShmSize},
    source::SourceId,
};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
builder.push_container(
    ContainerKey::ContainerName,
    EntryValue::new("example-application")?,
)?;
builder.push_container(
    ContainerKey::AddHost,
    EntryValue::new("host.docker.internal:host-gateway")?,
)?;
builder.push_container(
    ContainerKey::Image,
    EntryValue::new("example.invalid/application:1")?,
)?;
builder.push_container(
    ContainerKey::Entrypoint,
    EntryValue::new(r#"["/usr/bin/env","php"]"#)?,
)?;
builder.push_container(ContainerKey::RunInit, EntryValue::new("true")?)?;
builder.push_container(ContainerKey::StopSignal, EntryValue::new("SIGUSR1")?)?;
builder.push_container(ContainerKey::StopTimeout, EntryValue::new("0")?)?;
builder.push_container(ContainerKey::Pull, EntryValue::new("newer")?)?;
builder.push_container(ContainerKey::PidsLimit, PidsLimit::finite("127")?.into())?;
builder.push_container(ContainerKey::HostName, EntryValue::new("app.example")?)?;
builder.push_container(ContainerKey::ShmSize, ShmSize::new("128m")?.into())?;
builder.push_container(ContainerKey::Memory, Memory::new("268435456b")?.into())?;
builder.push_container(ContainerKey::DropCapability, EntryValue::new("CAP_NET_ADMIN")?)?;
builder.push_container(
    ContainerKey::DropCapability,
    EntryValue::new("CAP_DAC_OVERRIDE CAP_IPC_OWNER")?,
)?;
builder.push_container(ContainerKey::AddCapability, EntryValue::new("CAP_NET_BIND_SERVICE")?)?;
builder.push_container(ContainerKey::Tmpfs, EntryValue::new("/data:mode=755,uid=1009,gid=1009")?)?;
builder.push_container(ContainerKey::Sysctl, EntryValue::new("net.ipv4.ip_forward=1")?)?;
builder.push_container(ContainerKey::Ulimit, EntryValue::new("nproc=4096:8192")?)?;
builder.push_container(
    ContainerKey::AddDevice,
    EntryValue::new("/dev/null:/dev/null:r")?,
)?;
builder.push_container(
    ContainerKey::Mask,
    EntryValue::new("/proc/acpi:/sys/firmware")?,
)?;
builder.push_container(ContainerKey::Unmask, EntryValue::new("ALL")?)?;
builder.push_container(
    ContainerKey::Environment,
    EntryValue::new("APP_ENV=production")?,
)?;
builder.push_container(
    ContainerKey::Label,
    EntryValue::new("org.example.application=example")?,
)?;

let generated = builder.build(SourceId::new(1))?;
assert_eq!(
    generated.text(),
    concat!(
        "[Container]\n",
        "ContainerName=example-application\n",
        "AddHost=host.docker.internal:host-gateway\n",
        "Image=example.invalid/application:1\n",
        "Entrypoint=[\"/usr/bin/env\",\"php\"]\n",
        "RunInit=true\n",
        "StopSignal=SIGUSR1\n",
        "StopTimeout=0\n",
        "Pull=newer\n",
        "PidsLimit=127\n",
        "HostName=app.example\n",
        "ShmSize=128m\n",
        "Memory=268435456b\n",
        "DropCapability=CAP_NET_ADMIN\n",
        "DropCapability=CAP_DAC_OVERRIDE CAP_IPC_OWNER\n",
        "AddCapability=CAP_NET_BIND_SERVICE\n",
        "Tmpfs=/data:mode=755,uid=1009,gid=1009\n",
        "Sysctl=net.ipv4.ip_forward=1\n",
        "Ulimit=nproc=4096:8192\n",
        "AddDevice=/dev/null:/dev/null:r\n",
        "Environment=APP_ENV=production\n",
        "Label=org.example.application=example\n",
    ),
);
# Ok(())
# }
```

## Volume `Copy`

`VolumeKey::Copy` is an opaque singleton. The builder renders exact physical-line-safe text and
rejects a second generated value; it does not parse booleans or add an `Image` field. The recorded
generator facts are dry-run command text only, not copy-up, volume creation, image pulls, runtime,
rootless, plugin, Compose, or BoxFerry behavior.

## Experimental Artifact and shared Quadlet options

`QuadletDocumentBuilder::push_artifact` accepts the thirteen typed experimental Artifact keys.
`Artifact` is required and nonblank at build time; `ContainersConfModule`, `GlobalArgs`, and
`PodmanArgs` retain ordered repeats while other Artifact keys are singletons. `Creds` and
`DecryptionKey` are redacted only from repository-owned debug output after they are paired with
their key; explicit rendering remains exact. `push_quadlet(QuadletKey::DefaultDependencies, ...)`
accepts one opaque physical-line-safe value on every typed unit. Neither API parses values, opens
files, contacts a registry, derives a service name, or constructs runtime behavior.
