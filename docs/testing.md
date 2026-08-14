# Testing strategy

QuadletLens requires both pure library tests and tests against real released generators. Documentation alone cannot establish parser and generated-command behavior across versions.

## Investment boundary

Pull-request checks must remain deterministic and reasonably fast. New behavior should normally
have focused positive and negative coverage, including malformed input, reset/repetition behavior,
public API use, or representative end-to-end behavior when relevant.

The project does not require fuzzing or 100% code coverage. Coverage floors detect broad
regressions; they do not replace meaningful assertions. Full generator matrices, privilege-specific
runs, and other environment-dependent checks remain opt-in or scheduled unless they are the only
practical way to protect a supported contract. See the [quality plan](quality-plan.md) for the
prioritized work.

## Test layers

### Syntax tests

Cover sections, ordered and repeated keys, comments, blank lines, continuations, quoting, resets, specifiers, malformed lines, Unicode, line endings, and source spans.

### Typed-model tests

Cover every supported unit type, section, key, short/long value form, reference type, unknown entry, and generic systemd section.

### Manual-inventory policy tests

Run fully offline in normal policy and MSRV lanes. They validate the strict versioned 222-key
inventory's provenance, spelling, uniqueness, normalized order, sections, and classifications;
parse every typed row through the public native model; and prove an untyped row remains losslessly
preserved. The small extraction fixture exercises the shell extractor without a network request.
Only the separate scheduled/manual workflow downloads the upstream aggregate manual and reports
key additions or removals. This phase does not claim prose or value-grammar drift detection.

### Programmatic-generation tests

Cover typed native-key placement, deterministic section order, repeated entries, singleton
rejection, unsafe physical values, generic systemd directives, parse-back validation, and complete
multi-document relationships for the first conversion subset.

### Round-trip and property tests

Verify that parsing never panics, canonical output is deterministic, preservation edits do not rewrite unrelated entries, and supported typed values survive parse-render-parse cycles.

### Capability-schema tests

Every catalogue entry must:

- conform to the schema
- have a unique stable identifier
- use a coherent version range
- link to evidence or state a documented evidence gap
- identify test coverage
- avoid contradictory native/fallback ranges

### Version-boundary tests

For every introduction, change, deprecation, removal, bug range, or fallback, test the nearest supported version below and above the boundary where available.

### Real-generator tests

Run fixtures through exact Podman system-generator versions from 5.4 through the newest supported release. Capture command, version, environment, exit status, generated service, and diagnostics. Separate syntactic acceptance from successful generated-command behavior.

The implemented container harness records every official immutable Podman patch image from 5.4.0
through 5.8.2 with its manifest digest. For 5.8.3 through current 6.0.2 it builds the standalone
generator from an exact release commit in a digest-pinned Go container. The smoke lane covers the
minimum, the official-image boundary, and current stable; its scheduled/manual full lane covers all
20 patches. It invokes only the dry-run generator. The fixture covers container and pod
relationships, ports, environment-file path forms, repeated container labels, execution identity
and context, networks, image and rootfs workload sources, volume and bind forms, explicit true and
false init selection, distinct container/pod user namespaces, named and numeric stop signals,
positive and zero stop timeouts, health modes, isolated positive and unlimited process-ID limits,
an isolated container hostname, positive and zero container shared-memory sizes, a pod-owned
shared-memory size with a joined container, restart behavior, continued raw arguments, and stable
generated dependencies. An isolated repeated `DropCapability` fixture requires exactly four
ordered lowercase `--cap-drop` forms and no `--cap-add` across all 20 patches. A parallel
`AddCapability` fixture requires exactly four ordered lowercase `--cap-add` forms and no drop; a
combined fixture requires exactly one drop-all before one specific addition and no other
capability argument. An isolated `Tmpfs` fixture authors two pre-reset entries, an empty reset, and
one final entry; every patch must emit exactly one logical final `--tmpfs` argument, no pre-reset
path, and no duplicate or alternative tmpfs form.

The isolated Image fixture records a literal pull unit, missing and empty source errors, and target
duplicate-last selection across all 20 releases. It does not pull an image or establish registry,
authentication, TLS, image-storage, systemd, runtime, or conversion behavior.

