# Native Quadlet coverage

This document distinguishes loss-aware parsing from typed construction and version-evidenced
generation. It was audited against the current official
[Quadlet manual](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html) and the
supported Podman 5.4 floor on 2026-08-07. The exact current untyped-key inventory and promotion
order live in the [roadmap](roadmap.md).

## Coverage layers

| Layer       | Contract                                                                                                     |
| ----------- | ------------------------------------------------------------------------------------------------------------ |
| Syntax      | Ordered sections, repeated keys, continuations, comments, unknown keys, and systemd specifiers are retained. |
| Native type | A unit, section, or key can be inspected and constructed through a typed public API.                         |
| Capability  | The data catalogue states support over an explicit Podman version range and cites evidence.                  |
| Generator   | Repository fixtures have been accepted by the recorded real Quadlet generators.                              |

Recognition is not a version claim. A key is ready for BoxFerry generation only when the native
type, capability, and relevant generator evidence agree.

## Unit types

| Quadlet unit | Syntax preservation | Typed document/builder                                                                                                                                                                                                                                                                                                                                   | Current BoxFerry output                                         |
| ------------ | ------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------- |
| `.container` | yes                 | yes                                                                                                                                                                                                                                                                                                                                                      | yes                                                             |
| `.pod`       | yes                 | yes                                                                                                                                                                                                                                                                                                                                                      | optional explicit grouping                                      |
| `.network`   | yes                 | yes                                                                                                                                                                                                                                                                                                                                                      | application-owned networks                                      |
| `.volume`    | yes                 | yes                                                                                                                                                                                                                                                                                                                                                      | application-owned volumes                                       |
| `.image`     | yes                 | `Image`, `ImageTag`, `ServiceName`, `AllTags`, `Arch`, `AuthFile`, `CertDir`, `ContainersConfModule`, `Creds`, `DecryptionKey`, `GlobalArgs`, `OS`, `PodmanArgs`, `Policy`, `Retry`, `RetryDelay`, `TLSVerify`, `Variant`                                                                                                                                | no                                                              |
| `.build`     | yes                 | `ImageTag`, `Network`, `Label`, `File`, `SetWorkingDirectory`, `Target`, `BuildArg`, `Secret`, `Arch`, `Variant`, `Pull`, `Retry`, `RetryDelay`, `TLSVerify`, `ForceRM`, `AuthFile`, `IgnoreFile`, `ServiceName`, `GroupAdd`, `DNS`, `DNSOption`, `DNSSearch`, `Annotation`, `Environment`, `ContainersConfModule`, `GlobalArgs`, `Volume`, `PodmanArgs` | no                                                              |
| `.kube`      | yes                 | `AutoUpdate`, `ConfigMap`, `ContainersConfModule`, `ExitCodePropagation`, `GlobalArgs`, `KubeDownForce`, `LogDriver`, `LogOpt`, `Network`, `PodmanArgs`, `PublishPort`, `RemapGid`, `RemapUid`, `RemapUidSize`, `RemapUsers`, `ServiceName`, `SetWorkingDirectory`, `UserNS`, `Yaml`                                                                     | no                                                              |
| `.artifact`  | yes                 | `Artifact`, `AuthFile`, `CertDir`, `Creds`, `DecryptionKey`, `Quiet`, `Retry`, `RetryDelay`, `ServiceName`, `TLSVerify`, `ContainersConfModule`, `GlobalArgs`, `PodmanArgs`; shared `[Quadlet] DefaultDependencies`                                                                                                                                      | experimental upstream surface; no registry or filesystem access |

Unsupported native sections remain available through the syntax tree. They are not mislabeled as
one of the eight typed unit types.

## Typed key boundary

