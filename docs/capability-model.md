# Capability model

## Purpose

Quadlet evolves with Podman and relies on systemd behavior. The catalogue gives the parser, validator, renderer, and downstream tools a shared evidence-backed answer about target compatibility.

It does not attempt to catalogue every Podman or systemd feature. Its scope is the Quadlet document contract and documented fallbacks.

## Capability identity

Capabilities use stable, namespaced identifiers, for example:

```text
quadlet.unit-type.container
quadlet.container.example-key
quadlet.container.example-key.value-form
quadlet.reference.template-instance
systemd.specifier.home-directory
```

Identifiers describe semantics rather than Rust type names.

## Capability record

A record can include:

- stable identifier and description
- applicable unit types and sections
- introduced version
- changed versions
- deprecated version
- removed version
- evidenced and supported caller value forms
- repetition/reset semantics
- native support classification
- fallback kind and fallback range
- known broken patch ranges
- documentation and source evidence
- automated-test evidence and known gaps

Catalogue files are validated against a strict versioned TOML schema. Unknown fields, duplicate
identifiers, inverted or uncovered ranges, overlapping native/unsupported declarations, missing
evidence, and documentation-only claims without an explicit evidence gap are rejected. An
evidence-backed `unsupported` range is explicit data, distinct from `unknown` when evidence cannot
establish behavior.

`value_forms` describes the caller representations for which a support claim has evidence. It does
not imply that the syntax parser, typed model, or shared `EntryValue` builder validates that
key-specific grammar. Those layers preserve authored values and enforce their own structural or
physical-line boundaries unless a dedicated value type is documented separately.

## Support classifications

| Classification | Meaning                                                                  |
| -------------- | ------------------------------------------------------------------------ |
| `native`       | Target directly supports the capability.                                 |
| `fallback`     | A documented compatible representation exists, such as Podman arguments. |
| `deprecated`   | Accepted but discouraged for the target.                                 |
| `removed`      | Previously supported but unavailable in the target.                      |
| `unsupported`  | No supported representation exists.                                      |
| `unknown`      | Available evidence cannot establish behavior.                            |
| `broken`       | Advertised or accepted, but a known target range behaves incorrectly.    |

## Version ranges

The product support policy and catalogue evidence coverage are different ranges. Podman 5.4.0 is
the fixed minimum; the upper product target follows the newest stable Podman release. The finite
catalogue range expands only as documentation and generator evidence are reviewed. See the
[generator matrix](generator-matrix.md) and [ADR 0006](decisions/0006-rolling-support-window-and-generator-evidence.md).

A requested range contains:

```text
podmanMinimumVersion
podmanMaximumVersion  # optional
```

Validation succeeds only if the selected representation works throughout the range. A capability introduced after the minimum cannot be selected unless an earlier-compatible fallback covers the rest of the range.

When the maximum is omitted, evaluation extends through the newest catalogue version and reports
that later releases are untested assumptions. The built-in supported-range catalogue currently has
finite evidence coverage from Podman 5.4.0 through current Podman 6.0.2. Generator-proven
first-conversion capabilities span that range; capabilities not protected by the fixture remain
`unknown` above their narrower evidence boundary. A newer upstream release becomes a tracked target
before it becomes catalogue evidence.

The container stop-lifecycle capabilities use the conservative supported caller forms
`signal-token-or-number` and `non-negative-integer-seconds`. Their generator evidence records named
and numeric signals, a positive timeout, and zero. These catalogue forms are not parser or builder
validators, and the evidence does not establish a broader signal grammar, an undocumented timeout
maximum, runtime timing, whether zero sends a signal, or cross-format equivalence.

The container `RunInit` capability similarly limits its supported caller form to literal `true` or
`false`. That form does not add boolean parsing to the model or builder: omission stays absent and
all authored one-line values remain raw. The generator evidence covers only the observed CLI
difference—one `--init` for `true` and one `--init=false` for `false`—not runtime init behavior.

The container `Pull` capability limits its supported caller forms to `always`, `missing`, `never`,
and `newer`. The shared model and builder preserve any authored one-line value without semantic
validation. Generator evidence establishes matching `--pull` arguments, not registry or local
image-storage runtime behavior.

The container `PidsLimit` capability covers `-1` for unlimited and positive integers. The focused
construction helper accepts nonzero ASCII-decimal spelling and rejects empty, nondecimal, or
all-zero text. It preserves leading zeros and arbitrary-precision digits without parsing or
silently normalizing them. The supported-window evidence does not establish one portable numeric
maximum; a future target-specific bound belongs in target validation. This helper does not validate
parsed or raw `EntryValue` text: omission, authored zero, and noncanonical one-line values remain
preserved. Generator evidence covers one positive value and `-1` only; zero and runtime pids-cgroup
enforcement are not capability-evidenced.

The container `HostName` capability records the native hostname spelling without adding an
RFC-1123 or other semantic validator to the model or builder. The Quadlet documentation establishes
the `HostName`-to-`--hostname` mapping; the separate Podman run documentation requires a private UTS
namespace and records the private-container/shared-pod UTS defaults. The isolated generator fixture
relies on Podman's default private UTS namespace and proves exactly one emitted logical
`--hostname app.example` argument only. If a container joins a pod with the default shared UTS
namespace, the pod hostname wins; runtime hostname inspection and that precedence remain outside
generator evidence.

The Build `File` capability is repeatable and opaque from Podman 5.4.0 through 6.0.2. QuadletLens
retains every authored physical line, including duplicates, empty assignments, quoting, specifiers,
and order, without path or URL classification, Containerfile resolution, or normalization. Tagged
5.4.0 and 6.0.2 source observes that the generator selects one last effective value, and the full
20-release matrix emits one final `--file Containerfile.final` argument. That target behavior is
evidence only: it is not applied by the source-aware model or builder, and no build, context,
filesystem, registry, runtime, or cross-format behavior is claimed.