The container batch coverage proves `AutoUpdate` registry/local labels, all four documented
`CgroupsMode` values, `EnvironmentHost=false`, `ReadOnlyTmpfs`, `Timezone`, and
`HealthOnFailure`, plus ordered post-reset `Mount` output. The completion fixture independently
asserts post-reset ordered `ContainersConfModule`/`GlobalArgs` command construction, all promoted
health-log and health-startup argument pairs, and final `ServiceName` output naming across the
recorded range. `ImageVolume` intentionally has no generator claim. Separate direct-map and subordinate-map
fixtures prevent unsupported mapping combinations while proving ordered post-reset `UIDMap` and
`GIDMap` pairs and isolated `SubUIDMap`/`SubGIDMap` forms. The version-scoped retry fixture rejects
`Retry`/`RetryDelay` through 5.4.2 and requires exact pairs from 5.5.0; the proxy fixture rejects
`HttpProxy` through 5.6.2 and requires exactly `--http-proxy=false` from 5.7.0. The StartWithPod
fixture records `%t/batch-pod.pod-id` before 5.7.0 and `--pod systemd-batch` from 5.7.0. Model
tests separately include source-spanned diagnostics for effective mapping/UserNS/subordinate-map/
pod relationships and `StartWithPod`/`Pod` and `ReadOnlyTmpfs`/`ReadOnly` relationships. Those
two diagnostic checks recognize case-insensitive `1`/`yes`/`true`/`on` and
`0`/`no`/`false`/`off` forms only, after target-style trailing-whitespace removal and matched
double-quote lookup; blank resets and unknown or escaped spellings remain undiagnosed. No test
reads host environment, proxy, subordinate-ID, timezone, filesystem, registry, or runtime state.

The Pod completion fixture extends the isolated Pod ServiceName directory with post-reset
`ContainersConfModule`, `GlobalArgs`, DNS, DNS-option, DNS-search, label, and network-alias forms;
singleton hostname and IPv4/IPv6 forms; reset-aware final-only opaque `PodmanArgs`; and separate valid direct
and subordinate mapping documents. Every recorded generator must retain only the final values in
the expected order and must not combine direct with subordinate maps. It verifies generated command
construction, not module loading, argument semantics, resolver behavior, ID lookup, IPAM, network
creation, labels, pod creation, or runtime state.

An isolated `Sysctl` fixture likewise authors two pre-reset assignments, an empty reset, and one
final entry; every patch must emit exactly one final `--sysctl net.ipv4.ip_forward=1`, no other
sysctl form, and neither pre-reset setting. An isolated `Ulimit` fixture authors pre-reset `core`
and `nofile` entries, an empty reset, then `nproc=4096:8192` and `stack=-1:-1`; every patch must
emit exactly those two ordered final `--ulimit` arguments with no pre-reset, duplicate, empty, or
alternate form. An isolated `AddDevice` fixture authors one pre-reset line containing two mappings,
an empty reset, then one final line containing two mappings; every patch must emit exactly the two
ordered final `--device` arguments, exactly two total, and no pre-reset, empty, or alternate form.
It deliberately authors no leading `-`. All four isolated fixtures exercise generator command construction only and
never start a container. See the
[matrix documentation](generator-matrix.md).

A separate container-logging fixture authors one singleton `LogDriver`, two pre-reset `LogOpt`
entries, an empty reset, and two final options. Every selected patch must emit one
`--log-driver k8s-file` argument and exactly two ordered final post-reset `--log-opt` arguments,
with no pre-reset or alternate form. It validates command construction only and reads no logs.

A separate network-identity fixture authors singleton `IP` and `IP6`, one `Network=bridge`, two
pre-reset aliases, an empty reset, and two final aliases. Every selected patch must emit one
`--ip`, one `--ip6`, one network selection, and exactly two ordered final aliases without a
pre-reset or alternate form. The assertion deliberately ignores map-dependent relative ordering
between network selection and identity flags and makes no address, IPAM, DNS, network-option,
runtime, or cross-format claim.

A separate network-driver/options fixture authors singleton `Driver=bridge`, pre-reset options,
an empty reset, duplicate final `alpha` assignments, a later `zeta` assignment, and a bare token.
Every selected patch proves the final driver, reset, duplicate-key collapse, and sorted retained
options; assertions separately require 5.4.0 to drop the bare token and 6.0.2 to emit it. The
fixture does not validate a driver, provider option semantics, create a network, or inspect runtime
state.

