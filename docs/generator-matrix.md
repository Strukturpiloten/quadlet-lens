# Podman generator matrix

## Support policy

QuadletLens has a fixed minimum Podman version and a rolling upper target:

- minimum supported version: Podman 5.4.0;
- current upstream target checked on 2026-08-06: Podman 6.0.2;
- current generator-verified first-conversion range: Podman 5.4.0 through 6.0.2.

“Supported target,” “catalogue evidence,” and “generator verified” are deliberately separate. A
new upstream release expands the target immediately, but it does not become verified merely because
its version exists. Unsupported means evidence shows that no representation exists; unknown means
the required evidence is incomplete.

The exact tracked current release and date live in [`../tools/generator-matrix.toml`](../tools/generator-matrix.toml).

## Systemd Unit relationship references

The isolated Unit-relationship fixtures cover `Requires`, `Wants`, `After`, `Requisite`,
`BindsTo`, `PartOf`, `Upholds`, `Conflicts`, and `Before`. Podman 5.4.x must preserve native
Quadlet basenames literally; Podman 5.5.0 and newer must rewrite container, pod, network, volume,
build, image, and kube basenames to generated service names. From Artifact-unit introduction, the
matrix also requires `.artifact` to become `-artifact.service`. Duplicate tokens, source order,
empty resets, continuations, ordinary `.service`/`.target` names, and missing-source failure are
checked explicitly. This is dry-run generator evidence only; no unit is started and no generated
Podman command is executed. `Upholds` additionally requires systemd 249 or newer.
Renovate watches the current release value so new Podman releases create visible maintenance work.

## Official versioned containers

The public `quay.io/podman/stable` repository currently provides exact `-immutable` tags for every
Podman patch release from 5.4.0 through 5.8.2. QuadletLens records both the exact tag and registry
manifest digest for every image. A generator test also asks the Podman binary inside the image to
report its version before accepting its output.

The registry currently has no exact release images for Podman 5.8.3 through 6.0.2. For those six
patch releases, the harness fetches the full commit recorded from the corresponding upstream
release tag and builds only `./cmd/quadlet` in a version-and-digest-pinned Go container. It verifies
the checked-out commit and the generator's reported version before accepting output. The harness
does not currently perform cryptographic release-tag signature verification.

## What the container test does

For official images, the harness:

1. mounts an authored fixture directory read-only;
2. disables container label separation for that read-only test mount rather than relabelling source files;
3. sets `QUADLET_UNIT_DIRS=/fixtures`;
4. runs `/usr/lib/systemd/system-generators/podman-system-generator -dryrun -no-kmsg-log`;
5. verifies stable generated service fragments for `.container`, `.pod`, `.network`, `.volume`,
   `.image`, and `.kube` files.

For source-backed releases it first checks out the recorded commit with Git, builds the standalone
generator using read-only source plus persistent Go caches, and then performs the same version and
fixture checks inside the pinned builder image.

It does not run nested containers, pull the fixture's declared application image, install units,
invoke systemctl, or start generated services. Runtime, rootless/rootful, cgroup, networking, and
SELinux behavior remain separate test tiers.