Build `Network` is repeatable and opaque from Podman 5.4.0 through 6.0.2. The endpoint manuals,
tagged source, and complete generator matrix record the ordered `host`, `none`, and `.network`
forms, including the generated dependency for the exact `.network` reference. QuadletLens itself
classifies only that exact lowercase suffix for document-set resolution; it does not parse modes or
options, normalize text, adopt observed-but-undocumented `.container` reference semantics, or claim
build-time or runtime networking behavior.

Build `Retry` and `RetryDelay` are opaque singleton values. Podman 5.4.0 through 5.4.2 rejects or
excludes their isolated fixture, so both capabilities are `unsupported` there; they are native from
5.5.0 through 6.0.2 and `unknown` outside that finite range. The complete supported 5.5.0-through-
6.0.2 matrix requires exactly one separate `--retry 4` pair and one separate `--retry-delay 7s`
pair before the final `.` build context, without asserting an order between those pairs. This does not parse retry
counts or delays, select defaults, model effective-last behavior, link Compose `dockerfile_inline`,
access a registry, execute retries or timing, establish build success, inspect runtime behavior, or
define conversion behavior.

Build `TLSVerify` is an opaque singleton from Podman 5.4.0 through 6.0.2 and `unknown` outside
that finite range. Its two-unit matrix fixture requires one bare `--tls-verify` for authored `true`
and one `--tls-verify=false` for authored `false`, each before the final `.` build context. This
does not add boolean parsing, defaults, or effective-last behavior, and does not establish TLS
connectivity, certificate validation, registry configuration, image pull, build success, security
posture, provenance equivalence, runtime behavior, or conversion behavior.

Build `ForceRM` is an opaque singleton from Podman 5.4.0 through 6.0.2 and `unknown` outside
that finite range. Its two-unit matrix fixture requires one bare `--force-rm` for authored `true`
and one `--force-rm=false` for authored `false`, each before the final `.` build context. This
does not add boolean parsing, defaults, or effective-last behavior, and does not establish cleanup
occurrence, failure behavior, execution, defaults or configuration, cache equivalence, runtime
behavior, or conversion behavior.

Build `GroupAdd` is repeatable opaque physical-line text from Podman 5.4.0 through 6.0.2 and
`unknown` outside that finite range. Its fixture requires ordered separate `--group-add 1234` then
`--group-add 5678` pairs before the final `.` build context, without a relative-order claim against
map-derived flags. This does not perform group lookup,
interpret keep-groups exclusivity, rootless or user-namespace behavior, runtime behavior, build
execution, Compose privilege equivalence, or conversion behavior.

Build `DNS` is repeatable opaque physical-line text from Podman 5.4.0 through 6.0.2 and `unknown`
outside that finite range. Its fixture requires ordered separate `--dns 9.9.9.9` then
`--dns 2001:4860:4860::8888` pairs before the final `.` build context, without a relative-order
claim against map-derived flags. This does not perform resolver work, establish `none`
compatibility, inspect `resolv.conf` or host DNS, execute a build, map Compose endpoints, or define
conversion behavior.

Build `DNSSearch` is repeatable opaque physical-line text from Podman 5.4.0 through 6.0.2 and
`unknown` outside that finite range. Its fixture requires ordered separate `--dns-search
corp.example` then `--dns-search .` pairs before the final `.` build context, without a
relative-order claim against map-derived flags. QuadletLens does not apply reset or special-dot
semantics, perform domain removal, resolve DNS, inspect resolver state, execute a build, map
Compose values, or define conversion behavior.

Build `AuthFile` is opaque singleton physical-line text from Podman 5.4.0 through 6.0.2 and
`unknown` outside that finite range. Its fixture proves one separate `--authfile PATH` pair, the
generator's effective-last result for repeated entries, and final-empty omission only. QuadletLens
does not normalize those results, validate or read paths, parse credentials, classify content or
path metadata as sensitive, authenticate to a registry, establish build success, or define runtime,
Compose, or conversion behavior.

Build `IgnoreFile` is opaque singleton physical-line text, unsupported from Podman 5.4.0 through
5.6.2, native from 5.7.0 through 6.0.2, and `unknown` outside those ranges. Its fixture proves one
separate `--ignorefile PATH` pair, generator-effective-last output for repeated entries, and
final-empty omission only. QuadletLens does not normalize those results, resolve or read paths,
parse ignore files, infer `.containerignore` or `.dockerignore` defaults, normalize relative paths,
establish build success, or define runtime, Compose, or conversion behavior.

Build `Annotation` is opaque repeatable physical-line text, native from Podman 5.4.0 through 6.0.2
and `unknown` outside that range. Its fixture records only the generator's reset, tokenization,
unquoting, C-unescaping, duplicate-key collapse, sorting, and 5.6.0 bare/malformed-token boundary.
QuadletLens preserves raw lines in source order without applying those target semantics or claiming
OCI annotation validity, image metadata, build success, runtime, Compose, or conversion behavior.

Build `Environment` is opaque repeatable physical-line text, native from Podman 5.4.0 through
6.0.2 and `unknown` outside that range. The 5.4–5.5 parser represents only assigned tokens, while
5.6+ retains bare tokens; the fixture records the target's reset, tokenization, unquoting,
C-unescaping, final-name selection, sorted effective map, and that representation boundary only.
QuadletLens preserves raw lines in source order without applying those target semantics, host
lookup, build success, runtime, Compose, or conversion behavior.