A separate network-label fixture authors pre-reset labels, an empty reset, a final duplicate key,
explicit empty and embedded-equals values, a bare value, and quoted whitespace. Every selected
patch proves reset, last-wins duplicate collapse, sorted final keys, `key=`, `key=a=b`, and one
logical quoted-whitespace argument. Bare labels are absent through 5.5.2 and present exactly once
from 5.6.0 onward. It asserts generated command presentation only; it does not create or inspect
networks or claim runtime label behavior.

A separate volume-label fixture covers the same physical forms independently. Every selected patch
proves reset, final duplicate-key collapse, sorted labels, `key=`, `key=a=b`, and quoted
whitespace. Bare labels are absent through 5.5.2 and present exactly once from 5.6.0 onward; the
fixture records literal-space command presentation in 5.4.x and `\\x20` from 5.5.0 onward. It
does not create or inspect a volume or claim runtime label behavior.

A separate network-IPAM fixture authors singleton `IPAMDriver=host-local`, independently reset
pre-values for `Subnet`, `Gateway`, and `IPRange`, and two final columns. Every selected generator
must emit one explicit driver, exactly two ordered `--subnet`, `--gateway`, `--ip-range` groups,
and no pre-reset form; a companion blank-driver unit emits no `--ipam-driver`. Tagged source
documents the no-subnet and gateway/range-overrun rejections, but the harness does not match their
plain-text diagnostics because they are not a stable structured error interface.

A separate network-boolean fixture distinguishes omitted, literal true, and literal false
`Internal` and `IPv6` values. Every selected generator must emit no flag for omission, one plain
flag for true, and one explicit `=false` flag for false, without asserting relative flag order. It runs only the
generator; no driver selection, network creation, external isolation, dual-stack behavior, or
IPv4-enable key is claimed.

A separate volume-driver/options/device/type suite records four opaque singletons. Its all-range
fixture proves final Device/Type construction, final-empty omission, and quote/specifier/
continuation presentation; a negative fixture requires Type without Device to fail across all 20
releases. Type=bind records no additional Device-derived `RequiresMountsFor` through 5.5.2,
unescaped output through 5.7.1, and quoted `\\x20` output from 5.8.0. It asserts dry-run command
construction only, never source availability, filesystem support, driver/plugin availability,
mount creation, rootless behavior, runtime state, Compose `driver_opts`, or BoxFerry policy.

The promoted networking, annotation, and security fixtures cover ordered post-reset output,
singleton/boolean command construction, and unsupported-version behavior across the complete
20-patch matrix. AppArmor is rejected through 5.7.1 and accepted from 5.8.0. Model/render tests
separately protect opaque values, duplicates, malformed text, and scope.

The isolated Build retry fixture records the 5.4.x rejection boundary and supported 5.5.0-through-
6.0.2 command construction. It requires one separate `--retry 4` pair and one separate
`--retry-delay 7s` pair before the final `.` context without asserting an order between those pairs.
It does not parse count/duration text,
select defaults, model effective-last behavior, link Compose `dockerfile_inline`, contact a registry,
execute retries or timing, establish build success, start a workload, inspect runtime behavior, or
claim conversion behavior.

The isolated two-unit Build TLSVerify fixture covers all 20 recorded releases. It requires one bare
`--tls-verify` for authored `true` and one `--tls-verify=false` for authored `false`, each before
the final `.` context and without equals, quoted, alternate, duplicate, or post-context forms. It
does not parse booleans, select defaults, exercise TLS connectivity or certificate validation,
configure a registry, pull an image, complete a build, establish a security posture or provenance
equivalence, inspect runtime behavior, or claim conversion behavior.

The isolated two-unit Build ForceRM fixture covers all 20 recorded releases. It requires one bare
`--force-rm` for authored `true` and one `--force-rm=false` for authored `false`, each before the
final `.` context and without equals, quoted, alternate, duplicate, or post-context forms. It does
not parse booleans, select defaults, apply effective-last behavior, establish cleanup occurrence,
failure behavior, execution, defaults or configuration, cache equivalence, runtime behavior, or
claim conversion behavior.

The isolated Build GroupAdd fixture covers all 20 recorded releases. It requires ordered separate
`--group-add 1234` then `--group-add 5678` pairs before the final `.` context, rejecting equals,
quoted, merged, duplicate, reordered, and post-context forms without a relative-order claim
against map-derived flags. It does not look up groups, interpret
keep-groups exclusivity, rootless or user-namespace behavior, runtime behavior, build execution,
Compose privilege equivalence, or claim conversion behavior.