The first-conversion fixture covers the minimal Build core with two ordered `ImageTag` values, ordered
`Network=host`, `Network=none`, and `Network=app.network` values, `Label=build.label=one` and
`Label=empty=`, two ordered `File` values, `Target=build-stage`, and `SetWorkingDirectory=file`.
Every recorded generator emits the ordered three Build `--network` arguments and an exact dependency
on `app.network`, exactly one argument for each of the two label values without an ordering claim, plus only the
final `--file Containerfile.final` argument and a file-derived service working directory; that
effective-last observation remains tagged-source/generator evidence and is never Lens
normalization, plus one `--target build-stage` argument without stage validation. It does not test
bare labels, duplicate-label ordering or collapse, label grammar, build execution, runtime, or
cross-format behavior. A separate BuildArg fixture requires rejection or exclusion without a mapping through
5.6.2, then exactly `--build-arg key=value` and `--build-arg empty=` from 5.7.0 through 6.0.2; it
does not establish bare/null, environment, secret, build, runtime, or
cross-format behavior. A separate Build Secret fixture requires exactly two ordered separate `--secret`
arguments for opaque placeholder-source values across all 20 releases; it does not establish bare,
environment, comma/argument parsing, path resolution, secret materialization, build, runtime, or
cross-format behavior. A separate Build platform fixture requires exactly one `--arch arm64` and one
`--variant v8` argument across all 20 releases without asserting relative argument order. It does not
parse platform grammar, select host defaults, apply effective-last behavior, build an image, inspect
metadata, or establish runtime or cross-format behavior. A separate Build Pull fixture requires
exactly one `--pull=always` argument across all 20 releases; source evidence records blank-value
omission only, and policy acceptance, defaults, Compose mapping, registry, image-pull, and runtime behavior remain unclaimed. The separate Build retry fixture records rejection or exclusion without
`--retry`/`--retry-delay` output in all three 5.4.x releases, then requires each of the 17 releases
from 5.5.0 through 6.0.2 to emit exactly one separate `--retry 4` pair and one separate
`--retry-delay 7s` pair before the final `.` context without a relative-order claim between pairs.
The endpoint [5.5.0 Retry manual](https://docs.podman.io/en/v5.5.0/markdown/podman-systemd.unit.5.html#id54),
[5.5.0 RetryDelay manual](https://docs.podman.io/en/v5.5.0/markdown/podman-systemd.unit.5.html#id55),
[6.0.2 Retry manual](https://docs.podman.io/en/v6.0.2/markdown/podman-systemd.unit.5.html#id60),
[6.0.2 RetryDelay manual](https://docs.podman.io/en/v6.0.2/markdown/podman-systemd.unit.5.html#id61),
and tagged [5.5.0](https://github.com/podman-container-tools/podman/blob/v5.5.0/pkg/systemd/quadlet/quadlet.go)
and [6.0.2](https://github.com/podman-container-tools/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/quadlet/quadlet.go)
source document the finite range. It does not parse integer or duration text, choose defaults,
apply effective-last behavior, link Compose `dockerfile_inline`, contact a registry, execute retry
or timing behavior, establish build success, inspect runtime behavior, or claim conversion behavior.

The Build TLSVerify fixture has one `TLSVerify=true` unit and one `TLSVerify=false` unit. Every
recorded generator from 5.4.0 through 6.0.2 must emit one bare `--tls-verify` for the true unit and
one `--tls-verify=false` for the false unit, each before final `.`. Endpoint
[5.4.0](https://docs.podman.io/en/v5.4.0/markdown/podman-systemd.unit.5.html#tlsverify) and
[6.0.2](https://docs.podman.io/en/v6.0.2/markdown/podman-build.unit.5.html#tlsverify) manuals plus
tagged [5.4.0](https://github.com/podman-container-tools/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/pkg/systemd/quadlet/quadlet.go)
and [6.0.2](https://github.com/podman-container-tools/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/quadlet/quadlet.go)
mapping/formatter source ground this command-text observation. It does not establish TLS
connectivity, certificate validation, registry configuration, image pull, build success, security
posture, provenance equivalence, runtime behavior, or conversion behavior.

The Build ForceRM fixture has one `ForceRM=true` unit and one `ForceRM=false` unit. Every recorded
generator from 5.4.0 through 6.0.2 must emit one bare `--force-rm` for the true unit and one
`--force-rm=false` for the false unit, each before final `.` and without equals, quoted, alternate,
duplicate, or post-context forms. Endpoint
[5.4.0](https://docs.podman.io/en/v5.4.0/markdown/podman-systemd.unit.5.html#forcerm) and
[6.0.2](https://docs.podman.io/en/v6.0.2/markdown/podman-build.unit.5.html#forcerm) manuals plus
tagged [5.4.0](https://github.com/podman-container-tools/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/pkg/systemd/quadlet/quadlet.go)
and [6.0.2](https://github.com/podman-container-tools/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/quadlet/quadlet.go)
mapping/formatter source ground this command-text observation. It does not parse booleans, select
defaults, apply effective-last behavior, or establish cleanup occurrence, failure behavior,
execution, defaults or configuration, cache equivalence, runtime behavior, or conversion behavior.

The Build GroupAdd fixture has ordered `GroupAdd=1234` and `GroupAdd=5678` values. Every recorded
generator from 5.4.0 through 6.0.2 must emit ordered separate `--group-add 1234` then
`--group-add 5678` pairs before final `.`, rejecting equals, quoted, merged, duplicate, reordered,
and post-context forms, without a relative-order claim against map-derived flags. Endpoint
[5.4.0](https://docs.podman.io/en/v5.4.0/markdown/podman-systemd.unit.5.html#groupadd) and
[6.0.2](https://docs.podman.io/en/v6.0.2/markdown/podman-build.unit.5.html#groupadd) manuals plus
tagged mapping/formatter source establish command text only. The fixture does not look up groups,
interpret keep-groups exclusivity, rootless or user-namespace behavior, runtime behavior, build
execution, Compose privilege equivalence, or conversion behavior.

The Build DNS fixture has ordered `DNS=9.9.9.9` and `DNS=2001:4860:4860::8888` values. Every
recorded generator from 5.4.0 through 6.0.2 must emit ordered separate `--dns 9.9.9.9` then
`--dns 2001:4860:4860::8888` pairs before final `.`, rejecting equals, quoted, empty, merged,
duplicate, reordered, and post-context forms without a relative-order claim against map-derived
flags. Endpoint manuals and tagged mapping/formatter source establish command text only. The
fixture does not resolve DNS, establish `none` compatibility, inspect `resolv.conf` or host DNS,
execute a build, map Compose endpoints, or define conversion behavior.

The Build DNSSearch fixture has pre-reset `old.example`, an empty reset, `corp.example`, and
literal `.` values. Every recorded generator from 5.4.0 through 6.0.2 must emit ordered separate
`--dns-search corp.example` then `--dns-search .` pairs before final `.`, rejecting old, empty,
equals, quoted, merged, duplicate, reordered, and post-context forms without a relative-order
claim against map-derived flags. Endpoint manuals and tagged mapping/formatter source establish
command text only. The fixture does not apply model reset or dot semantics, remove domains, resolve
DNS, inspect resolver state, execute a build, map Compose values, or define conversion behavior.

The Build AuthFile fixture has a single-path unit, a repeated-path unit, and a final-empty unit.
Every recorded generator from 5.4.0 through 6.0.2 must emit one separate `--authfile PATH` pair
for the single unit, only the effective last path for the repeated unit, and no `--authfile` flag
for final empty, rejecting equals, quoted, duplicate, alternate, and post-context forms. Endpoint
manuals and tagged Lookup/formatter source establish command construction only. The fixture does
not read or validate paths, obtain or parse credentials, classify content or path metadata as
sensitive, authenticate, establish build success, or define runtime, Compose, or conversion behavior.

The Build IgnoreFile fixture has a single-path unit, a repeated-path unit, and a final-empty unit.
Every recorded generator from 5.4.0 through 5.6.2 must reject or exclude it with no `--ignorefile`
argument. Every recorded generator from 5.7.0 through 6.0.2 must emit one separate `--ignorefile
PATH` pair for the single unit, only the effective last path for the repeated unit, and no flag for
final empty, rejecting equals, quoted, duplicate, alternate, and post-context forms. Endpoint
manuals and tagged Lookup/formatter source establish command construction only. The fixture does
not resolve or read paths, parse ignore files, infer defaults, normalize relative paths, establish
build success, or define runtime, Compose, or conversion behavior.
A Build Annotation fixture authors pre-reset values, an empty reset, duplicate post-reset keys,
quoted and C-escaped values, and bare/malformed forms. Every recorded generator emits the target's
effective sorted map as separate `--annotation` arguments after reset, tokenization, unquoting,
C-unescaping, and final-key collapse. Bare and malformed tokens are absent through 5.5.2 and present
from 5.6.0. This records target command construction only; QuadletLens keeps the authored physical
lines untouched and makes no OCI, image-metadata, build, runtime, Compose, or conversion claim.
A Build Environment fixture authors pre-reset values, an empty reset, duplicate post-reset names,
quoted and C-escaped values, embedded equals text, and bare/malformed forms. Every recorded generator
emits a sorted effective map as separate `--env` arguments after reset, tokenization, unquoting,
C-unescaping, and final-name selection. Bare and malformed tokens are absent through 5.5.2 and
present from 5.6.0. This records target command construction only; QuadletLens keeps authored
physical lines untouched and makes no host-lookup, build, runtime, Compose, or conversion claim.
A Build ContainersConfModule fixture authors pre-reset values, an empty reset, and ordered
post-reset entries. Every recorded generator emits only `--module=post-one` then
`--module=post-two` before `build` and its final context. This records target logical lookup and
command construction only; QuadletLens keeps authored physical lines untouched and makes no
module-path resolution, module-read, configuration-effect, build, runtime, Compose, or conversion claim.
A Build GlobalArgs fixture authors duplicate pre-reset values, an empty reset, quoted and
C-escaped post-reset values, and a malformed physical line. Every recorded generator emits only
the retained tokens in authored order between `podman` and `build`. This records target command
construction only; QuadletLens keeps authored physical lines untouched and makes no option
validation, semantic/security/runtime, build, Compose, or conversion claim.
A separate Image GlobalArgs fixture authors the same pre-reset, reset, quoted, C-escaped, and
malformed physical forms. Every recorded generator emits only decoded post-reset tokens in authored
order between `podman` and `image pull`; QuadletLens preserves authored physical lines without
tokenization, reset, unquoting, C-unescaping, option validation, or pull semantics.
A separate Build PodmanArgs fixture requires exactly one separate
`--build-context extra=container-image://alpine:3.15` immediately before final positional `.` across all 20 releases,
rejecting equals, quoted, alternate, duplicate, and reordered forms. It runs only the dry-run generator and does not
lower Compose contexts, resolve paths/environments/images/services, validate a CLI, build, run, or claim cross-format behavior. A second isolated Build PodmanArgs fixture requires exactly one separate
`--no-cache` immediately before final positional `.` across all 20 releases, likewise rejecting equals,
quoted, alternate, duplicate, and reordered forms. It is repeatable command-text evidence only: it does
not lower Compose `no_cache`, interpret false, string, or interpolation values, establish cache semantic
equivalence, or claim execution, cache, image, runtime, or cross-format behavior. A third isolated
Build PodmanArgs fixture requires exactly one equals-form `--isolation=chroot` immediately before final positional `.` across all 20 releases, rejecting separate, quoted, alternate, duplicate, and reordered forms. It forwards command text only: it does not lower Compose, establish isolation-mode equivalence/defaults, or claim rootless/rootful, namespace, LSM, environment, build, runtime, or cross-format behavior. A fourth isolated
Build PodmanArgs fixture requires one equals-form `--ssh=default` immediately before final positional `.` across all 20 releases, rejecting separate, quoted, alternate, duplicate, and reordered forms. It forwards non-secret command text only: it does not provide, resolve, inspect, or claim keys, sockets, an agent, PEM data, paths, environments, mounts, builds, runtime state, or Compose lowering. A fifth isolated Build PodmanArgs fixture requires one equals-form `--shm-size=32m` immediately before final positional `.` across all 20 releases, rejecting separate, quoted, alternate, duplicate, and reordered forms. It adds no native Build `ShmSize` key and does not establish Compose or unit equivalence, zero or omission defaults, IPC selection, host/cgroup/memory behavior, build execution, runtime behavior, or conversion behavior. A sixth isolated Build PodmanArgs fixture requires one ordered terminal `--cache-from registry.invalid/quadlet-lens/cache-from --cache-to registry.invalid/quadlet-lens/cache-to .` chain across all 20 releases, rejecting equals, quoted, missing, duplicate, and reordered forms. It forwards command text only: it does not lower Compose, parse descriptors or cache types, resolve images, credentials, paths, or registries, validate the CLI, build, use an effective cache, run, or establish runtime or cross-format behavior. A seventh isolated Build PodmanArgs fixture requires one ordered terminal `--sbom=syft --sbom-output=/tmp/quadlet-lens-sbom.json .` pair across all 20 releases, rejecting missing output, quoted, alternate, duplicate, and reordered forms. It forwards command text only: it does not lower Compose; create a file; download an image; run a scanner; establish SBOM content, PURLs, attestations, publishing, provenance, build, runtime, security, or conversion behavior. An eighth isolated Build PodmanArgs fixture requires one equals-form `--add-host=buildhost:192.0.2.10` immediately before final positional `.` across all 20 releases, rejecting separate, quoted, alternate, duplicate, and reordered forms. It does not lower Compose list or map `extra_hosts` forms; establish IPv6 or `host-gateway` equivalence; alter DNS or `/etc/hosts`; resolve conflicts or defaults; execute a build; or establish runtime or conversion behavior. The fixture also covers mutually exclusive registry-image and host-rootfs workload
sources, including `name:tag@digest` images and absolute `Rootfs` values, explicit container names,
JSON-array entrypoints, explicit true/false init-process selection, commands,
named and numeric container stop signals plus positive and zero container stop timeouts,
isolated `always`, `missing`, `never`, and `newer` container pull policies,
isolated positive and `-1` container PID limits,
an isolated `HostName=app.example` container using Podman's default private UTS namespace,
isolated `ShmSize=67108864b` and `ShmSize=0` containers plus `ShmSize=32m` on a pod with a joined
container, isolated containers with three ordered `DropCapability` or `AddCapability` entries
covering one capability, uppercase `ALL`, and a two-capability space-separated list, plus a
combined `DropCapability=ALL` and `AddCapability=CAP_NET_BIND_SERVICE` container,
an isolated container with two pre-reset `Tmpfs` entries, an empty `Tmpfs=` reset, and one final
`Tmpfs=/data:mode=755,uid=1009,gid=1009` entry,
an isolated container with two pre-reset `Sysctl` entries, an empty `Sysctl=` reset, and one final
`Sysctl=net.ipv4.ip_forward=1` entry,
an isolated container with pre-reset `Ulimit=core=0:0` and `Ulimit=nofile=1024:2048` entries, an
empty `Ulimit=` reset, then `Ulimit=nproc=4096:8192` and `Ulimit=stack=-1:-1`,
an isolated container with one pre-reset `AddDevice=` line containing
`/dev/null:/dev/pre-null:r /dev/zero:/dev/pre-zero:w`, an empty reset, then one final line
containing `/dev/null:/dev/final-null:r /dev/zero:/dev/final-zero:w`,
isolated post-reset fixtures for `DNS`, `DNSOption`, `DNSSearch`, `ExposeHostPort`,
`Annotation`, `Mask`, and `Unmask`,
an isolated network-identity container with singleton `IP` and `IP6`, one `Network=bridge`, two
pre-reset aliases, an empty reset, and two final aliases,
isolated singleton fixtures for AppArmor, no-new-privileges, seccomp, and each SELinux-label key,
isolated generic `PodmanArgs=--interactive`, `PodmanArgs=--tty`, `PodmanArgs=--privileged`, and
`PodmanArgs=--privileged=false` escape-hatch fixtures,
environment and systemd specifiers, absolute and unit-relative environment files, repeated
container labels, repeated mounted and environment-variable secrets with options, repeatable
container and pod host mappings including `host-gateway`, container and pod membership, the
container user/group and user namespace, the pod's shared user namespace, supplementary groups,
working directory, read-only root filesystem, supported port spellings, native and external
networks, named/anonymous/relative and `.volume` mounts, SELinux mount-option spelling, health
commands including `none`, regular health timings, `Notify=healthy` readiness, generic systemd
`Requires`/`Wants`/`After` dependency ordering and restart behavior, continued `PodmanArgs`,
and generated cross-unit dependencies. The isolated Build `PodmanArgs=--ulimit=nproc=4096:8192`
fixture requires exactly one equals-form immediately before final positional `.` across every
recorded release, rejecting separate, quoted, alternate, duplicate, and reordered spellings. It
adds no native Build `Ulimit` key and establishes no Compose name, range, or `-1` equivalence;
host/rootless/rootful, `RUN`, cgroup, default, build, runtime resource-limit enforcement, or
conversion behavior. These are generator claims; actual activation, failure
propagation, rootless/rootful, and SELinux enforcement remain runtime evidence.

The endpoint [Podman 5.4.0 Quadlet](https://docs.podman.io/en/v5.4.0/markdown/podman-systemd.unit.5.html#podmanargs)
and [6.0.2 Build-unit](https://docs.podman.io/en/v6.0.2/markdown/podman-build.unit.5.html#podmanargs)
manuals establish generic `PodmanArgs` forwarding. The matching [5.4.0](https://docs.podman.io/en/v5.4.0/markdown/podman-build.1.html#ulimit-type-soft-limit-hard-limit)
and [6.0.2](https://docs.podman.io/en/v6.0.2/markdown/podman-build.1.html#ulimit-type-soft-limit-hard-limit)
build manuals document `--ulimit=type=soft-limit[:hard-limit]` for `RUN` processes. Endpoint
generator source at [5.4.0](https://github.com/podman-container-tools/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/pkg/systemd/quadlet/quadlet.go#L1949-L1953)
and [6.0.2](https://github.com/podman-container-tools/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/quadlet/quadlet.go#L2009-L2013)
records repeatable `LookupAllArgs` forwarding before the final build context. None of this equates
Compose names/ranges/`-1` with Podman, claims a native Build `Ulimit` key, or establishes host,
rootless/rootful, `RUN`, cgroup, default, build, runtime, or conversion behavior.

The endpoint [Podman 5.4.0 Quadlet](https://docs.podman.io/en/v5.4.0/markdown/podman-systemd.unit.5.html#podmanargs)
and [6.0.2 Build-unit](https://docs.podman.io/en/v6.0.2/markdown/podman-build.unit.5.html#podmanargs)
manuals establish generic `PodmanArgs` forwarding. The matching [5.4.0](https://docs.podman.io/en/v5.4.0/markdown/podman-build.1.html#add-host-hostname-hostname-ip)
and [6.0.2](https://docs.podman.io/en/v6.0.2/markdown/podman-build.1.html#add-host-hostname-hostname-ip)
build manuals document `--add-host=hostname:ip`. Endpoint generator source at
[5.4.0](https://github.com/podman-container-tools/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/pkg/systemd/quadlet/quadlet.go#L1949-L1953)
and [6.0.2](https://github.com/podman-container-tools/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/quadlet/quadlet.go#L2009-L2013)
records repeatable `LookupAllArgs` forwarding before the final build context. The all-20-release
fixture proves only one terminal `--add-host=buildhost:192.0.2.10 .` command-text pair. It does
not lower Compose list or map `extra_hosts` forms; establish IPv6 or `host-gateway` equivalence;
alter DNS or `/etc/hosts`; resolve conflicts or defaults; execute a build; or establish runtime or
conversion behavior.

The endpoint [Podman 5.4.0 Quadlet](https://docs.podman.io/en/v5.4.0/markdown/podman-systemd.unit.5.html#podmanargs)
and [6.0.2 Build-unit](https://docs.podman.io/en/v6.0.2/markdown/podman-build.unit.5.html#podmanargs)
manuals establish generic `PodmanArgs` forwarding. The matching [5.4.0](https://docs.podman.io/en/v5.4.0/markdown/podman-build.1.html#cap-add-capability)
and [6.0.2](https://docs.podman.io/en/v6.0.2/markdown/podman-build.1.html#cap-add-capability)
build manuals document `--cap-add=CAPABILITY`. Endpoint generator source at
[5.4.0](https://github.com/podman-container-tools/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/pkg/systemd/quadlet/quadlet.go#L1949-L1953)
and [6.0.2](https://github.com/podman-container-tools/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/quadlet/quadlet.go#L2009-L2013)
records repeatable `LookupAllArgs` forwarding before the final build context. The all-20-release
fixture proves only one terminal `--cap-add=CAP_SYS_ADMIN .` command-text pair. It does not
establish Compose entitlement equivalence or conversion; actual capability grants; build execution;
LSM, seccomp, rootless, or runtime effects.

The endpoint [Podman 5.4.0 Quadlet](https://docs.podman.io/en/v5.4.0/markdown/podman-systemd.unit.5.html#podmanargs)
and [6.0.2 Build-unit](https://docs.podman.io/en/v6.0.2/markdown/podman-build.unit.5.html#podmanargs)
manuals establish generic `PodmanArgs` forwarding. The matching 5.4.0 and 6.0.2 build manuals
document [`--sbom=PRESET`](https://docs.podman.io/en/v5.4.0/markdown/podman-build.1.html#sbom-preset)
and [`--sbom-output`](https://docs.podman.io/en/v6.0.2/markdown/podman-build.1.html#sbom-output-path).
Endpoint generator source at [5.4.0](https://github.com/podman-container-tools/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/pkg/systemd/quadlet/quadlet.go#L1949-L1953)
and [6.0.2](https://github.com/podman-container-tools/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/quadlet/quadlet.go#L2009-L2013)
records repeatable `LookupAllArgs` forwarding before the final build context. This is command-text
evidence only: it does not create files, download images, run scanners, or establish SBOM, PURL,
attestation, publishing, provenance, build, runtime, security, or conversion behavior.

`Memory` uses a separate fixture because the native key was introduced in Podman 5.5.0 and must
not make the existing all-20 first-conversion fixture conditional. It authors an earlier
`Memory=32m`, an empty assignment, and a final explicit-byte `Memory=16777216b`. The full lane runs
that fixture against the three 5.4.x releases to require rejection or exclusion with no memory
argument, then against all 17 recorded patches from 5.5.0 through 6.0.2 to require exactly one
final `--memory 16777216b` argument and no duplicate, equals, empty, quoted, or alternate form.
The smoke lane protects the 5.4.0 unsupported boundary plus 5.8.2 and current 6.0.2 support.

Container logging uses a separate all-20 fixture. It authors `LogDriver=k8s-file`, two pre-reset
`LogOpt` entries, an empty reset, then final `tag=quadlet-lens-final` and
`path=/tmp/quadlet-lens-final.log` entries. Each selected smoke, full, or exact-version run requires
one driver argument and exactly those two ordered final option arguments.

The `PodmanArgs=--interactive` fixture is also isolated and runs across all 20 releases. It
requires exactly one separate `--interactive` argument immediately before its image, rejecting
short, equals, quoted, alternate, and duplicate forms. It invokes only the dry-run generator, so
it proves command text rather than runtime stdin, attach, or TTY behavior.

The `PodmanArgs=--tty` fixture is also isolated and runs across all 20 releases. It requires
exactly one separate `--tty` argument immediately before its image, rejecting `-t`, combined,
equals, quoted, alternate, and duplicate forms. It retains generic `PodmanArgs` as the sole public
API rather than adding a `Tty` key or wrapper. It invokes only the dry-run generator, so it proves
command text rather than runtime TTY, stdout, stderr, or pipe behavior.

The `PodmanArgs=--privileged` fixture is likewise isolated and runs two units across all 20
releases: one requires exactly one bare `--privileged` argument and one exactly one
`--privileged=false` argument, each immediately before its respective image. It rejects
`--privileged=true`, positional false, short, quoted, bundled, alternate, duplicate, and
conflicting forms. Generic `PodmanArgs` remains the sole public API; no `Privileged` key or wrapper
is introduced. Endpoint Quadlet manuals, tagged command-placement source, and Podman CLI
boolean/default documentation support the finite 5.4.0-through-6.0.2 claim. The dry-run evidence
is command text only, not runtime privileges, devices, LSM, seccomp, rootless, or cross-format
equivalence.

Volume Device/Type coverage uses the all-20 volume fixture plus a Type-only negative fixture. It
requires final `--opt device=tmpfs` and `--opt type=bind` forms, final blank suppression, and
single logical command construction for matched/unmatched quotes, specifiers, and continuations.
Every release rejects `Type=bind` without `Device`. A bind source containing a literal space adds
no Device-derived `RequiresMountsFor` line through 5.5.2, an unescaped line through 5.7.1, and a
quoted `\\x20` line from 5.8.0; the stable `%t/containers` dependency is intentionally excluded
from that comparison. These are dry-run generator observations, not source-path, filesystem,
mount, rootless, runtime, or cross-format claims.

The quote-bearing label case also records a generated-service presentation boundary. Podman 5.4.x
keeps the JSON-like label's space literal inside a quoted argument; every tested release from 5.5.0
onward writes the equivalent systemd `\x20` escape. The harness requires exactly the observed form
family and still verifies that the complete label remains one quoted `--label` argument.

The entrypoint case likewise records an exact generated-service presentation boundary. Podman
5.4.0 through 5.8.1 emit `--entrypoint` plus a separate JSON-array argument; Podman 5.8.2 through
6.0.2 emit one quoted `--entrypoint=...` argument. The harness requires exactly the observed form
and verifies that both encodings carry the same JSON array.

The `RunInit` observations are version-invariant across the matrix: authored `RunInit=true`
generates exactly one `--init` argument, while a dedicated authored `RunInit=false` unit generates
exactly one `--init=false` argument. These are generator-output observations, separate from
QuadletLens's source and model behavior: omission remains omission, and explicit `true`, `false`,
or raw noncanonical one-line text remains exactly authored. The harness does not start a container,
inspect the init binary, or test runtime signal forwarding or child reaping.

The stop-lifecycle observations require `--stop-signal SIGUSR1`, a separate numeric
`--stop-signal 9`, `--stop-timeout 37`, and a separate `--stop-timeout 0` case. These prove native
generator emission for the named/numeric signal forms and preservation of zero; the harness does
not start a container, measure its stop interval, establish whether zero sends a signal, or infer
equivalence with another format's default or zero behavior.

The pull-policy observations use four isolated services and require exactly one matching
`--pull always`, `--pull missing`, `--pull never`, or `--pull newer` argument in each generated
service. They prove generator output only: the harness does not contact a registry, inspect local
image storage, or start a container.

The PID-limit observations use two isolated services and require exactly one matching
`--pids-limit 127` or `--pids-limit -1` argument in the corresponding generated service, with no
second PID-limit form. They prove generator output only: the harness does not exercise authored
zero, start a container, inspect a pids cgroup, or observe enforcement and exhaustion behavior.

The hostname observation uses one isolated service and requires exactly one
`--hostname app.example` argument, rejecting duplicates and any other `--hostname` form in that
service. The fixture relies on Podman's default private UTS namespace. It does not start the
container, inspect the runtime hostname or namespace, join the container to a pod, or prove the
documented rule that a pod's hostname wins when the pod shares UTS by default.

The shared-memory observations use three owning services and require exactly one matching
`--shm-size 67108864b`, `--shm-size 0`, or pod-owned `--shm-size 32m` argument, rejecting duplicates.
The container joining the shared-memory pod must contain no `--shm-size` argument of its own. This
proves explicit generator output and pod ownership only. It does not start a workload, inspect
`/dev/shm` or IPC namespaces, exercise the documented host-IPC conflict, enforce a memory limit,
or establish the documented `64m` omission default or rootless behavior.

The capability observations are grounded in three distinct evidence layers. The authoritative
[Podman 5.4 Quadlet manual](https://docs.podman.io/en/v5.4.0/markdown/podman-systemd.unit.5.html#dropcapability)
documents a repeatable space-separated drop list and lowercase `all`. Its separate
[AddCapability prose](https://docs.podman.io/en/v5.4.0/markdown/podman-systemd.unit.5.html#addcapability)
documents repeatable space-separated additions beyond the default set, but does not document
`all`; QuadletLens attributes that special behavior only to tagged source and generator output.

Exact tagged source at both evidence boundaries records the implementation behavior:

- [Podman 5.4.0 command construction](https://github.com/podman-container-tools/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/pkg/systemd/quadlet/quadlet.go#L742-L751)
  and [Podman 6.0.2 command construction](https://github.com/podman-container-tools/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/quadlet/quadlet.go#L814-L823)
  lowercase individual capability arguments and append every drop before every addition;
- the [Podman 5.4.0 list parser](https://github.com/podman-container-tools/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/pkg/systemd/parser/unitfile.go#L760-L805)
  and [Podman 6.0.2 list parser](https://github.com/podman-container-tools/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/parser/unitfile.go#L769-L814)
  clear earlier repeated values on an empty assignment before splitting later space-separated
  values; and
- the capability merger vendored by [Podman 5.4.0](https://github.com/podman-container-tools/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/vendor/github.com/containers/common/pkg/capabilities/capabilities.go#L125-L196)
  and [Podman 6.0.2](https://github.com/podman-container-tools/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/vendor/go.podman.io/common/pkg/capabilities/capabilities.go#L125-L196)
  treats add-all as the known bounding set and drop-all plus specific additions as only those
  additions.

Separately, every one of the 20 recorded generators emits exactly four ordered lowercase
separate-argument forms from the isolated drop fixture and exactly four from the isolated add
fixture. Each unit has no opposite capability form. The combined fixture emits exactly
`--cap-drop all` followed by `--cap-add cap_net_bind_service` and no other capability argument.
No equals-form presentation was observed, so the harness guards the separate form only and
duplicate-safely checks exact counts and order. The combined output is generator/source evidence
for command construction and documented merger intent, not execution of the merger or runtime
privilege state. The lane does not test rootless/rootful execution, effective or bounding sets,
user namespaces, SELinux/seccomp interaction, or whether the runtime ultimately grants or removes
a privilege.

The temporary-filesystem observation likewise separates its evidence layers. The authoritative
[Podman 5.4 Quadlet manual](https://docs.podman.io/en/v5.4.0/markdown/podman-systemd.unit.5.html#tmpfs)
documents repeatable `Tmpfs=CONTAINER-DIR[:OPTIONS]` mapping to Podman `--tmpfs`. The separate
[Podman run CLI documentation](https://docs.podman.io/en/v5.4.0/markdown/podman-run.1.html#tmpfsfs)
describes the supported options as Linux default mount flags and records
`rw,noexec,nosuid,nodev` when options are omitted. Those CLI target/runtime details are not parser
or generator validation rules.

Tagged source at [Podman 5.4.0](https://github.com/podman-container-tools/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/pkg/systemd/quadlet/quadlet.go#L641-L651)
and [Podman 6.0.2](https://github.com/podman-container-tools/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/quadlet/quadlet.go#L706-L716)
maps `Tmpfs` through the repeated-string helper, which uses `LookupAll`. The corresponding 5.4.0
and 6.0.2 parser evidence cited by the catalogue shows that an empty assignment clears earlier
logical values before later entries are returned. Every one of the 20 recorded generators then
confirms the post-reset command result: `tmpfs.service` contains exactly one logical
`--tmpfs /data:mode=755,uid=1009,gid=1009`, neither pre-reset path, and no duplicate, equals, or
other tmpfs form.

That fixture does not split or validate options, run Podman, start a container, create or inspect a
mount, verify the documented default flags, exercise `tmpcopyup`/`notmpcopyup`, or establish Linux
target-option availability, rootless behavior, ownership effects, or runtime filesystem
properties. It also makes no claim about pod keys, `Volume` tmpfs syntax, or cross-format mapping.

The kernel-parameter observation also separates native definition, tagged implementation, and
generated-command evidence. The endpoint [Podman 5.4.0](https://docs.podman.io/en/v5.4.0/markdown/podman-systemd.unit.5.html#sysctl)
and [Podman 6.0.2](https://docs.podman.io/en/v6.0.2/markdown/podman-systemd.unit.5.html#sysctl)
Quadlet manuals document repeatable, space-separated `name=value` lists mapped to `--sysctl`.
The corresponding Podman-run manuals limit accepted parameters by their namespaced IPC/network
context; QuadletLens does not turn those runtime limits into parser or builder validation.

Tagged generator source at [5.4.0](https://github.com/podman-container-tools/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/pkg/systemd/quadlet/quadlet.go#L754-L757)
and [6.0.2](https://github.com/podman-container-tools/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/quadlet/quadlet.go#L826-L829)
uses `LookupAllStrv` to append one `--sysctl` pair per effective word. Separate tagged parser
evidence in the catalogue records systemd-compatible whitespace/quote tokenization and the empty
assignment that resets prior logical values. All 20 recorded generators confirm that
`sysctl.service` contains exactly one final `--sysctl net.ipv4.ip_forward=1`, neither pre-reset
setting, and no other sysctl form.

This isolated fixture invokes only the dry-run generator. It does not start a container, inspect
IPC or network namespaces, exercise rootless behavior, ask the kernel to accept a parameter,
compare runtime equivalence, or observe an actual sysctl effect. Pod `Sysctl`, Compose, and
BoxFerry mapping remain outside this evidence.

The resource-limit observation keeps native definition, target CLI grammar, tagged implementation,
and generated-command evidence separate. The endpoint [Podman 5.4.0](https://docs.podman.io/en/v5.4.0/markdown/podman-systemd.unit.5.html#ulimit)
and [Podman 6.0.2](https://docs.podman.io/en/v6.0.2/markdown/podman-systemd.unit.5.html#ulimit)
Quadlet manuals document repeatable `Ulimit` entries and show their `--ulimit` mapping. The
corresponding [5.4.0](https://docs.podman.io/en/v5.4.0/markdown/podman-run.1.html#ulimit-option)
and [6.0.2](https://docs.podman.io/en/v6.0.2/markdown/podman-run.1.html#ulimit-option) Podman-run
manuals document `TYPE=SOFT[:HARD]`, `-1`, and omission/default caveats. QuadletLens records those
as target documentation without parsing or validating the grammar or adopting the defaults.

Tagged generator source at [5.4.0](https://github.com/podman-container-tools/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/pkg/systemd/quadlet/quadlet.go#L641-L651)
and [6.0.2](https://github.com/podman-container-tools/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/quadlet/quadlet.go#L706-L716)
maps `KeyUlimit` to `--ulimit` through the repeated-string helper. The endpoint helpers at
[5.4.0](https://github.com/podman-container-tools/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/pkg/systemd/quadlet/quadlet.go#L2027-L2034)
and [6.0.2](https://github.com/podman-container-tools/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/quadlet/quadlet.go#L2091-L2098)
use `LookupAll`, not `LookupAllStrv`, so every effective authored entry becomes one logical
argument. Separate tagged parser evidence in the catalogue records the empty assignment that
resets prior logical values.

All 20 recorded generators require `ulimit.service` to contain exactly two ordered final
arguments, `--ulimit nproc=4096:8192` followed by `--ulimit stack=-1:-1`, with neither pre-reset
limit and no duplicate, empty, or alternate form. The fixture invokes only the dry-run generator;
it does not execute a container or claim runtime enforcement, host inheritance, defaults,
cgroups, rootless behavior, or acceptance of unverified resource names. Pod `Ulimit`, Compose,
and BoxFerry mapping remain outside this evidence.

The host-device observation keeps native definition, Podman CLI caveats, tagged implementation,
and generated-command evidence separate. The endpoint [Podman 5.4.0](https://docs.podman.io/en/v5.4.0/markdown/podman-systemd.unit.5.html#adddevice)
and [Podman 6.0.2](https://docs.podman.io/en/v6.0.2/markdown/podman-systemd.unit.5.html#adddevice)
Quadlet manuals document repeatable `AddDevice` entries, their
`HOST-DEVICE[:CONTAINER-DEVICE][:PERMISSIONS]` spelling, and a leading `-` for conditional
inclusion. The corresponding [5.4.0](https://docs.podman.io/en/v5.4.0/markdown/podman-run.1.html#device-host-device-container-device-permissions)
and [6.0.2](https://docs.podman.io/en/v6.0.2/markdown/podman-run.1.html#device-host-device-container-device-permissions)
Podman-run manuals are retained as target caveat evidence, not runtime claims.

Tagged generator source at [5.4.0](https://github.com/podman-container-tools/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/pkg/systemd/quadlet/quadlet.go#L724-L734)
and [6.0.2](https://github.com/podman-container-tools/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/quadlet/quadlet.go#L795-L805)
uses `LookupAllStrv`, applies the documented conditional leading-minus branch, and appends one
`--device` pair per retained word. Separate tagged parser evidence records systemd-compatible
whitespace/quote tokenization and empty-assignment reset behavior. QuadletLens does not reproduce
any of those transformations in its native model or builder; every authored physical value stays
opaque.

All 20 recorded generators require `device.service` to contain exactly two ordered final
arguments, `--device /dev/null:/dev/final-null:r` followed by
`--device /dev/zero:/dev/final-zero:w`, and exactly two `--device` forms total, with neither
pre-reset mapping nor duplicate, empty, or alternate form. The fixture deliberately contains no
leading `-`, invokes only the dry-run generator, and starts no workload. It establishes no CDI,
runtime access, rootless, SELinux, cgroup, host-device-existence, or symlink behavior. Pod
`AddDevice`, Compose, and BoxFerry mapping remain outside this evidence.

The container-logging observation is similarly generator-only. The endpoint
[5.4.0](https://docs.podman.io/en/v5.4.0/markdown/podman-systemd.unit.5.html#logdriver) and
[6.0.2](https://docs.podman.io/en/v6.0.2/markdown/podman-systemd.unit.5.html#logdriver) manuals map
`LogDriver` to `--log-driver`; their `LogOpt` sections document repeatable `--log-opt` mappings.
Tagged [5.4.0](https://github.com/podman-container-tools/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/pkg/systemd/quadlet/quadlet.go#L1880-L1890)
and [6.0.2](https://github.com/podman-container-tools/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/quadlet/quadlet.go#L1937-L1947)
source uses singleton `Lookup` for the driver and reset-aware ordered `LookupAllStrv` for options.
All 20 generators must emit one `--log-driver k8s-file` followed by exactly two ordered post-reset
options and no alternate form. QuadletLens does not reproduce tokenization, parse options as
key/value maps, validate drivers/options, inject defaults, start workloads, or inspect logs.

The isolated container network-identity observation uses endpoint `IP`, `IP6`, and `NetworkAlias`
documentation plus every recorded generator. Each patch must emit one `--ip 192.0.2.40`, one
`--ip6 2001:db8::40`, one `--network bridge`, and exactly two ordered final post-reset
`--network-alias` arguments. The assertion does not compare map-dependent relative ordering
between the network selection and identity flags. It does not validate addresses, aliases, IPAM,
IPv6 enablement, DNS, network names/options, runtime behavior, or cross-format equivalence.

The isolated network driver/options observation uses endpoint `Driver` and `Options` manuals plus
tagged 5.4.0/6.0.2 source. Every recorded generator must emit one `--driver bridge`, clear the
pre-reset options, collapse duplicate `alpha` assignments to the final value, and emit retained
options in sorted key order. The test separately requires 5.4.0 to drop the authored bare token
and 6.0.2 to emit `--opt bare-token`; it neither validates drivers/options nor creates a network.

The isolated network-label observation uses the 5.4.0 combined manual, the 6.0.2 split
`podman-network.unit(5)` manual, and tagged source for reset, tokenization, duplicate selection,
and sorting. Every recorded generator clears pre-reset values, keeps final duplicate keys, sorts
the final keys, preserves `key=` and `key=a=b`, and presents quoted whitespace as one logical
argument. Bare labels are absent through 5.5.2 and emitted once from 5.6.0 onward. This dry-run
observation does not create or inspect a network or establish label/runtime semantics.

The isolated volume-label observation uses the 5.4.0 combined manual, the 6.0.2 split
`podman-volume.unit(5)` manual, and tagged parser/helper source. Every recorded generator clears
pre-reset values, keeps final duplicate keys, sorts final keys, preserves `key=` and `key=a=b`,
and emits quoted whitespace as one logical argument. Bare labels are absent through 5.5.2 and
emitted once from 5.6.0 onward; literal-space presentation is observed in 5.4.x and `\\x20` from
5.5.0 onward. This dry-run observation does not create or inspect a volume or establish
label/runtime semantics.

The isolated network-IPAM observation uses endpoint `IPAMDriver`, `Subnet`, `Gateway`, and
`IPRange` manuals plus tagged 5.4.0/6.0.2 source. Every recorded generator must emit one
`--ipam-driver host-local`, then exactly two ordered post-reset subnet/gateway/range groups. A
separate blank-driver unit must omit the driver flag. Tagged source records no-subnet and
gateway/range-overrun rejection, but the matrix deliberately avoids matching unstable
human-readable diagnostics. It neither validates driver availability/defaults, addresses/ranges,
network creation, provider behavior, or runtime state.

The isolated network-boolean observation uses endpoint `Internal` and `IPv6` manuals plus tagged
5.4.0/6.0.2 source. It distinguishes omission, literal true, and literal false for each key:
omission emits no flag, true emits exactly one plain `--internal` or `--ipv6`, and false emits
exactly one `--internal=false` or `--ipv6=false`. It does not assert relative flag order, select or validate a driver, create a network, infer an
IPv4-enable key, or establish isolation or dual-stack runtime behavior.

The promoted fixtures record the following dry-run expectations across the full matrix:

| Fixture group                          | Required result                                                                                                               |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| DNS, DNS option, DNS search            | Ordered final values after an empty reset                                                                                     |
| IP, IP6, network alias                 | One address flag each and ordered final aliases after reset                                                                   |
| IPAM driver, subnet, gateway, range    | Explicit/blank driver behavior and two indexed final groups after reset                                                       |
| ExposeHostPort                         | Four ordered TCP/UDP-compatible values after reset                                                                            |
| Annotation                             | Two final key-sorted assignments after reset                                                                                  |
| Container completion keys              | Ordered post-reset `--module` and GlobalArgs before `run`; health-log/startup pairs; final `ServiceName` output name          |
| Build Environment                      | Final key-sorted `--env` arguments after reset; 5.6.0 bare-token boundary                                                     |
| Build ContainersConfModule             | Two ordered post-reset `--module=VALUE` arguments before `build`                                                              |
| Build GlobalArgs                       | Ordered post-reset tokens between `podman` and `build`; malformed line omitted                                                |
| Image GlobalArgs                       | Ordered decoded post-reset tokens between `podman` and `image pull`; malformed line omitted                                   |
| Image OS                               | Normal/duplicate-last `--os VALUE`, final-blank omission, and endpoint-specific unmatched-quote presentation                  |
| Build ServiceName                      | Last value, `.service` addition, and 5.7.0/5.8.2 naming boundaries                                                            |
| Pod ServiceName                        | Omitted default, duplicate-last, `.service`, template/quote, blank, and extension-bearing naming observations                 |
| Pod completion keys                    | Post-reset modules/global arguments; final DNS/label/alias values; direct or subordinate maps; final-only PodmanArgs placement |
| Build Volume                           | Reset/continuation `-v` order, relative `.`, and `.volume` substitution/dependency                                            |
| Volume ContainersConfModule            | Ordered post-reset `--module=VALUE` arguments before `volume create`; 5.4 literal-space/5.5 `\\x20` continuation presentation |
| Volume GlobalArgs                      | Decoded post-reset tokens in authored order between `podman` and `volume create`; malformed line omitted                      |
| Volume PodmanArgs                      | Decoded post-reset tokens in authored order at the end of `volume create` before the volume name; malformed line omitted      |
| Volume User                            | Unambiguous `User=123` emits `o=uid=123` before the volume name                                                               |
| Volume Group                           | Unambiguous `Group=456` emits `o=gid=456` before the volume name                                                              |
| Volume GID                             | Rejected through 5.8.5; exactly one `--gid 5678` before the volume name from 6.0.0                                            |
| Volume ServiceName                     | Target last value, `.service`, ordinary/template, and unmatched-quote naming boundaries                                       |
| Volume Image                           | Literal, missing, ignored-driver, and exact image/build-reference observations                                                |
| Image core                             | Literal pull unit, missing/empty errors, and target duplicate-last source selection                                           |
| Image ImageTag                         | Normal/archive source commands plus target-only resource-name, dependency, default, and quote observations                    |
| Image ServiceName                      | Target default, duplicate-last, `.service`, template, and unmatched-quote naming observations                                 |
| Image AllTags                          | Target true/false, duplicate-last, absent/blank, and 5.8.2 unmatched-quote pull-command observations                          |
| Image Arch                             | Target normal, duplicate-last, blank omission, and 5.8.2 unmatched-quote pull-command observations                            |
| Image AuthFile                         | Target normal, duplicate-last, blank omission, and 5.8.2 unmatched-quote pull-command observations                            |
| Image CertDir                          | Target normal, duplicate-last, blank omission, and 5.8.2 unmatched-quote pull-command observations                            |
| Image ContainersConfModule             | Target reset and ordered post-reset `--module` arguments before image pull                                                    |
| AppArmor                               | Rejected through 5.7.1; one separate option from 5.8.0                                                                        |
| NoNewPrivileges and boolean label keys | One option for true; none for false                                                                                           |
| Seccomp and valued label keys          | One isolated separate option per value                                                                                        |
| Mask                                   | One final path-list option after reset                                                                                        |
| Unmask                                 | Ordered `ALL` and path-list options after reset                                                                               |
| Kube                                   | Required YAML source, reset-aware module/global/argument forms, native network dependency, and force-cleanup command text    |
| Artifact                               | 5.7.0 boundary, required final source, reset-aware arguments, oneshot defaults, naming observations, and DefaultDependencies |

The model preserves raw physical values and does not emulate the generator's effective lookup,
sorting, reset, or tokenization rules. These fixtures start no workload and establish no resolver,
OCI, profile, SELinux, path, filesystem, host, runtime, or cross-format behavior. Exact source,
commands, and expected fragments remain recorded in the fixture manifests and capability catalogue.

The memory-limit observation keeps introduction, singleton lookup, and generated command text
separate. The upstream
[`quadlet: support Memory=` change](https://github.com/podman-container-tools/podman/commit/543be25ef35d3127eeea6a34e16e758ad6fd4418)
first ships in Podman 5.5.0 and is absent from the 5.4 tags. Tagged
[5.5.0 command construction](https://github.com/podman-container-tools/podman/blob/v5.5.0/pkg/systemd/quadlet/quadlet.go#L655-L671)
and [6.0.2 command construction](https://github.com/podman-container-tools/podman/blob/v6.0.2/pkg/systemd/quadlet/quadlet.go#L690-L704)
map the last effective singleton value to one `--memory` pair; corresponding tagged parser source
records last-assignment lookup. All three 5.4.x generators reject or exclude the unsupported
fixture without emitting `--memory`. All 17 releases from 5.5.0 through 6.0.2 emit exactly one
`--memory 16777216b` argument and no alternate form. The fixture invokes only the dry-run
generator; it does not start a workload or establish cgroup enforcement, page rounding, swap
interaction, host-memory availability, rootless behavior, runtime inspection, or Compose/BoxFerry
equivalence. Pod `Memory` remains outside this evidence.

`ReloadCmd` and `ReloadSignal` use isolated command and signal units plus a raw conflicting unit.
The three 5.4.x generators reject the unsupported keys. Podman 5.5.x emits cidfile-targeted
`ExecReload` commands, while 5.6.0 through 6.0.2 use the generated container name. The matrix also
records final blank omission, malformed `ReloadCmd` tokenization, and every supported-range conflict.
It invokes only the dry-run generator and establishes no command execution, signal delivery,
container inspection, runtime reload, or cross-format semantics.

Pod `ExitPolicy` uses isolated continue, stop, duplicate-final-stop, and final-blank units. The six
recorded 5.4.0–5.5.2 generators reject the unsupported key; every 5.6.0–6.0.2 generator emits one
`--exit-policy` argument after `--replace`, selecting the final duplicate and retaining an empty
argument for a final blank assignment. This dry-run evidence does not create a pod or establish
policy defaults, restart behavior, runtime behavior, or cross-format equivalence.

Pod `StopTimeout` uses isolated normal-37, zero, negative-one, duplicate-final-37, and final-blank
units. The nine recorded 5.4.0–5.6.2 generators reject the unsupported key; every 5.7.0–6.0.2
generator emits exactly one final `--time=` form for the recorded values, retaining `--time=` for a
final blank assignment. This dry-run evidence does not stop a pod or establish defaults, timing,
systemd, restart, runtime, or cross-format equivalence.

Pod `ServiceName` uses omitted-default, ordinary override, duplicate-final override, template,
unmatched-quote, final-blank, and extension-bearing units. Every recorded 5.4.0–6.0.2 generator
selects the final physical value and appends `.service`; template-default handling changes at 5.7.0
and unmatched-quote lookup changes at 5.8.2. The blank and extension-bearing outputs are retained
as generated text observations only; they do not assign Lens document/dependency identity, operate
systemd, establish restart behavior, or claim runtime or cross-format semantics.

## Commands

Podman is the default local engine:

```shell
cargo ci-generators
QUADLET_LENS_GENERATOR_LANE=full cargo ci-generators
QUADLET_LENS_GENERATOR_VERSION=5.6.2 cargo ci-generators
```

The harness can use Docker where Podman is unavailable:

```shell
QUADLET_LENS_CONTAINER_ENGINE=docker cargo ci-generators
```

The smoke lane tests 5.4.0, the official-image boundary at 5.8.2, and current stable 6.0.2. The full
lane tests the first-conversion, container-logging, container-network-identity, container-reload, network
driver/options, network-labels, volume-labels, network-IPAM, and network-boolean fixtures on all 20 patch releases,
and the separate Memory and reload fixtures on the same three unsupported 5.4.x boundaries plus all 17
supported 5.5.0-through-6.0.2
patches: 14 digest-pinned official images and six exact source builds. It
belongs in the scheduled/manual GitHub workflow rather than pull-request CI.

## Local requirements

Running generator containers requires either `podman` or `docker`; source-backed releases also
require Git. Go itself runs inside the pinned builder and is not a host requirement. Maintaining
the registry matrix benefits from `skopeo` and `jq`, but the Rust harness does not require them. The
current development machine already has Podman 6.0.2, Git, Skopeo, and jq, so no additional
installation is needed.

## Volume `Copy`

The full lane includes 20 isolated physical Copy forms. It records only dry-run command text,
including the 5.8.2 unmatched-quote parser boundary and image-driver suppression; it does not
create volumes, pull images, inspect runtime state, or establish cross-format behavior.