Build `ContainersConfModule` is opaque repeatable physical-line text, native from Podman 5.4.0
through 6.0.2 and `unknown` outside that range. Tagged base-command helpers and the full matrix
record only target logical lookup, empty reset, and ordered `--module=VALUE` placement before the
build subcommand. QuadletLens preserves every authored physical line without path parsing, module
reads, configuration inspection, reset, deduplication, tokenization, normalization, build success,
runtime, Compose, or conversion behavior.

Build `GlobalArgs` is opaque repeatable physical-line text, native from Podman 5.4.0 through
6.0.2 and `unknown` outside that range. The full matrix records only target empty-reset,
tokenization/unquoting/C-unescaping, malformed physical-line omission, and authored-order token
placement between `podman` and `build`. QuadletLens preserves the source without applying those
target rules or validating options, inferring semantic/security/runtime effects, building, running,
or defining Compose or conversion behavior.

Build `ServiceName` is opaque singleton physical-line text, native from Podman 5.4.0 through 6.0.2
and `unknown` outside that range. The Build manual is absent through 5.8.5 and appears from 6.0.0.
The full matrix records only target last-value selection, `.service` addition, ordinary/default and
template naming, plus the 5.7.0 template and 5.8.2 unmatched-quote lookup boundaries. QuadletLens
does not strip extensions, derive names, mutate document or dependency identity, operate systemd,
resolve collisions or escaping, build, run, or define Compose or conversion behavior.

Build `Volume` is opaque repeatable physical-line text, native from Podman 5.4.0 through 6.0.2 and
`unknown` outside that range. Its full matrix records target reset/continuation lookup, ordered `-v`
output, relative `.` resolution, and exact `.volume` substitution/dependency only. QuadletLens does
not parse mount grammar or options, access a filesystem, mutate generated identities, or establish
mount, build, runtime, lifecycle, image, systemd, security, Compose, or conversion behavior.

Volume `ContainersConfModule` is opaque repeatable physical-line text, native from Podman 5.4.0
through 6.0.2 and `unknown` outside that range. Tagged volume-command helpers and the full matrix
record only logical lookup, empty reset, continuation presentation, and ordered `--module=VALUE`
placement before `volume create`. QuadletLens preserves every authored physical line without path
parsing, module reads, configuration inspection, reset, deduplication, tokenization,
normalization, sensitivity inference, option validation, volume creation, lifecycle, filesystem,
security, runtime, Compose, or conversion behavior.

Volume `GlobalArgs` is opaque repeatable physical-line text, native from Podman 5.4.0 through
6.0.2 and `unknown` outside that range. Tagged base-command helpers and the full matrix record
only target empty reset, tokenization/unquoting/C-unescaping, malformed-line omission, and ordered
post-reset tokens before `volume create`. QuadletLens preserves every authored physical line
without applying those rules, parsing or validating arguments, inferring sensitivity, reading
modules or configuration, creating a volume, or establishing lifecycle, filesystem, security,
runtime, Compose, or conversion behavior.

Volume `PodmanArgs` is opaque repeatable physical-line text, native from Podman 5.4.0 through
6.0.2 and `unknown` outside that range. Tagged volume generation and its shared helper, plus the
full matrix, record only target empty reset, tokenization/unquoting/C-unescaping, malformed-line
omission, and ordered terminal tokens before the volume name. QuadletLens preserves every authored
physical line without applying those rules, parsing a CLI, assigning dedicated-key behavior,
inferring sensitivity, creating a volume, or establishing lifecycle, filesystem, systemd, security,
runtime, Compose, or conversion behavior.

Volume `User` is opaque singleton physical-line text, native from Podman 5.4.0 through 6.0.2 and
unknown outside that range. Boundary manuals describe a numeric UID or user name, while tagged
generator source numeric-converts with invalid/default behavior; that discrepancy is explicit
evidence, not Lens validation or provider behavior. The all-20 fixture uses only `User=123` and
observes `o=uid=123` before the volume name. It makes no UID/name grammar, ownership, mount,
filesystem, lifecycle, security, runtime, Compose, or conversion claim.

Volume `Group` is opaque singleton physical-line text, native from Podman 5.4.0 through 6.0.2 and
unknown outside that range. Boundary manuals describe a numeric GID or group name, while tagged
generator source numeric-converts with invalid/out-of-range/default behavior and constructs
`o=gid`; that discrepancy is evidence only, not Lens validation or provider behavior. The all-20
fixture uses only `Group=456` and observes `o=gid=456` before the volume name. It makes no GID/name
grammar, account lookup, ownership, mount, filesystem, lifecycle, security, runtime, Compose, or conversion claim.

Volume `UID` is opaque singleton physical-line text, unsupported from 5.4.0 through 5.8.5 and
native only from 6.0.0 through 6.0.2. The three 6.0.x dry-run generators emit exactly one
`--uid 1234` before the terminal volume name; this is command-text evidence only, with no UID
grammar, lookup, default, ownership, mount, filesystem, security, runtime, Compose, or conversion claim.

Volume `GID` is opaque singleton physical-line text, unsupported from 5.4.0 through 5.8.5 and
native only from 6.0.0 through 6.0.2. The three 6.0.x dry-run generators emit exactly one
`--gid 5678` before the terminal volume name; this is command-text evidence only and does not
interpret the authored value.

Volume `ServiceName` is opaque singleton physical-line text, native from 5.4.0 through 6.0.2 and
unknown outside that range. The all-20 fixture records target last-value, `.service`, ordinary,
template, and unmatched-quote naming observations without interpreting the authored value.

Pod `ServiceName` is opaque singleton physical-line text, native from Podman 5.4.0 through 6.0.2
and unknown outside that range. Its all-20-release fixture records only target generated-unit
default, duplicate-last, `.service`, template, unmatched-quote, final-blank, and extension-bearing
naming observations. QuadletLens retains source text without stripping or requiring extensions,
deriving names, normalizing an effective value, evaluating templates/specifiers, or assigning
document/dependency identity, systemd, restart, runtime, or cross-format semantics.