The isolated Build DNS fixture covers all 20 recorded releases. It requires ordered separate
`--dns 9.9.9.9` then `--dns 2001:4860:4860::8888` pairs before the final `.` context, rejecting
equals, quoted, empty, merged, duplicate, reordered, and post-context forms without a
relative-order claim against map-derived flags. It does not resolve DNS, establish `none`
compatibility, inspect `resolv.conf` or host DNS, execute a build, map Compose endpoints, or claim
conversion behavior.

The isolated Build DNSSearch fixture covers all 20 recorded releases. It requires ordered separate
`--dns-search corp.example` then `--dns-search .` pairs before the final `.` context, rejecting
old, empty, equals, quoted, merged, duplicate, reordered, and post-context forms without a
relative-order claim against map-derived flags. It does not apply reset or dot semantics in the
Lens model, remove domains, resolve DNS, inspect resolver state, execute a build, map Compose
values, or claim conversion behavior.

The isolated Build AuthFile fixture covers all 20 recorded releases. It requires exactly one
separate `--authfile PATH` pair for a single value, effective-last output for repeated entries, and
no flag after a final empty entry, rejecting equals, quoted, duplicate, alternate, and post-context
forms. These are generator-only results: the Lens model preserves duplicate physical lines and its
ordinary singleton diagnostic without normalizing, reading, or validating paths, parsing
credentials, classifying content or path metadata as sensitive, authenticating, or claiming build
success, runtime, Compose, or conversion behavior.

The isolated Build IgnoreFile fixture covers all 20 recorded releases. Podman 5.4.0 through 5.6.2
must reject or exclude it with no `--ignorefile` argument; Podman 5.7.0 through 6.0.2 must emit one
separate path, effective-last repeated output, and no flag after final empty, rejecting equals,
quoted, duplicate, alternate, and post-context forms. These are generator-only results: the Lens
model preserves duplicate physical lines and its ordinary singleton diagnostic without normalizing,
resolving or reading paths, parsing ignore files, inferring default ignore files, normalizing
relative paths, or claiming build success, runtime, Compose, or conversion behavior.

The isolated Build Annotation fixture covers all 20 recorded releases. It authors pre-reset values,
an empty reset, duplicate post-reset keys, quoted and C-escaped text, plus bare and malformed forms.
The generator emits its tokenized/unquoted/C-unescaped, last-key-collapsed, sorted effective map;
bare and malformed tokens are omitted through 5.5.2 and emitted from 5.6.0. QuadletLens preserves
all raw physical lines instead, without OCI validation, image metadata, build, runtime, Compose, or
conversion claims.

The isolated Build Environment fixture covers all 20 recorded releases. It authors pre-reset
values, an empty reset, duplicate names, quoted and C-escaped text, embedded equals text, plus bare
and malformed forms. The generator emits its tokenized/unquoted/C-unescaped, final-name-selected,
sorted effective map; bare and malformed tokens are omitted through 5.5.2 and emitted from 5.6.0.
QuadletLens preserves all raw physical lines instead, without host lookup, build, runtime, Compose,
or conversion claims.

The isolated Build ContainersConfModule fixture covers all 20 recorded releases. It authors
pre-reset values, an empty reset, and two post-reset entries. The generator emits only ordered
`--module=post-one` and `--module=post-two` arguments before `build`; QuadletLens preserves every
raw physical line instead, without path parsing, module reads, configuration inspection, build,
runtime, Compose, or conversion claims.

The isolated Volume ContainersConfModule fixture covers all 20 recorded releases. It authors
pre-reset values, an empty reset, two post-reset values, and a continuation. The generator emits
only ordered post-reset `--module=post-one`, `--module=post-two`, and continuation arguments
before `volume create`; its continued space is literal in 5.4.x and `\\x20` from 5.5.0. QuadletLens
preserves every raw physical line without path parsing, module reads, configuration inspection,
volume creation, lifecycle, filesystem, security, runtime, Compose, or conversion claims.