| Section                            | Typed keys                                                                                                                                                                                                                                                                                                                                          |
| ---------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `[Container]`                      | 90 keys; see the [coverage ledger](roadmap.md#specification-coverage-ledger)                                                                                                                                                                                                                                                                        |
| `[Pod]`                            | `AddHost`, `PodName`, `PublishPort`, `Network`, `Volume`, `UserNS`, `ShmSize`, `ExitPolicy`, `StopTimeout`, `ServiceName`, `ContainersConfModule`, `DNS`, `DNSOption`, `DNSSearch`, `GIDMap`, `GlobalArgs`, `HostName`, `IP`, `IP6`, `Label`, `NetworkAlias`, `PodmanArgs`, `SubGIDMap`, `SubUIDMap`, `UIDMap`                                      |
| `[Network]`                        | `NetworkName`, `Driver`, `Options`, `Label`, `Internal`, `IPv6`, `IPAMDriver`, `Subnet`, `Gateway`, `IPRange`, `ContainersConfModule`, `DisableDNS`, `DNS`, `GlobalArgs`, `InterfaceName`, `NetworkDeleteOnStop`, `PodmanArgs`, `ServiceName`                                                                                                       |
| `[Volume]`                         | `VolumeName`, `Driver`, `Options`, `Label`, `Device`, `Type`, `Copy`, `ContainersConfModule`, `GlobalArgs`, `PodmanArgs`, `User`, `Group`, `UID`, `GID`, `ServiceName`, `Image`                                                                                                                                                                     |
| `[Build]`                          | repeatable `ImageTag`/`Network`/`Label`/`File`/`BuildArg`/`Secret`/`GroupAdd`/`DNS`/`DNSOption`/`DNSSearch`/`Annotation`/`Environment`/`ContainersConfModule`/`GlobalArgs`/`Volume`/`PodmanArgs`, singleton `SetWorkingDirectory`/`Target`/`Arch`/`Variant`/`Pull`/`Retry`/`RetryDelay`/`TLSVerify`/`ForceRM`/`AuthFile`/`IgnoreFile`/`ServiceName` |
| `[Image]`                          | required opaque singleton `Image`; opaque singletons `ImageTag`, `ServiceName`, `AllTags`, `Arch`, `AuthFile`, `CertDir`, `Creds`, `DecryptionKey`, `OS`, `Policy`, `Retry`, `RetryDelay`, `TLSVerify`, `Variant`; repeatable `ContainersConfModule`, `GlobalArgs`, `PodmanArgs`                                                                    |
| `[Kube]`                           | required repeatable opaque `Yaml`; repeatable `AutoUpdate`, `ConfigMap`, `ContainersConfModule`, `GlobalArgs`, `LogOpt`, `Network`, `PodmanArgs`, `PublishPort`, `RemapGid`, `RemapUid`; opaque singletons `ExitCodePropagation`, `KubeDownForce`, `LogDriver`, `RemapUidSize`, `RemapUsers`, `ServiceName`, `SetWorkingDirectory`, `UserNS`        |
| `[Artifact]`                       | required opaque singleton `Artifact`; repeatable `ContainersConfModule`, `GlobalArgs`, `PodmanArgs`; opaque singletons `AuthFile`, `CertDir`, `Creds`, `DecryptionKey`, `Quiet`, `Retry`, `RetryDelay`, `ServiceName`, `TLSVerify`                                                                                                                  |
| `[Quadlet]`                        | opaque singleton `DefaultDependencies` for every typed Quadlet unit                                                                                                                                                                                                                                                                                 |
| `[Unit]`, `[Service]`, `[Install]` | Open-ended generic systemd directives with source/order preservation; typed generation and explicit capability evidence exist for nine reviewed `[Unit]` relationship keys and `[Service]` `Restart=`.                                                                                                                                              |

All current Container, Pod, Network, Image, Kube, Artifact, and Quadlet-section keys are typed. Artifact remains
experimental and is capability-supported only from Podman 5.7.0 through 6.0.2; the generator fixture proves
pre-5.7 exclusion plus command construction only. `ImageVolume` is intentionally typed without a
positive capability claim while its target support remains unknown. Network/Image completion fixtures record
ordered/reset command text and the 5.5.0/5.6.0 boundaries without claiming network or pull runtime behavior. The complete
lists, plus every current build, image, kube, artifact, and Quadlet-section key, are maintained in
the [specification coverage ledger](roadmap.md#specification-coverage-ledger).

## Next promotion

The execution-identity subset available since the Podman 5.4 floor includes container `User`,
`Group`, `UserNS`, repeatable `GroupAdd`, `WorkingDir`, and `ReadOnly`, plus pod-level `UserNS` for
the namespace shared by pod members. The exact generator matrix confirms the corresponding
`--user`, `--userns`, `--group-add`, `--workdir`, and `--read-only` output across all 20 recorded
patch releases through 6.0.2. Values remain exact authored text; QuadletLens does not resolve
users, groups, paths, or namespace state.

The secret subset includes repeatable mounted-file and environment-variable Podman secret
references, with target, UID, GID, and mode option spellings retained as exact native text.
Generator evidence proves the emitted `--secret` arguments; secret creation, content, rotation,
and runtime availability remain caller-owned concerns.

The label subset includes ordered, repeatable container `Label=key=value` assignments. The full
generator matrix proves ordinary, empty, and JSON-like quote/whitespace values from Podman 5.4.0
through 6.0.2. It explicitly accepts the literal-space systemd spelling emitted by 5.4.x and the
equivalent `\x20` spelling emitted from 5.5.0 onward. Label name conventions, duplicate-name
semantics, and labels owned by network or volume resources remain caller- or future-model
responsibilities.

The workload-source subset accepts exactly one container `Image` or `Rootfs` entry. `Rootfs` is
documented at the Podman 5.4 floor, exercised by the public `containers/qm` unit, and verified as a
generated `--rootfs` argument through the supported generator matrix. QuadletLens retains its exact
value and does not inspect the host filesystem, parse overlay-rootfs options, or verify SELinux
labels.

The minimal Image subset accepts required opaque `Image`, opaque singleton `ImageTag`/`ServiceName`/`AllTags`/`Arch`/`AuthFile`/`CertDir`/`Creds`/`DecryptionKey`, and repeatable `ContainersConfModule`/`GlobalArgs` text. The
document set resolves exact lowercase Container and Volume `.image` references to a matching typed
Image document. The 20-release dry-run fixtures record source pulls and target-only ImageTag
resource-name/default/quote, service-name, AllTags boolean, Arch platform, AuthFile, CertDir, ContainersConfModule, and GlobalArgs command-text observations; source
preservation does not model them.

The minimal Build subset recognizes repeatable `ImageTag`, `Network`, `Label`, `File`, `BuildArg`, `Secret`, `GroupAdd`, `DNS`, `DNSOption`, `DNSSearch`, `Annotation`, `Environment`, `ContainersConfModule`, `GlobalArgs`, and `PodmanArgs` values plus singleton
`SetWorkingDirectory`/`Target`/`Arch`/`Variant`/`Pull`/`Retry`/`RetryDelay`/`TLSVerify`/`ForceRM`/`AuthFile`/`IgnoreFile` values without interpreting image-reference grammar, platform grammar, pull policy, retry count or delay grammar, TLS or force-removal boolean grammar, auth-file path or ignore-file grammar, network modes or
options, paths, URLs, build contexts, or generated service precedence. Build `Annotation` retains raw physical lines without target tokenization, unquoting, C-unescaping, reset, duplicate-key collapse, sorting, OCI validation, or image-metadata inference. Build `Environment` likewise retains raw physical lines without target tokenization, unquoting, C-unescaping, reset, duplicate-name selection, sorting, or host lookup. Build `ContainersConfModule` likewise retains raw physical lines without path parsing, module reads, configuration inspection, reset, deduplication, tokenization, or normalization. An exact Build
`Network=name.network` is a document-set reference; other network text, including observed but
undocumented `.container` forms, remains opaque. Tagged source and all recorded Podman generators from
5.4.0 through 6.0.2 observe one final effective `File` command argument, but that does not alter
the lossless model or builder. A container `Image=name.build` is an exact document-set reference to
a typed `.build` unit. `Label` retains physical-line text without `KEY=VALUE` parsing, unquoting,
duplicate-name selection, map collapse or sorting, or validation. The matrix proves two ordered
`--tag` forms, three ordered Build `--network` forms with the `.network` dependency, and exactly
`--label build.label=one` and `--label empty=` without an ordering claim, plus the final `--file`
form, file-derived service working directory, and one `--target build-stage` form. It does not cover
bare labels, duplicate-label ordering or collapse, label grammar, image builds, or runtime behavior. BuildArg is
native from 5.7.0 through 6.0.2, where a separate fixture proves `key=value` and empty-value `key=` forms; it remains
opaque without assignment parsing or environment/secret resolution, and makes no bare/null claim.
Build `Secret` is native only from 5.4.0 through 6.0.2 and stays opaque: it does not parse commas,
arguments, environment forms, or paths, and never materializes secret data. Its isolated matrix
fixture proves two ordered separate placeholder-source `--secret` arguments only. Build `Arch` and
`Variant` are opaque singleton values over the same finite range. Their isolated matrix fixture proves
exactly one `--arch arm64` and one `--variant v8` without an assertion about relative argument order;
it does not select platform defaults, parse platform grammar, build an image, or inspect runtime metadata.
Build `Pull` is native only from 5.4.0 through 6.0.2 and unknown outside that range. Its isolated
fixture proves exactly one `--pull=always` argument; blank-value omission is source evidence only,
and the model/builder neither validate policies nor infer Compose boolean, registry, image-pull, or runtime behavior.
Build `Retry` and `RetryDelay` are unsupported from 5.4.0 through 5.4.2, native from 5.5.0 through
6.0.2, and unknown outside those ranges. Their isolated supported-range fixture proves exactly one
separate `--retry 4` pair and one separate `--retry-delay 7s` pair before the final `.` context
with no relative-order claim between pairs.
The opaque model and builder do not parse integers or durations, select defaults, apply
effective-last behavior, link Compose `dockerfile_inline`, access a registry, execute retries or
timing, establish build success, inspect runtime behavior, or define conversion behavior.
Build `TLSVerify` is native from 5.4.0 through 6.0.2 and unknown outside that range. Its isolated
two-unit fixture proves exactly one bare `--tls-verify` for `true` and exactly one
`--tls-verify=false` for `false`, each before its final `.` context. The opaque model and builder do
not parse booleans or select defaults, and the evidence does not establish TLS connectivity,
certificate validation, registry configuration, image pull, build success, security posture,
provenance equivalence, runtime behavior, or conversion behavior.
Build `ForceRM` is native from 5.4.0 through 6.0.2 and unknown outside that range. Its isolated
two-unit fixture proves exactly one bare `--force-rm` for `true` and exactly one
`--force-rm=false` for `false`, each before its final `.` context. The opaque model and builder do
not parse booleans, select defaults, or apply effective-last behavior, and the evidence does not
establish cleanup occurrence, failure behavior, execution, defaults or configuration, cache
equivalence, runtime behavior, or conversion behavior.
Build `GroupAdd` is native from 5.4.0 through 6.0.2 and unknown outside that range. Its isolated
fixture proves ordered separate `--group-add 1234` then `--group-add 5678` pairs before its final
`.` context, without a relative-order claim against map-derived flags. The opaque model and builder retain authored physical lines in source order without
group lookup, keep-groups exclusivity, rootless or user-namespace interpretation, runtime behavior,
build execution, Compose privilege equivalence, or conversion behavior.
Build `DNS` is native from 5.4.0 through 6.0.2 and unknown outside that range. Its isolated fixture
proves ordered separate `--dns 9.9.9.9` then `--dns 2001:4860:4860::8888` pairs before its final
`.` context, without a relative-order claim against map-derived flags. The opaque model and builder
retain authored physical lines in source order without resolver behavior, `none` compatibility,
`resolv.conf` or host-DNS inspection, build execution, Compose endpoint mapping, or conversion
behavior.

Build `DNSSearch` is native from 5.4.0 through 6.0.2 and unknown outside that range. Its isolated
fixture proves ordered separate `--dns-search corp.example` then `--dns-search .` pairs before its
final `.` context, without a relative-order claim against map-derived flags. The opaque model and
builder retain physical lines without reset or dot semantics, domain removal, DNS or resolver work,
network, build, Compose mapping, or conversion behavior.

Build `AuthFile` is native from 5.4.0 through 6.0.2 and unknown outside that range. Its isolated
fixture proves one separate `--authfile PATH` pair, generator-effective-last output for repeated
entries, and final-empty omission only. QuadletLens retains opaque singleton physical lines with
ordinary duplicate diagnostics; it does not normalize last/empty behavior, read or validate paths,
parse credentials, classify content sensitivity, authenticate, establish build success, or claim
runtime, Compose, or conversion behavior.

Build `IgnoreFile` is unsupported from 5.4.0 through 5.6.2, native from 5.7.0 through 6.0.2,
and unknown outside those reviewed ranges. Its isolated fixture proves one separate `--ignorefile
PATH` pair, generator-effective-last repeated output, and final-empty omission only. QuadletLens
retains opaque singleton physical lines with ordinary duplicate diagnostics; it does not normalize
last/empty behavior, resolve or read paths, parse ignore files, infer `.containerignore` or
`.dockerignore` defaults, normalize relative paths, establish build success, or claim runtime,
Compose, or conversion behavior.
Build `PodmanArgs` has finite native evidence from 5.4.0 through 6.0.2. Its all-20 fixtures prove only

Build `Annotation` is native from 5.4.0 through 6.0.2 and unknown outside that range. Its isolated
fixture proves the generator's empty reset, tokenization/unquoting/C-unescaping, final duplicate-key
selection, sorted map output, and the recorded 5.6.0 bare/malformed-token boundary only. QuadletLens
retains raw repeatable physical lines and does not apply any of those target semantics or claim OCI,
image metadata, build success, runtime, Compose, or conversion behavior.

Build `Environment` is native from 5.4.0 through 6.0.2 and unknown outside that range. Its
isolated fixture proves target reset, tokenization/unquoting/C-unescaping, final-name selection,
sorted output, and the 5.6.0 bare/malformed-token representation boundary only. QuadletLens retains
raw repeatable physical lines without applying those target rules, host lookup, build success,
runtime, Compose, or conversion behavior.

Build `ContainersConfModule` is native from 5.4.0 through 6.0.2 and unknown outside that range.
Its isolated fixture proves only target logical reset and ordered `--module=post-one` then
`--module=post-two` arguments before `build` and the final context. QuadletLens preserves raw
repeatable physical lines without applying those target rules or claiming module-path resolution,
module reads, configuration effects, build success, runtime, Compose, or conversion behavior.

one separate `--build-context extra=container-image://alpine:3.15`, exact `--no-cache`, or equals-form
`--isolation=chroot`/`--ssh=default`/`--shm-size=32m`/`--ulimit=nproc=4096:8192`/`--add-host=buildhost:192.0.2.10`/`--cap-add=CAP_SYS_ADMIN` argument immediately before final positional `.`; each rejects alternate, quoted,
duplicate, and reordered forms (the isolation, SSH, and shared-memory fixtures also reject the separate spelling). The isolation
capability proves command text only: it does not lower Compose, establish mode equivalence/defaults, or claim rootless/rootful, namespace, LSM, environment, build, runtime, or cross-format behavior. The repeatable `--no-cache` capability is command-text evidence only: it does not
lower Compose `no_cache`, interpret false, string, or interpolation values, establish cache semantic
equivalence, or claim execution, cache, image, runtime, or cross-format behavior. The non-secret SSH fixture does not provide, resolve, inspect, or claim keys, sockets, an agent, PEM data, paths, environments, mounts, builds, runtime state, or Compose lowering. The shared-memory fixture adds no native Build `ShmSize` key and does not establish Compose or unit equivalence, zero or omission defaults, IPC selection, host/cgroup/memory behavior, build execution, runtime behavior, or conversion behavior. The ulimit fixture adds no native Build `Ulimit` key and establishes no Compose name, range, or `-1` equivalence; host/rootless/rootful, `RUN`, cgroup, default, build, runtime resource-limit enforcement, or conversion behavior. The add-host fixture does not lower Compose list or map `extra_hosts` forms; establish IPv6 or `host-gateway` equivalence; alter DNS or `/etc/hosts`; resolve conflicts or defaults; execute a build; or establish runtime or conversion behavior. The cap-add fixture does not establish Compose entitlement equivalence or conversion; actual capability grants; build execution; LSM, seccomp, rootless, or runtime effects.

The container-identity subset includes singleton `ContainerName`. It is documented at the Podman
5.4 floor and verified as an exact `--name` generator argument through 6.0.2. The value is not
derived from the unit basename, checked for host collisions, or treated as a systemd unit name.

The process subset includes singleton `Entrypoint`. QuadletLens retains the exact executable or
JSON command-array text instead of decoding systemd/JSON quoting. The generator matrix verifies
that every supported Podman release passes the JSON array to `podman run`; it records the exact
presentation boundary from a separate argument through 5.8.1 to `--entrypoint=...` from 5.8.2.
The same process subset includes singleton `RunInit`. Authored omission remains absent, explicit
`true` and `false` remain distinct model values, and raw noncanonical one-line values are preserved
without boolean interpretation. For every supported patch release, the generator matrix proves
that `RunInit=true` emits exactly one `--init` argument and `RunInit=false` emits exactly one
`--init=false`. It does not establish runtime init behavior.

The generic repeatable `PodmanArgs` escape hatch has evidence-scoped exact
`PodmanArgs=--interactive`, `PodmanArgs=--tty`, `PodmanArgs=--privileged`, and
`PodmanArgs=--privileged=false` forms, without adding a `Tty` or `Privileged` key or wrapper. Their
isolated full generator fixtures require the respective separate argument immediately before the
image from every recorded Podman 5.4.0 through 6.0.2 release; support is unknown outside that
finite range. The privileged two-unit fixture rejects `--privileged=true`, positional false,
short, quoted, bundled, alternate, duplicate, and conflicting spellings. Its endpoint Quadlet
manual, tagged generator placement, and Podman CLI boolean/default evidence establish generated
command text only, not runtime privileges, devices, LSM, seccomp, rootless, or cross-format
equivalence.

The container stop-lifecycle subset includes singleton `StopSignal` and `StopTimeout`. The native
model retains exact authored one-line values, including zero, while the capability catalogue
records named/numeric signals and non-negative integer seconds as evidenced supported caller forms.
The generator matrix observes `--stop-signal SIGUSR1`, `--stop-signal 9`, a positive
`--stop-timeout 37`, and `--stop-timeout 0` across the supported range. It does not semantically
validate other raw values, start containers, measure elapsed stop time, establish whether zero
sends a signal, or assert equivalence with another format's lifecycle defaults.

The image-acquisition subset includes singleton `Pull`. Omission remains absent and exact one-line
values stay uninterpreted. Capability and generator evidence cover `always`, `missing`, `never`,
and `newer` as matching `--pull` arguments, without contacting a registry or inspecting local
image storage.

The resource subset includes singleton `PidsLimit`. Omission, authored zero, and noncanonical
one-line values remain distinct raw model and builder values. `PidsLimit::unlimited()` and
`PidsLimit::finite` provide safe construction for `-1` or nonzero ASCII-decimal spellings and
reject empty, nondecimal, or all-zero text. They retain leading zeros and arbitrary-precision
digits without parsing or overflow. Capability and generator evidence cover isolated
`--pids-limit 127` and `--pids-limit -1` output across Podman 5.4.0 through 6.0.2. They do not
establish a portable numeric maximum, cover zero, start a container, inspect its cgroup, or
establish process-exhaustion behavior.

The shared-memory subset includes separate singleton container and pod `ShmSize` keys. Parsed and
raw builder values remain exact and opaque. `ShmSize::new` accepts only a non-negative
ASCII-decimal amount with optional lowercase `b`, `k`, `m`, or `g`, while retaining leading zeros
and arbitrary-precision text without parsing. `ShmSize::unlimited()` emits the documented explicit
zero value, distinguishable from omission and Podman's documented `64m` default. The generator
matrix proves one matching `--shm-size 67108864b`, `--shm-size 0`, and pod-owned `--shm-size 32m`
argument, with no duplicate in the joined container. It does not start workloads, inspect shared
IPC or `/dev/shm`, exercise host IPC, establish runtime enforcement, or make rootless claims.

The capability-security subset includes repeatable container `DropCapability` and `AddCapability`.
Omission, empty native reset assignments, repeated entries, authored order, and exact one-line
values remain distinct. The model and builder do not
split space-separated lists, deduplicate capability names, lowercase source text, or validate a
native capability whitelist. Podman 5.4 documentation defines both repeatable space-separated lists
and documents lowercase `all` only for drops. The complete generator matrix observes four ordered
lowercase arguments for each isolated fixture, with no opposite capability form, plus exactly one
drop-all before one specific addition in a combined fixture. Tagged 5.4.0 and 6.0.2 source records
empty resets, lowercasing, drop-before-add construction, and the special merger semantics of
`all`; that special addition behavior is not attributed to the Quadlet prose. These are
definition, source, and generator-output observations, not claims about rootless/rootful
execution, effective bounding sets, user namespaces, SELinux/seccomp interaction, or runtime
privilege outcomes.

The temporary-filesystem subset includes repeatable container `Tmpfs`. Omission, empty reset
assignments, duplicates, order, case, and exact `CONTAINER-DIR[:OPTIONS]` spelling remain distinct
opaque values. QuadletLens does not split, normalize, deduplicate, validate target mount options,
or conflate `Tmpfs` with the separate `Volume` grammar. Podman's Quadlet documentation establishes
the repeatable mapping; separate Podman CLI documentation records Linux mount flags and the
`rw,noexec,nosuid,nodev` omission default. Tagged source and the complete generator matrix prove
that `LookupAll` leaves exactly one final post-reset
`--tmpfs /data:mode=755,uid=1009,gid=1009` command form. They do not start a container, create or
inspect a mount, enforce defaults, exercise copy-up, or establish rootless/runtime behavior.
There is no pod `Tmpfs` typed key or capability in this slice.

The kernel-parameter subset includes repeatable container `Sysctl`. Omission, empty resets,
duplicates, ordering, case, whitespace, systemd quoting/specifiers, and exact one-line values are
preserved without parsing `name=value` assignments, splitting lists, normalization, or namespace
validation. Endpoint manuals and tagged source establish native spelling, `LookupAllStrv`
tokenization, command construction, and reset behavior. The complete generator matrix proves one
final post-reset `--sysctl net.ipv4.ip_forward=1`, with neither pre-reset setting nor any other
sysctl argument. It does not start a container or establish namespace state, rootless behavior,
kernel acceptance, runtime equivalence, or actual parameter effects. There is no pod `Sysctl`
typed key or capability; Compose and BoxFerry mapping are outside this native slice.

The resource-limit subset includes repeatable container `Ulimit`. Omission, empty resets,
duplicates, order, case, quotes/specifiers, and every exact one-line value are preserved without
splitting, unquoting, or validating `TYPE=SOFT[:HARD]`. Endpoint manuals, Podman-run grammar/default
caveats, and tagged source establish the native spelling and `LookupAll` command/reset path. The
complete generator matrix proves exactly two ordered final post-reset `--ulimit` arguments for
`nproc=4096:8192` and `stack=-1:-1`, with no pre-reset, duplicate, empty, or alternate form. It does
not execute a container or claim runtime enforcement, host inheritance, defaults, cgroups,
rootless behavior, or acceptance of unverified resource names. There is no pod `Ulimit` typed key
or capability; Compose and BoxFerry mapping are outside this native slice.

The host-device subset includes repeatable container `AddDevice`. Omission, every physical value,
empty resets, duplicates, order, case, quotes/specifiers, whitespace-token-containing lines, and a
leading `-` are preserved without splitting, unquoting, parsing, or validation. Endpoint manuals,
Podman-run caveats, and tagged source establish native spelling plus generator-only
`LookupAllStrv`, reset, conditional leading-minus, and command construction behavior. The complete
generator matrix proves exactly two ordered final post-reset `--device` arguments and exactly two
total, with neither pre-reset mapping nor empty or alternate form. The fixture deliberately uses no
leading `-`, accesses no device, and starts no workload. There is no pod `AddDevice` typed key or
capability; CDI, runtime access, rootless, SELinux, cgroup, device existence, symlink behavior,
Compose, and BoxFerry mapping remain outside this native slice.

The memory-limit subset includes singleton container `Memory`, introduced by Podman 5.5.0. Parsed
and raw builder values preserve omission, duplicates, empty assignments, quotes, specifiers, zero,
and noncanonical text without runtime interpretation. `Memory::new` constructs positive
arbitrary-precision ASCII-decimal amounts with no suffix or one lowercase `b`, `k`, `m`, or `g`,
retaining leading zeros without parsing. A separate fixture keeps the existing 5.4-compatible
matrix unchanged: the three 5.4.x generators reject or exclude `Memory`, while every one of the 17
patch releases from 5.5.0 through 6.0.2 emits exactly one final `--memory 16777216b`. This does not
establish cgroup enforcement, page rounding, swap interaction, host-memory availability, rootless
behavior, runtime inspection, or cross-format equivalence. Pod `Memory` remains unknown.

The reload subset includes opaque singleton `ReloadCmd` and `ReloadSignal`, introduced by Podman
5.5.0. Parser and builder values remain exact one-line source text; neither command arguments nor
signals are interpreted. The complete matrix records the 5.4.x rejection boundary, 5.5.x cidfile
`ExecReload` form, 5.6.0–6.0.2 generated-name form, final blank omission and malformed-command
tokenization, and
the upstream mutual-exclusion rejection. The Lens preserves every authored line and reports a
structured conflict without reproducing target selection or executing, inspecting, or reloading a
container.

Pod `ExitPolicy` is an opaque singleton introduced by Podman 5.6.0. The parser and builder retain
exact one-line source text and duplicate diagnostics without interpreting `continue` or `stop`.
The complete matrix records rejection through 5.5.2, then one post-`--replace` `--exit-policy`
argument for both documented values, final-duplicate selection, and an empty argument for a final
blank assignment. It does not create a pod or establish default, restart, runtime, or cross-format
semantics.

Pod `StopTimeout` is an opaque singleton introduced by Podman 5.7.0. The parser and builder retain
exact one-line source text and duplicate diagnostics without interpreting seconds or `-1`. The
complete matrix records rejection through 5.6.2, then exactly one final `--time=` form for 37, 0,
-1, and duplicate-final-37 values plus `--time=` for a final blank assignment. It does not stop a
pod or establish default, timing, systemd, restart, runtime, or cross-format semantics.

The container-logging subset includes opaque singleton `LogDriver` and opaque repeatable `LogOpt`.
The source-aware model preserves omission, every physical value, duplicate singleton assignments
with their standard diagnostic, option duplicates and order, empty option resets, quotes, and
systemd specifiers. It does not parse options as key/value maps, validate drivers/options, or
inject defaults. Endpoint manuals and tagged 5.4.0/6.0.2 source establish native command,
singleton-lookup, ordered tokenization, and reset construction. The isolated complete-matrix
fixture proves exactly one `--log-driver k8s-file` plus two ordered final post-reset `--log-opt`
arguments. It starts no workload, reads no logs, and makes no host-policy, default, runtime, or
cross-format claim. Other native scopes remain unknown and preserved.

The container network-identity subset adds opaque singleton `IP` and `IP6` plus opaque repeatable,
resettable `NetworkAlias`. Omission, physical values, duplicate singletons and their diagnostics,
alias duplicates and order, empty resets, quotes, specifiers, and continuations remain preserved.
All 20 generators emit one `--ip`, one `--ip6`, one `--network bridge`, and two ordered final
post-reset aliases in the isolated fixture. Address, IPAM, DNS, network name/option, runtime, and
cross-format validation remain outside this native boundary, as does map-dependent relative flag
ordering.

The network subset adds opaque singleton `Driver` and opaque repeatable/resettable `Options`.
Every physical authored entry stays source-aware, including resets, duplicate keys, and bare
tokens. The complete generator matrix proves driver construction, reset behavior, duplicate-key
collapse, and key sorting, while preserving the observed 5.4.0 bare-token drop versus 6.0.2 bare
token emission as generator evidence rather than native-model behavior. Driver availability,
provider-specific option semantics, runtime network creation, and BoxFerry policy remain outside
this slice.

The network-IPAM subset adds opaque singleton `IPAMDriver` and opaque repeatable/resettable
`Subnet`, `Gateway`, and `IPRange`. Every physical entry remains source-aware, including resets,
duplicates, quotes, specifiers, continuations, and authored inter-key order. The complete matrix
proves explicit/blank driver behavior and two ordered final indexed subnet/gateway/range groups.
It does not validate IPAM drivers/defaults, address/range grammar, network creation, provider
behavior, Compose `aux_addresses` or IPAM-options equivalence, IPv4-disable inference, automatic
IPv6 inference, runtime behavior, or BoxFerry-owned prefix-complete mapping policy.

Network `Label` is also opaque, repeatable, and resettable from 5.4.0 through 6.0.2. Its source
entries retain empties, duplicates, bare values, embedded equals signs, quotes, specifiers,
continuations, and source order. The complete generator matrix observes effective reset,
duplicate collapse, key sorting, explicit empty and embedded-equals forms, quoted whitespace, and
the bare-token boundary; none of those generator rules alters the native model or builder.

Volume `Label` is likewise opaque, repeatable, and resettable from 5.4.0 through 6.0.2. Its source
entries retain empties, duplicates, bare values, embedded equals signs, quotes, specifiers,
continuations, and source order. The complete generator matrix observes reset, duplicate collapse,
key sorting, explicit empty and embedded-equals forms, quoted whitespace, and the bare-token
boundary without applying any of those generator rules to the model or builder.

Volume `ContainersConfModule` is opaque, repeatable physical-line text, native from 5.4.0 through
6.0.2 and `unknown` outside that range. Its full matrix records target empty reset, continuation
presentation, and ordered post-reset `--module` arguments before `volume create` only. QuadletLens
keeps every source value ordered and does not parse paths, read modules or configuration, apply
target reset behavior, infer sensitivity, validate options, create a volume, or claim filesystem,
lifecycle, security, runtime, Compose, or conversion behavior.

Volume `GlobalArgs` is opaque, repeatable physical-line text, native from 5.4.0 through 6.0.2 and
`unknown` outside that range. Its full matrix records only target empty reset,
tokenization/unquoting/C-unescaping, malformed-line omission, and ordered post-reset tokens before
`volume create`. QuadletLens preserves every source line without applying those rules, parsing or
validating arguments, inferring sensitivity, creating a volume, or claiming lifecycle, filesystem,
runtime, Compose, or conversion behavior.

Volume `PodmanArgs` is opaque, repeatable physical-line text, native from 5.4.0 through 6.0.2 and
`unknown` outside that range. Its full matrix records only target empty reset,
tokenization/unquoting/C-unescaping, malformed-line omission, and ordered terminal tokens before
the volume name. QuadletLens preserves every source line without applying those rules, parsing a
CLI, assigning dedicated-key behavior, inferring sensitivity, creating a volume, or claiming
lifecycle, filesystem, systemd, runtime, Compose, or conversion behavior.

Volume `User` is opaque singleton physical-line text, native from 5.4.0 through 6.0.2 and unknown
outside that range. The all-20 fixture observes only unambiguous `User=123` becoming `o=uid=123`
before the volume name; no UID/name parsing, ownership, mount, filesystem, runtime, Compose, or
conversion behavior is claimed.

Volume `Group` is opaque singleton physical-line text, native from 5.4.0 through 6.0.2 and unknown
outside that range. The all-20 fixture observes only unambiguous `Group=456` becoming `o=gid=456`
before the volume name; no GID/name parsing, account lookup, ownership, mount, filesystem, runtime,
Compose, or conversion behavior is claimed.

Volume `GID` is opaque singleton physical-line text, unsupported through 5.8.5 and native only
from 6.0.0 through 6.0.2. The 6.0.x fixture observes exactly one `--gid 5678` before the terminal
volume name; it records command text without interpreting the authored value.

Volume `ServiceName` is opaque singleton physical-line text, native from 5.4.0 through 6.0.2 and
unknown outside that range. The matrix records target generated-unit naming observations only.

The current promotion adds these container-only typed capabilities:

| Keys                                                            | Cardinality | Reviewed native range                     |
| --------------------------------------------------------------- | ----------- | ----------------------------------------- |
| `DNS`, `DNSOption`, `DNSSearch`, `ExposeHostPort`, `Annotation` | Repeatable  | 5.4.0–6.0.2                               |
| `IP`, `IP6`                                                     | Singleton   | 5.4.0–6.0.2                               |
| `NetworkAlias`                                                  | Repeatable  | 5.4.0–6.0.2                               |
| `AppArmor`                                                      | Singleton   | 5.8.0–6.0.2; unsupported through 5.7.1    |
| `NoNewPrivileges`, `SeccompProfile`, `SecurityLabel*`           | Singleton   | 5.4.0–6.0.2                               |
| `Mask`, `Unmask`                                                | Repeatable  | 5.4.0–6.0.2; earlier introduction unknown |

All retain opaque physical values and standard cardinality diagnostics. The complete generator
matrix verifies version support, ordering, and reset effects without claiming resolver, OCI,
security-policy, filesystem, runtime, or cross-format behavior. Other sections and unit types
remain unknown and preserved.

Cross-format selection remains BoxFerry-owned. A future exact Compose mapping is intentionally
bounded to a positive explicitly byte-qualified value on a separate container with private IPC;
pod lifting, host IPC, implicit defaults, zero equivalence, and runtime inspection require separate
policy and evidence.

The container-network-identity subset includes singleton `HostName`. Its exact one-line value is
opaque and omission remains omission; QuadletLens neither applies Compose RFC-1123 validation nor
normalizes native spelling. Podman documents that the key requires a private UTS namespace. The
isolated generator fixture relies on the default private UTS namespace and verifies exactly one
logical `--hostname app.example` argument across Podman 5.4.0 through 6.0.2. When a container joins
a pod with the default shared UTS namespace, the pod hostname wins. The fixture does not start a
container, inspect its runtime hostname or namespace, or prove pod precedence.

The reviewed systemd Unit relationship subset includes:

- `Notify=healthy` to delay service readiness until Podman reports a healthy container;
- `Requires=`, `Wants=`, and `Requisite=` activation dependencies;
- `BindsTo=`, `PartOf=`, `Upholds=`, and `Conflicts=` lifecycle relationships; and
- `After=` and `Before=` ordering.

The dependency graph applies per-key empty resets and systemd-style whitespace/quote tokenization,
records the relationship key on each reference and edge, and ignores ordinary `.service` and
`.target` names. Podman 5.4 literal behavior, the 5.5 rewrite boundary, current suffix mappings,
and missing-source failure are dry-run generator evidence. `Upholds=` requires systemd 249 or
newer; the catalogue records that limitation but cannot evaluate the host systemd version.

The parser still retains every generic systemd directive without forcing it into a closed enum.
Typed systemd keys are a programmatic-generation aid, not a complete systemd semantic model.
Runtime activation, failed-unit propagation, cycles, stop ordering, and restart propagation remain
outside current generator evidence and require separate systemd-aware validation.

The complete current documented Quadlet key inventory is typed. Further promotion is driven by
upstream additions or concrete value-grammar and runtime contracts.

## Promotion checklist

A key or unit type becomes supported only with:

1. parser classification and deterministic rendering;
2. builder cardinality and section validation;
3. data-driven minimum/maximum version capability records;
4. exact documentation or source evidence;
5. real-generator fixtures across the claimed support range; and
6. public API and limitation documentation.