Volume `Image` is opaque singleton physical-line text, native from 5.4.0 through 6.0.2 and
unknown outside that range. Only exact lowercase `.image` and `.build` basenames become native
document-set references and resolve when their typed documents are present. The fixture records
target driver behavior separately.

Image `ImageTag` is opaque singleton physical-line text, native from 5.4.0 through 6.0.2 and
unknown outside that range. Its fixture records target-only resource-name substitution,
duplicate-last/final-blank behavior, generated dependencies, and quote presentation; QuadletLens
does not adopt any of those as model, builder, or graph behavior.

Image `ServiceName` is opaque singleton physical-line text, native from 5.4.0 through 6.0.2 and
unknown outside that range. Its fixture records target-only naming defaults, duplicate-last,
template, and quote boundaries; QuadletLens does not derive identity or expose those semantics.

Image `AllTags` is opaque singleton physical-line text, native from 5.4.0 through 6.0.2 and
unknown outside that range. Its fixture records target-only true/false, duplicate-last,
absent/blank, and 5.8.2 unmatched-quote command-text observations; QuadletLens does not parse
booleans, select defaults, construct pulls, or expose registry, runtime, or graph semantics.

Image `Arch` is opaque singleton physical-line text, native from 5.4.0 through 6.0.2 and unknown
outside that range. Its fixture records target-only normal, duplicate-last, blank-omission, and
5.8.2 unmatched-quote command presentation; QuadletLens does not parse platform grammar, select a
host default, construct pulls, or expose image metadata, storage, registry, runtime, or graph semantics.

Image `AuthFile` is opaque singleton physical-line text, native from 5.4.0 through 6.0.2 and unknown
outside that range. Its fixture records target-only normal, duplicate-last, blank-omission, and
5.8.2 unmatched-quote command presentation; QuadletLens does not validate or read paths, parse
credentials, infer sensitivity, authenticate to a registry, construct pulls, or expose runtime or graph semantics.

Image `CertDir` is opaque singleton physical-line text, native from 5.4.0 through 6.0.2 and unknown
outside that range. Its fixture records target-only normal, duplicate-last, blank-omission, and
5.8.2 unmatched-quote command presentation; QuadletLens does not validate or read paths or certificates,
select containers-certs.d defaults or remote-client policy, infer sensitivity, authenticate to a registry,
construct pulls, or expose runtime or graph semantics.

Image `ContainersConfModule` is repeatable opaque physical-line text, native from 5.4.0 through
6.0.2 and unknown outside that range. Its fixture records target-only empty-reset and ordered
post-reset `--module` arguments before image pull; QuadletLens preserves every authored physical
line without module or configuration reads, reset, tokenization, CLI validation, pull, runtime, or graph semantics.

Image `Creds` is opaque singleton physical-line text, native from 5.4.0 through 6.0.2 and unknown
outside that finite range. It preserves blanks, quotes, specifiers, continuations, and duplicate
authored entries with `QLM0004`; repository-owned debug output redacts only this key's authored
value and continuation segments, while explicit text access and rendering remain exact. The
placeholder-only fixture records target normal, duplicate-last, final-blank omission, and
unmatched-quote command text without a flag-order or quote-boundary claim. QuadletLens does not
split, parse, validate, read, default, authenticate, pull, or otherwise handle credentials.

Image `DecryptionKey` is opaque singleton physical-line text, native from 5.4.0 through 6.0.2 and
unknown outside that finite range. It preserves blanks, quotes, specifiers, continuations, and
duplicate authored entries with `QLM0004`; repository-owned debug output redacts its authored
value and continuation segments, while explicit text access and rendering remain exact. The
placeholder-only fixture records target normal, duplicate-last, final-blank omission, and
unmatched-quote command text without a flag-order or quote-boundary claim. QuadletLens does not
split key or passphrase text, validate or read files, decrypt, select defaults, authenticate, pull,
or otherwise handle key material.

Image `GlobalArgs` is opaque repeatable physical-line text, native from 5.4.0 through 6.0.2 and
unknown outside that finite range. Its full matrix records only target empty reset,
tokenization/unquoting/C-unescaping, malformed-line omission, and ordered post-reset tokens between
`podman` and `image pull`. QuadletLens preserves every source line without applying those rules,
parsing or validating arguments, inferring sensitivity, or claiming image-pull, runtime, Compose,
or conversion behavior.

Image `OS` is opaque singleton physical-line text, native from 5.4.0 through 6.0.2 and unknown
outside that finite range. Its full matrix records target effective-last lookup, final-blank
omission, and endpoint-specific unmatched-quote command presentation before the image name.
QuadletLens preserves every authored value and duplicate diagnostic without applying those rules,
parsing operating-system grammar, or claiming host/default, pull, runtime, Compose, or conversion behavior.

Build `Label` is repeatable and opaque from Podman 5.4.0 through 6.0.2. QuadletLens retains every
physical line and its order without parsing `KEY=VALUE`, unquoting, duplicate-name selection, map
collapse or sorting, or validation. The full matrix asserts only `build.label=one` and `empty=` as
separate `--label` arguments. It does not establish bare-label acceptance, duplicate-order or
collapse behavior, label grammar, build execution, runtime behavior, or cross-format behavior.

Build `Target` is an opaque singleton from Podman 5.4.0 through 6.0.2. Duplicate authored values
remain retained with the ordinary singleton diagnostic, while generated construction rejects a
second value. Endpoint manuals, tagged source, and the complete generator matrix record one
`--target build-stage` argument only; they do not validate a stage name, Containerfile, build, or
runtime behavior.