The isolated Volume GlobalArgs fixture covers all 20 recorded releases. It authors pre-reset
values, an empty reset, debug and quoted/C-escaped event-backend values, and a malformed backslash
line. The generator emits only the decoded post-reset tokens in authored order between `podman`
and `volume create`; QuadletLens preserves every raw physical line without applying target reset,
tokenization, unquoting, C-unescaping, omission, argument validation, security inference, volume
creation, lifecycle, filesystem, runtime, Compose, or conversion claims.

The isolated Volume PodmanArgs fixture covers all 20 recorded releases. It authors pre-reset
values, an empty reset, quoted and C-escaped labels, and a malformed backslash line. The generator
emits only decoded post-reset tokens in authored order at the end of `volume create` before the
volume name; QuadletLens preserves every raw physical line without target reset, tokenization,
unquoting, C-unescaping, omission, CLI parsing, dedicated-key semantics, security inference,
volume creation, lifecycle, filesystem, systemd, runtime, Compose, or conversion claims.

The isolated Volume User fixture runs on all 20 recorded releases with only unambiguous `User=123`.
It requires one `o=uid=123` option before the generated volume name and performs no volume operation;
name and invalid/default behavior remain outside the public promise.

The isolated Volume Group fixture runs on all 20 recorded releases with only unambiguous `Group=456`.
It requires one `o=gid=456` option before the generated volume name and performs no volume operation;
name and invalid/default behavior remain outside the public promise.

The isolated Build GlobalArgs fixture covers all 20 recorded releases. It authors duplicate
pre-reset values, an empty reset, quoted and C-escaped post-reset values, and a malformed physical
line. The generator emits only the retained target tokens, in authored order between `podman` and
`build`; QuadletLens preserves all authored source instead, without tokenization, reset, option
validation, semantic/security/runtime inference, build, runtime, Compose, or conversion claims.

The isolated Build ServiceName fixture covers all 20 recorded releases. It records default and
duplicate-last override names, `.service` addition, the 5.7.0 template-default boundary, and the
5.8.2 unmatched-quote lookup boundary. It observes dry-run generated-unit names only; QuadletLens
preserves raw singleton values and does not claim extension enforcement, systemd operation,
collisions, escaping, runtime, build, dependency, filename mutation, Compose, or conversion behavior.

The isolated Build Volume fixture covers all 20 recorded releases. It records reset and continuation
lookup, ordered effective `-v` entries, relative `.` source resolution, and exact `.volume`
substitution/dependency. It does not mount, access a filesystem, build, run, operate systemd, or
claim lifecycle, image, security, Compose, or conversion behavior.

These fixtures are dry-run command evidence: they start no workload and do not inspect resolver,
OCI, profile, SELinux, path, filesystem, host, runtime, or cross-format behavior.

`Memory` uses a separate version-scoped fixture because it was introduced in Podman 5.5.0. The
three 5.4.x generators must reject or exclude it and emit no memory argument. Every one of the 17
recorded releases from 5.5.0 through 6.0.2 must apply singleton last-value behavior to an earlier
value and empty assignment, then emit exactly one `--memory 16777216b` argument and no duplicate,
equals, empty, quoted, or alternate form. This is dry-run command evidence only; no workload,
cgroup, page-size, swap, host-memory, rootless, runtime, or cross-format behavior is tested.

Systemd-dependent fixtures record the systemd version and rootless/rootful context.

The Unit-relationship generator fixtures cover literal Podman 5.4 basenames, the Podman 5.5
rewrite boundary, all nine reviewed keys, all eight current native suffixes, ordinary systemd unit
names, duplicates, source order, empty resets, continuations, and missing-source failure. They run
the generator in dry-run mode only and never start a generated service or execute its Podman
command. `Upholds=` is recorded as requiring systemd 249; no runtime activation claim is made.

### Real-world fixtures

Fixtures require source provenance, redistribution permission, version assumptions, secret review, and an explanation of the protected behavior.

The implemented opt-in corpus covers 35 immutable unit files from ten public projects across
upstream application, vendor, distribution, platform, organization-example, and community evidence
classes. It verifies Git blobs, required feature markers, syntax validity, byte preservation,
canonical reparsing, typed-model validation, and native document-set references without starting a
generator or workload. See the [real-world corpus](real-world-quadlet-corpus.md).

## Test organization

Cargo-discovered integration tests live in [`../tests/`](../tests/README.md), with private helpers in `tests/support/`. Fixtures live in [`../fixtures/`](../fixtures/README.md) and are validated against the versioned [fixture manifest contract](fixture-format.md). Product suites are added only with implemented behavior and meaningful assertions.