Build `BuildArg` is repeatable opaque physical-line text with native evidence from Podman 5.7.0
through 6.0.2. The exact 5.4.0–5.6.2 generator range rejects or excludes it. Tagged 5.7/5.8
manuals and generator source record the `--build-arg` mapping; the current manual confirms it,
while the versioned v6.0.2 manual omits the key as an explicit documentation gap. The isolated
fixture proves only `key=value` and explicit empty-value `key=` command forms. QuadletLens does not parse assignments,
resolve environments or secrets, or claim bare/null, build, runtime, or cross-format behavior.

Build `Secret` is repeatable opaque physical-line text with native evidence from Podman 5.4.0
through 6.0.2 and unknown support outside that finite range. QuadletLens retains every authored
line and order without splitting comma-separated text, parsing argument names, resolving
environment forms or paths, or materializing secret data. Tagged 5.4.0 and 6.0.2 generator source
uses repeated effective values to append `--secret`; the isolated 20-release fixture asserts only
two separate ordered placeholder-source arguments, with no bare, environment, build, runtime, or
cross-format claim.

Build `PodmanArgs` is repeatable opaque physical-line text with native evidence from Podman 5.4.0
through 6.0.2 and unknown support outside that finite range. Boundary manuals and tagged source
record appended build arguments; isolated all-20 fixtures prove only one separate
`--build-context extra=container-image://alpine:3.15`, exact `--no-cache`, or equals-form
`--isolation=chroot` argument immediately before final positional `.`. The isolation fixture rejects
separate, quoted, alternate, duplicate, and reordered forms; it does not lower Compose, establish
mode equivalence/defaults, or claim rootless/rootful, namespace, LSM, environment, build, or runtime behavior. A fourth fixture proves exactly one equals-form `--ssh=default` immediately before final positional `.` and rejects separate, quoted, alternate, duplicate, and reordered forms. It does not provide, resolve, inspect, or claim keys, sockets, an agent, PEM data, paths, environments, mounts, builds, runtime state, or Compose lowering. A fifth fixture proves exactly one equals-form `--shm-size=32m` immediately before final positional `.` and rejects separate, quoted, alternate, duplicate, and reordered forms. It adds no native Build `ShmSize` key and does not establish Compose or unit equivalence, zero or omission defaults, IPC selection, host/cgroup/memory behavior, build execution, runtime behavior, or conversion behavior. A sixth fixture proves one ordered terminal `--cache-from
registry.invalid/quadlet-lens/cache-from --cache-to registry.invalid/quadlet-lens/cache-to .`
chain. The exact-form cache capabilities are repeatable command-text evidence only, not a cache
policy or effective cache use. QuadletLens does not split or quote arguments, lower Compose
`additional_contexts`, `service:`, `no_cache`, or cache forms; parse descriptors or cache types;
resolve contexts, paths, environments, images, credentials, registries, or services; validate the
CLI; build; run; or claim cache, image, runtime, or cross-format behavior.

The exact Build `PodmanArgs=--sbom=syft` then
`PodmanArgs=--sbom-output=/tmp/quadlet-lens-sbom.json` pair is separately native only from Podman
5.4.0 through 6.0.2 and unknown outside that range. Its full generator matrix proves one ordered
terminal pair before final positional `.` and rejects missing output, quoted, alternate, duplicate,
and reordered forms. Endpoint Quadlet `PodmanArgs`, build `--sbom=PRESET`, build `--sbom-output`,
and tagged `LookupAllArgs` evidence establish forwarding only. It does not lower Compose; create a
file; download an image; run a scanner; establish SBOM content, PURLs, attestations, publishing,
provenance, build, runtime, security, or conversion behavior.

The exact Build `PodmanArgs=--ulimit=nproc=4096:8192` form is separately native only from
Podman 5.4.0 through 6.0.2 and unknown outside that range. Its full generator matrix proves one
equals-form immediately before final positional `.` and rejects separate, quoted, alternate,
duplicate, and reordered forms. Endpoint Quadlet `PodmanArgs`, build `--ulimit`, and tagged
`LookupAllArgs` evidence establish forwarding only. It adds no native Build `Ulimit` key and does
not establish Compose name, range, or `-1` equivalence; host/rootless/rootful, `RUN`, cgroup,
default, build, runtime resource-limit enforcement, or conversion behavior.

The exact Build `PodmanArgs=--add-host=buildhost:192.0.2.10` form is separately native only from
Podman 5.4.0 through 6.0.2 and unknown outside that range. Its full generator matrix proves one
equals-form immediately before final positional `.` and rejects separate, quoted, alternate,
duplicate, and reordered forms. Endpoint Quadlet `PodmanArgs`, build `--add-host`, and tagged
`LookupAllArgs` evidence establish forwarding only. It does not lower Compose list or map
`extra_hosts` forms; establish IPv6 or `host-gateway` equivalence; alter DNS or `/etc/hosts`;
resolve conflicts or defaults; execute a build; or establish runtime or conversion behavior.

The exact Build `PodmanArgs=--cap-add=CAP_SYS_ADMIN` form is separately native only from Podman
5.4.0 through 6.0.2 and unknown outside that range. Its full generator matrix proves one
equals-form immediately before final positional `.` and rejects separate, quoted, alternate,
duplicate, and reordered forms. Endpoint Quadlet `PodmanArgs`, build `--cap-add`, and tagged
`LookupAllArgs` evidence establish forwarding only. It does not establish Compose entitlement
equivalence or conversion; actual capability grants; build execution; LSM, seccomp, rootless, or
runtime effects.

Build `Arch` and `Variant` are opaque singletons with native evidence from Podman 5.4.0 through
6.0.2 and unknown support outside that finite range. Duplicate and blank physical entries remain
source-aware while programmatic construction rejects a second value. Endpoint manuals and tagged
source record singleton `--arch` and `--variant` construction. The isolated 20-release fixture
asserts exactly one `--arch arm64` and one `--variant v8` argument without a relative-order claim.
QuadletLens does not parse platform grammar, select host defaults, apply effective-last behavior,
build an image, inspect metadata, or claim runtime or cross-format behavior.