The initial syntax suite protects authored ordering, repeated `After=` entries, `#` comments inside
a continuation, `%h` values, Unicode locations, exact preservation, LF/CRLF and final-line-ending
spelling, and structured recovery for malformed headers, entries before a section, empty keys,
invalid lines, and dangling continuations.

The canonical-rendering suite protects structural normalization without value rewriting and
refuses invalid input. A bounded generated corpus combines line endings, comment markers, quoted
values, image tags plus digests, continuations, and specifiers to prove preservation and canonical
idempotence without introducing a random dependency.

The capability suite parses the embedded strict TOML schema, locks the first-conversion surface
from Podman 5.4.0 through current 6.0.2, checks required/repeated/path/reference metadata, evaluates
the lower and rolling upper coverage boundaries, and uses synthetic evidence to exercise fallback
and known-bug precedence. Capabilities outside the generator fixture retain explicit evidence gaps.

The typed-model suite protects the initial `.container`, `.pod`, `.network`, `.volume`, `.image`, and minimal
`.build` surface. It checks ordered repeated Build `ImageTag`, `Network`, `Label`, `File`, `BuildArg`, `Secret`, `GroupAdd`, `DNS`, `DNSOption`, `DNSSearch`, `Annotation`, `Environment`, `ContainersConfModule`, and `PodmanArgs` values, singleton
opaque `SetWorkingDirectory`/`Target`/`Arch`/`Variant`/`Pull`/`Retry`/`RetryDelay`/`TLSVerify`/`ForceRM`/`AuthFile`/`IgnoreFile`/`ServiceName` values, duplicate singleton diagnostics, unknown Build-key
preservation, exact `.container` Image-to-`.build` resolution, and exact `.build`
Network-to-`.network` resolution without interpreting a build context or network mode.
Image `ImageTag`/`ServiceName`/`AllTags`/`Arch`/`AuthFile`/`CertDir` tests retain blank, duplicate, quoted, unmatched-quote,
continuation, and specifier text without reference, identity, boolean/platform/auth-file/certificate-directory, credential, certificate, or target-effective-value semantics.
Image `ContainersConfModule` tests additionally preserve blank, duplicate, quoted, escaped, specifier, continuation-looking, and leading-dash physical values without module, configuration, reset, tokenization, or command semantics.
Image `Creds` uses placeholder-only fixture values, retains raw physical text, and redacts only
repository-owned debug output; its 20-release dry-run matrix records normal, duplicate-last,
final-blank omission, and unmatched-quote output without a flag-order or quote-boundary claim.
Image `DecryptionKey` uses placeholder-only fixture values, retains raw physical text, and redacts
only repository-owned debug output; its 20-release dry-run matrix records the same four target
observations without a flag-order or quote-boundary claim.
Image `GlobalArgs` retains raw physical text, including duplicates, blank resets, quotes,
C-escapes, continuations, malformed text, and specifiers. Its 20-release dry-run matrix records
only decoded post-reset tokens in authored order between `podman` and `image pull`.
Network completion and Image pull-control tests preserve physical ordering for repeatable
`ContainersConfModule`/`DNS`/`GlobalArgs`/`PodmanArgs` entries, preserve opaque singleton text,
and exercise duplicate-singleton diagnostics. Their independently versioned generator assertions
cover the documented command forms, reset/order behavior, and 5.5.0/5.6.0 boundaries rather than
inferring behavior from similarly named Container, Build, Pod, or Volume keys.
Kube tests apply blank `Yaml=` reset semantics and require at least one resulting effective source,
retain every physical source/configuration line, and observe exact `.network` references without reading those paths or parsing Kubernetes
YAML. The generator fixture records command construction and KubeDownForce cleanup text only; it
does not execute `kube play` or `kube down`.
Artifact tests retain every native key, reset-aware repeatable values, duplicate singleton source
warnings, final `Artifact=` diagnostics (`QLM0021`/`QLM0022`), and seeded credential/decryption-key
canaries redacted from repository-owned debug output. Document-set tests resolve exact Artifact
Volume/Mount references and report missing or ambiguous targets without parsing mount text. The
generator fixture requires pre-5.7 rejection or exclusion and 5.7.0–6.0.2 command construction,
oneshot defaults, naming observations, and `DefaultDependencies` true/false/noncanonical/empty
dependency output. It never contacts a registry, reads an auth/key file, pulls an artifact, mounts
content, or starts a service.
Image `OS` retains raw singleton physical text and duplicate diagnostics; its 20-release fixture
records target normal/duplicate-last output, final-blank omission, and endpoint-specific unmatched-quote presentation.
Build labels additionally retain opaque physical lines, including bare, duplicate, empty, embedded
equals, quoted, and specifier text, without label parsing or normalization; generator assertions are
intentionally narrower and cover only `build.label=one` and `empty=`.
Build arguments retain opaque bare, empty, quoted, and specifier-bearing physical values; the isolated
generator fixture claims only `key=value` and empty-value `key=` forms from 5.7.0 through 6.0.2.
Build Secret lines likewise remain opaque with no comma, argument, environment, path, or secret-data
interpretation; their all-20-version fixture uses placeholder paths and asserts two ordered separate
`--secret` arguments only. Build Arch and Variant remain opaque singletons without platform parsing,
defaults, or effective-last normalization; their all-20-version fixture asserts exactly one `--arch arm64`
and one `--variant v8` without a relative-order assertion.
Build Pull preserves blank and duplicate physical values without policy parsing; its isolated fixture
requires exactly one `--pull=always` argument across all 20 releases, while blank omission remains source evidence only.
Build PodmanArgs retains exact physical lines without argument splitting or context resolution; its all-20 fixtures
assert one separate `--build-context extra=container-image://alpine:3.15`, exact `--no-cache`, or equals-form
`--isolation=chroot`/`--ssh=default`/`--shm-size=32m`/`--ulimit=nproc=4096:8192`/`--add-host=buildhost:192.0.2.10`/`--cap-add=CAP_SYS_ADMIN` immediately before final positional `.` only. The isolation, SSH, shared-memory, ulimit, add-host, and cap-add fixtures reject separate, quoted,
alternate, duplicate, and reordered forms; it does not lower Compose, establish mode equivalence/defaults, or claim rootless/rootful, namespace, LSM, environment, build, runtime, or cross-format behavior. The latter is repeatable command-text evidence, not Compose `no_cache` lowering,
false/string/interpolation interpretation, cache semantic equivalence, execution, cache, image, runtime, or
cross-format behavior. The non-secret SSH fixture does not provide, resolve, inspect, or claim keys, sockets, an agent, PEM data, paths, environments, mounts, builds, runtime state, or Compose lowering. The shared-memory fixture adds no native Build `ShmSize` key and does not establish Compose or unit equivalence, zero or omission defaults, IPC selection, host/cgroup/memory behavior, build execution, runtime behavior, or conversion behavior.
The add-host fixture does not lower Compose list or map `extra_hosts` forms; establish IPv6 or
`host-gateway` equivalence; alter DNS or `/etc/hosts`; resolve conflicts or defaults; execute a
build; or establish runtime or conversion behavior.
The cap-add fixture does not establish Compose entitlement equivalence or conversion; actual
capability grants; build execution; LSM, seccomp, rootless, or runtime effects.
It checks native key classification, repeated container/pod `AddHost`, container `Label`/`Secret`,
and network `Label`
entries, singleton container `ContainerName`, `RunInit` omission/true/false/raw preservation,
`StopSignal`, `StopTimeout` (including authored zero), Pod `ServiceName`, `Pull`, and `PidsLimit`
omission/supported/raw-value preservation without semantic validation, plus `HostName`
omission/raw preservation without hostname validation, opaque singleton container/pod `ShmSize`
omission/raw preservation, repeatable opaque container `DropCapability` and `AddCapability`
omission/order/raw-value preservation including empty addition resets and duplicates, repeatable
opaque container `Tmpfs` omission/order/case/options/reset/duplicate preservation without
conflation with `Volume`, plus repeatable opaque container `Sysctl`
omission/order/case/whitespace/quoting/specifier/reset/duplicate preservation without assignment
parsing or namespace validation, plus repeatable opaque container `Ulimit`
omission/order/case/quoting/specifier/reset/duplicate preservation without splitting, unquoting,
or grammar validation, plus repeatable opaque container `AddDevice`
omission/order/case/quoting/specifier/whitespace/reset/duplicate/leading-dash preservation without
splitting, unquoting, device validation, or runtime interpretation, singleton container `Memory`
omission/raw/empty/quoted/specifier/duplicate preservation with ordinary singleton diagnostics
while pod `Memory` remains unknown, plus opaque singleton `LogDriver` and repeatable/resettable
`LogOpt` physical-value, quote, specifier, duplicate, order, scope, and diagnostic preservation.
Opaque singleton `IP` and `IP6` plus repeatable/resettable `NetworkAlias` likewise protect physical
values, duplicates, order, empty resets, quotes, specifiers, continuations, scope, and standard
singleton diagnostics without semantic validation.
Opaque singleton Volume `GID` likewise protects physical values, duplicates, order, quoting,
specifiers, continuations, scope, and the standard singleton diagnostic without value
interpretation.
The Volume `ServiceName` fixture records all-20 generated-unit naming boundaries while the model
retains raw singleton source values and duplicate diagnostics without naming interpretation.
The Volume `Image` fixture records driver and resource-reference command observations only.
The promoted networking, annotation, and security keys are
tested as opaque repeatable or singleton values for ordering, resets, duplicates, malformed text,
scope, and standard diagnostics without key-specific or runtime interpretation. The suite also
checks singleton pod `UserNS`, generic systemd sections, repeated and unknown
entries, continuation segments, `%h` and unit-relative paths, native
unit references, explicit supported suffixes, required fields, singleton diagnostics, foreign
native sections, and source labels. Its document-set cases protect exact basename resolution,
resolved dependency edges, retained missing and ambiguous references, duplicate names, unique
source identities, and filename/type matching.