The generic repeatable container `PodmanArgs` escape hatch remains the sole public native API for
arguments that have no dedicated Quadlet key; there is no separate `Tty` or `Privileged` key or
wrapper. Its separately evidenced exact `PodmanArgs=--interactive`, `PodmanArgs=--tty`,
`PodmanArgs=--privileged`, and `PodmanArgs=--privileged=false` forms are native from Podman 5.4.0
through 6.0.2 and unknown outside that finite range. The two privileged forms each appear exactly
once as a separate argument immediately before their respective image in every recorded generator;
the assertion rejects `--privileged=true`, positional false, short, quoted, bundled, alternate,
duplicate, and conflicting forms. Boundary endpoint manuals, tagged generator placement, and
Podman CLI boolean/default documentation support the claim. These capabilities record generated
command text only; they do not claim runtime privileges, devices, LSM, seccomp, rootless, or
cross-format equivalence.

The separate container and pod `ShmSize` capabilities cover a non-negative ASCII-decimal amount
with either unitless bytes or one lowercase `b`, `k`, `m`, or `g` suffix. The focused constructor
retains leading zeros and arbitrary-precision spelling without integer parsing, while parsed and
raw `EntryValue` text remains opaque. Podman documents a `64m` default when the option is omitted,
zero as unlimited IPC memory, and a conflict with host IPC. Pods share IPC among their containers
by default, so pod `ShmSize` owns that shared context. Generator evidence establishes exactly one
matching explicit CLI argument in each isolated container or pod unit and no duplicate in the
joined container; it does not establish omission defaults, runtime enforcement, rootless behavior,
host-IPC conflict handling, namespace state, or `/dev/shm` contents.

The container `DropCapability` capability covers the documented repeatable, space-separated native
list and lowercase `all`. Parsed and programmatically constructed `EntryValue` text remains opaque:
QuadletLens does not split lists, deduplicate entries, lowercase authored values, or validate a
capability whitelist. The isolated generator fixture observes four ordered lowercase `--cap-drop`
arguments from three authored entries and no `--cap-add` argument across the finite supported range.
Tagged Podman 5.4.0 source separately shows drops being added before additions; neither that source
ordering nor generated command text proves rootless/rootful privilege behavior, an effective
bounding set, user-namespace effects, SELinux/seccomp interaction, or runtime privilege outcomes.

The container `AddCapability` capability likewise covers a documented repeatable,
space-separated native list of additions beyond Podman's default capability set. The model and
builder preserve omission, empty native reset assignments, duplicates, order, case, and exact raw
text without interpreting them. The Quadlet prose does not document `all`; tagged 5.4.0 and 6.0.2
source plus the complete generator matrix record its special handling, list splitting,
lowercasing, resets, and drop-before-add construction separately. The isolated fixture observes
four ordered lowercase `--cap-add` arguments and no drop, while a combined fixture observes one
drop-all before one specific addition. These remain source and generated-command observations,
not runtime privilege claims. See the [generator evidence](generator-matrix.md#what-the-container-test-does).

The container `Tmpfs` capability covers the Quadlet manual's repeatable
`CONTAINER-DIR[:OPTIONS]` mapping to Podman `--tmpfs`. Parsed and programmatically constructed
values remain opaque: omission, empty native reset assignments, duplicates, order, case, and exact
destination/options text are preserved without splitting, normalization, deduplication,
mount-option validation, or conflation with `Volume`. Separate Podman CLI documentation describes
the Linux default mount-flag option surface and `rw,noexec,nosuid,nodev` when options are omitted;
those target/runtime rules do not become parser validation. Tagged 5.4.0 and 6.0.2 source maps
`Tmpfs` through `LookupAll`, whose empty assignment clears earlier logical entries. Every recorded
generator confirms exactly one final post-reset `--tmpfs /data:mode=755,uid=1009,gid=1009` and no
pre-reset or extra tmpfs form. Source and generated command text do not prove target-only option
acceptance, mount creation, copy-up/default enforcement, rootless behavior, or runtime filesystem
properties.

The container `Sysctl` capability covers the endpoint Quadlet manuals' repeatable,
space-separated `name=value` list passed to Podman `--sysctl`. Parsed and programmatically
constructed entries remain opaque: omission, empty native resets, duplicates, order, case,
whitespace, systemd quoting/specifiers, and exact one-line text are preserved without parsing or
normalization. Tagged 5.4.0 and 6.0.2 source separately records `LookupAllStrv` command
construction, systemd-compatible whitespace/quote tokenization, and empty-assignment reset
behavior. Every recorded generator confirms exactly one final post-reset
`--sysctl net.ipv4.ip_forward=1`, neither pre-reset setting, and no other sysctl form. Podman-run
documentation limits accepted settings by their IPC/network namespace context; source and dry-run
output do not prove runtime namespace state, rootless behavior, kernel acceptance, runtime
equivalence, or actual sysctl effects.

The container `Ulimit` capability covers the endpoint Quadlet manuals' repeatable native key and
the Podman-run manual's `TYPE=SOFT[:HARD]` target grammar. Parsed and programmatically constructed
entries remain opaque: omission, empty native resets, duplicates, order, case, systemd
quoting/specifiers, and each exact one-line value are preserved without splitting, unquoting,
normalization, or validation. Tagged 5.4.0 and 6.0.2 source maps the key through the repeated-string
helper using `LookupAll`, not `LookupAllStrv`, and records empty-assignment reset behavior. Every
recorded generator confirms exactly two ordered final post-reset arguments,
`--ulimit nproc=4096:8192` and `--ulimit stack=-1:-1`, with no pre-reset, duplicate, empty, or
alternate form. The Podman-run documentation's default caveats are evidence context, not a
QuadletLens default claim; runtime enforcement, host inheritance, cgroups, rootless behavior, and
acceptance of unverified resource names remain outside this capability.

The container `AddDevice` capability covers the endpoint Quadlet manuals' repeatable
`HOST-DEVICE[:CONTAINER-DEVICE][:PERMISSIONS]` key and documented conditional leading-`-` form.
Parsed and programmatically constructed entries remain opaque: omission, empty native resets,
duplicates, order, case, systemd quoting/specifiers, whitespace-containing lines, leading `-`, and
each exact physical value are preserved without splitting, unquoting, parsing, normalization, or
validation. Tagged 5.4.0 and 6.0.2 source records `LookupAllStrv` tokenization, reset, conditional
leading-minus handling, and `--device` command construction separately. Every recorded generator
confirms exactly two ordered final post-reset arguments, `--device /dev/null:/dev/final-null:r`
and `--device /dev/zero:/dev/final-zero:w`, with no pre-reset, duplicate, empty, or alternate form.
The fixture deliberately avoids leading `-` and invokes no workload. Podman-run caveats bound the
evidence rather than establish QuadletLens claims: CDI, runtime access, rootless, SELinux, cgroup,
host-device existence, and symlink behavior remain outside this capability.

The container `Memory` capability begins at Podman 5.5.0, where the upstream introduction change
registers the singleton key and maps its last effective value to one `--memory` argument. Parsed
and raw builder values remain opaque, including empty, quoted, specifier-bearing, zero, and
noncanonical spellings. The focused constructor accepts only a positive ASCII-decimal amount with
no unit or one lowercase `b`, `k`, `m`, or `g`, preserving leading zeros and arbitrary precision
without integer parsing. Every recorded 5.5.0-through-6.0.2 generator emits exactly one final
`--memory 16777216b` argument for the separate fixture; the 5.4.x line rejects or excludes that
unsupported key and emits no memory argument. This establishes native recognition and dry-run
command construction only, not cgroup enforcement, page rounding, swap interaction, host-memory
availability, rootless behavior, runtime inspection, or Compose/BoxFerry equivalence.

The container logging capabilities cover opaque singleton `LogDriver` and repeatable `LogOpt`
from Podman 5.4.0 through 6.0.2. Parsed and constructed values remain physical-line-safe authored
text; `LogOpt` assignments may repeat and reset, but QuadletLens does not apply Podman's effective
tokenization to its model, parse options as key/value maps, validate drivers/options, or inject
defaults. Endpoint manuals and tagged 5.4.0/6.0.2 source establish the native flag mappings,
singleton lookup, `LookupAllStrv` order, and empty reset behavior. The isolated full matrix proves
one final `--log-driver k8s-file` and two ordered post-reset `--log-opt` arguments only. It does not
start a workload, inspect configuration or logs, or establish host-policy, default, runtime, or
cross-format behavior.

The container network-identity capabilities cover opaque singleton `IP` and `IP6` plus repeatable,
resettable `NetworkAlias` from Podman 5.4.0 through 6.0.2. The model and builder do not parse or
validate addresses or aliases. An isolated fixture with one `Network=bridge` proves one `--ip`,
one `--ip6`, one network selection, and two ordered final post-reset `--network-alias` arguments
across all 20 patches. It does not assert map-dependent relative flag ordering or establish IPAM,
IPv6 enablement, DNS, network-option, runtime, or cross-format behavior.

Network `Driver` is a singleton and `Options` is repeatable and resettable across Podman 5.4.0
through 6.0.2. The native model preserves every physical entry and does not validate driver
availability or provider options. The complete generator lane proves one `--driver bridge`, reset
handling, final duplicate-key collapse, and sorted retained options. It separately records that
5.4.0 drops the authored bare option token whereas 6.0.2 emits it. Those effective-generator
rules do not become model or builder semantics.

Network `Label` is an opaque, repeatable/resettable physical key across Podman 5.4.0 through
6.0.2. The endpoint manuals and tagged 5.4.0/6.0.2 source record mapping to `--label`, reset,
quote-aware tokenization, final duplicate-key selection, and sorted command emission. The full
matrix proves final labels, explicit `key=`, embedded `key=a=b`, and one quoted-whitespace
argument. Bare tokens are absent through 5.5.2 and emitted from 5.6.0 onward. QuadletLens keeps
all authored values without applying those generator rules; no network creation, inspection, or
runtime claim is made.

Network `IPAMDriver` is an opaque singleton; `Subnet`, `Gateway`, and `IPRange` are opaque,
repeatable/resettable physical keys across Podman 5.4.0–6.0.2. Endpoint manuals and tagged source
record `--ipam-driver` plus indexed subnet/gateway/range command construction. The full matrix
proves one explicit driver, blank-driver omission, and two ordered final reset-aware columns. It
does not validate IPAM-driver availability or defaults, address/range grammar, network creation,
runtime behavior, Compose `aux_addresses` or IPAM-options equivalence, IPv4-disable inference, or
automatic IPv6 inference. The generator's negative no-subnet and gateway/range-overrun paths have
only human-readable diagnostics, so they are recorded as tagged-source evidence rather than brittle
text-matching matrix assertions. Cross-format prefix-complete mapping policy belongs to BoxFerry;
QuadletLens preserves native syntax and validates target capability only.

Volume `Driver` is an opaque singleton across Podman 5.4.0–6.0.2. Endpoint evidence uses the
combined 5.4 manual and the split 6.0.2 `podman-volume.unit(5)` manual. Volume `Options` is a
separate opaque singleton raw mount-option string; it is observed in tagged source and absent from
those endpoint manuals. Tagged source and the full matrix record last-physical-value lookup, blank
omission, one `--opt o=<Options>` argument, the Device prerequisite through 5.8.5, its removal in
6.0.0, and the 5.8.2 unmatched-quote boundary. These are dry-run generator facts only: no
driver/plugin, mount, rootless, runtime, Compose `driver_opts`, or BoxFerry name/external/image
policy is claimed.

Volume `Device` and `Type` are separate opaque singletons across the same range. Endpoint manuals
define Device as the source and Type as its filesystem type; tagged parser evidence covers the
common last-physical-value lookup. All 20 generators emit final `device=` and `type=bind` options,
suppress final blanks, reject Type without Device, and retain quote, specifier, and continuation
inputs. For a bind Device containing a literal space, the additional generated `RequiresMountsFor`
line is absent through 5.5.2, unescaped from 5.6.0–5.7.1, and quoted with `\\x20` from 5.8.0–6.0.2.
These dry-run observations do not claim source availability, filesystem support, mount ordering,
runtime behavior, or Compose/BoxFerry equivalence.

Volume `Label` is an opaque, repeatable/resettable physical key across Podman 5.4.0 through
6.0.2. Endpoint manuals and tagged source record `--label`, reset, quote-aware tokenization,
final duplicate-key selection, and sorted command emission. The full matrix proves final labels,
explicit `key=`, embedded `key=a=b`, quoted whitespace, and the bare-token boundary; it also
records literal-space presentation in 5.4.x and `\\x20` from 5.5.0 onward. QuadletLens retains
every physical source entry and claims no volume creation, inspection, or runtime behavior.

Container `DNS`, `DNSOption`, and `DNSSearch` are native repeatable capabilities across
Podman 5.4.0–6.0.2. `ExposeHostPort` is native over the same range; tagged Quadlet accepts TCP/UDP
while the Podman CLI also documents SCTP, so `/sctp` remains preserved but not claimed as
generator-compatible. The full matrix verifies ordered post-reset command construction only.

Exact runtime detection can narrow validation to one version, but generated project files should normally declare their intended portable range.

`PodmanTarget` optionally carries a caller-supplied `SystemdVersion`; QuadletLens never probes the
host. The catalogue currently uses that context only for `systemd.unit.upholds`: no systemd target
is `Unknown`, releases below 249 are `Unsupported`, and 249 or newer retain the normal Podman
evaluation. Systemd requirements cite a separate typed `systemd_evidence` collection with
versioned source URLs and finite systemd release ranges; they do not reuse Podman evidence ranges.
Each declared minimum must reference evidence that covers that release. This is not a generic
systemd feature catalogue and does not account for distribution backports or overrides.
An in-coverage evaluation exposes the applicable systemd record identifiers through
`CapabilityEvaluation::systemd_evidence()`, including when missing or insufficient systemd context
changes the result. `CapabilityEvaluation::evidence()` remains Podman-only. Unknown,
out-of-coverage, and non-systemd evaluations expose no systemd evidence.

The security and metadata capabilities use the following reviewed ranges:

| Keys                                                  | Native range | Evidence summary                                                                 |
| ----------------------------------------------------- | ------------ | -------------------------------------------------------------------------------- |
| `Annotation`                                          | 5.4.0–6.0.2  | Repeatable reset and sorted final command arguments                              |
| `AppArmor`                                            | 5.8.0–6.0.2  | Explicitly unsupported through 5.7.1                                             |
| `NoNewPrivileges`, `SeccompProfile`, `SecurityLabel*` | 5.4.0–6.0.2  | Singleton/boolean command construction                                           |
| `Mask`, `Unmask`                                      | 5.4.0–6.0.2  | Repeatable reset/tokenization command construction; earlier introduction unknown |

Parsed and constructed values remain opaque. Generator fixtures do not validate profiles, OCI or
SELinux policy, paths, host state, runtime effects, or cross-format semantics.

## Patch releases and distribution backports

Feature introduction is usually tracked by Podman minor version, while known bugs may require patch-level ranges. Distribution packages may backport fixes or features without changing the upstream minor version.

QuadletLens does not currently apply distribution overrides. A future override contract requires a
concrete supported backport case, explicit caller selection, visible evaluation evidence, and a
separate architectural decision; it must never be inferred from the installed system.

## Evidence workflow

1. Compare tagged Podman documentation and release notes.
2. Inspect relevant tagged implementation behavior without copying implementation code.
3. Produce a candidate catalogue change.
4. Run fixtures against exact Podman generator versions.
5. Review semantic behavior, not only whether a key is accepted.
6. Record evidence, test result, and remaining uncertainty.

Generated diffs may assist this process, but a generated list of keys is not sufficient evidence of correct semantics.

Evidence records declare either `documentation` or `generator` verification and a finite exact
version or range. A generator range is valid only when every patch in it is executed.
Documentation-only records must name the missing generator evidence. A support result can therefore
be native according to primary documentation while still exposing the exact execution gap to
callers and maintainers.

## Fallbacks

Fallback records describe a semantic option, not a preassembled shell string. Rendering is responsible for safe argument construction and target syntax. A fallback must state which Podman versions support the underlying command behavior and what semantic differences remain.

## Volume `Copy`

Volume `Copy` is an opaque singleton with native evidence from Podman 5.4.0 through 6.0.2.
Endpoint manuals, tagged conversion/parser source, and the isolated dry-run matrix record omission,
accepted true spellings, invalid-as-`nocopy`, duplicate last-value selection, quote/continuation
handling, the 5.8.2 unmatched-quote boundary, and image-driver suppression. The model retains raw
physical text rather than coercing a boolean. Versions before the floor and after the ceiling remain
explicit gaps; no copy-up, volume creation, image pull, runtime, rootless, plugin, or cross-format
claim is made.