The generation suite constructs `.container`, `.network`, and `.volume` documents, verifies exact
deterministic output, reparses every result, and resolves the generated cross-file graph. It also
protects explicit container names, single and non-empty grouped validated environment assignments,
repeated AddHost, label, secret, systemd, and exact
capability-drop/add, tmpfs, sysctl, ulimit, and logging entries, including raw empty native resets,
plus all document-builder
rejection paths, including duplicate lifecycle, policy, and promoted security singletons,
process-ID-limit, hostname, and container/pod shared-memory singletons. The exact repeatable
boundary also covers raw host-device mappings, DNS resolver values, DNS resolver options, and mask
path lists with reset assignments, plus the opaque Volume `GID` singleton. The focused PID-limit,
shared-memory, and container-memory helpers' ASCII-decimal validation and arbitrary-precision preservation are tested
separately while the raw boundary preserves authored zero and noncanonical text.

## Regression rule

Every compatibility fix adds a version-specific regression fixture. Catalogue corrections include a test demonstrating why the earlier claim was wrong or an explicit note explaining why automation is not currently possible.

## Canonical commands

The crate uses Rust 2024 with an MSRV of 1.85.0. `rust-toolchain.toml` pins the normal development toolchain; the explicit MSRV command prevents that pin from hiding accidental use of newer language or library features.

```shell
./scripts/check-all.sh
./scripts/check-files.sh --check
cargo fmt --all -- --check
cargo ci-check
cargo ci-catalogue
cargo ci-generators
cargo ci-model
cargo ci-policy
cargo ci-real-world-quadlet
cargo ci-clippy
cargo ci-test
cargo ci-doctest
RUSTDOCFLAGS="-D warnings" cargo ci-doc
cargo llvm-cov --locked --workspace --all-features --all-targets --summary-only \
  --fail-under-regions 91 --fail-under-functions 92 --fail-under-lines 92
cargo +1.85.0 ci-check
cargo +1.85.0 ci-policy
cargo deny check
```

`scripts/check-all.sh` is the one-command local gate. It formats owned files before checking the
same deterministic Rust, coverage, MSRV, dependency, offline-link, package, and patch-SemVer
boundaries used for release preparation. Generator execution and real-world downloads remain
explicit opt-in commands.

The `ci-*` aliases use `--locked`, all workspace features, and all targets where the Cargo command
supports them. CI also runs markdownlint and lychee over the documentation. Add exact deterministic
property-style and real-generator matrix commands here before those harnesses become required checks.

The pinned `cargo-llvm-cov` 0.8.7 gate runs the locked workspace with all features and targets.
Its coarse integer floors—91% regions, 92% functions, and 92% lines—are regression guards, not a
claim that line execution proves behavior. Positive and negative assertions remain required at
each supported boundary.
