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
5. verifies stable generated service fragments for `.container`, `.pod`, `.network`, and `.volume`
   files.

For source-backed releases it first checks out the recorded commit with Git, builds the standalone
generator using read-only source plus persistent Go caches, and then performs the same version and
fixture checks inside the pinned builder image.

It does not run nested containers, pull the fixture's declared application image, install units,
invoke systemctl, or start generated services. Runtime, rootless/rootful, cgroup, networking, and
SELinux behavior remain separate test tiers.

The first-conversion fixture covers mutually exclusive registry-image and host-rootfs workload
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
isolated singleton fixtures for AppArmor, no-new-privileges, seccomp, and each SELinux-label key,
environment and systemd specifiers, absolute and unit-relative environment files, repeated
container labels, repeated mounted and environment-variable secrets with options, repeatable
container and pod host mappings including `host-gateway`, container and pod membership, the
container user/group and user namespace, the pod's shared user namespace, supplementary groups,
working directory, read-only root filesystem, supported port spellings, native and external
networks, named/anonymous/relative and `.volume` mounts, SELinux mount-option spelling, health
commands including `none`, regular health timings, `Notify=healthy` readiness, generic systemd
`Requires`/`Wants`/`After` dependency ordering and restart behavior, continued `PodmanArgs`,
and generated cross-unit dependencies. These are generator claims; actual activation, failure
propagation, rootless/rootful, and SELinux enforcement remain runtime evidence.

`Memory` uses a separate fixture because the native key was introduced in Podman 5.5.0 and must
not make the existing all-20 first-conversion fixture conditional. It authors an earlier
`Memory=32m`, an empty assignment, and a final explicit-byte `Memory=16777216b`. The full lane runs
that fixture against the three 5.4.x releases to require rejection or exclusion with no memory
argument, then against all 17 recorded patches from 5.5.0 through 6.0.2 to require exactly one
final `--memory 16777216b` argument and no duplicate, equals, empty, quoted, or alternate form.
The smoke lane protects the 5.4.0 unsupported boundary plus 5.8.2 and current 6.0.2 support.

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

- [Podman 5.4.0 command construction](https://github.com/containers/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/pkg/systemd/quadlet/quadlet.go#L742-L751)
  and [Podman 6.0.2 command construction](https://github.com/containers/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/quadlet/quadlet.go#L814-L823)
  lowercase individual capability arguments and append every drop before every addition;
- the [Podman 5.4.0 list parser](https://github.com/containers/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/pkg/systemd/parser/unitfile.go#L760-L805)
  and [Podman 6.0.2 list parser](https://github.com/containers/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/parser/unitfile.go#L769-L814)
  clear earlier repeated values on an empty assignment before splitting later space-separated
  values; and
- the capability merger vendored by [Podman 5.4.0](https://github.com/containers/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/vendor/github.com/containers/common/pkg/capabilities/capabilities.go#L125-L196)
  and [Podman 6.0.2](https://github.com/containers/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/vendor/go.podman.io/common/pkg/capabilities/capabilities.go#L125-L196)
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

Tagged source at [Podman 5.4.0](https://github.com/containers/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/pkg/systemd/quadlet/quadlet.go#L641-L651)
and [Podman 6.0.2](https://github.com/containers/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/quadlet/quadlet.go#L706-L716)
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

Tagged generator source at [5.4.0](https://github.com/containers/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/pkg/systemd/quadlet/quadlet.go#L754-L757)
and [6.0.2](https://github.com/containers/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/quadlet/quadlet.go#L826-L829)
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

Tagged generator source at [5.4.0](https://github.com/containers/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/pkg/systemd/quadlet/quadlet.go#L641-L651)
and [6.0.2](https://github.com/containers/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/quadlet/quadlet.go#L706-L716)
maps `KeyUlimit` to `--ulimit` through the repeated-string helper. The endpoint helpers at
[5.4.0](https://github.com/containers/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/pkg/systemd/quadlet/quadlet.go#L2027-L2034)
and [6.0.2](https://github.com/containers/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/quadlet/quadlet.go#L2091-L2098)
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

Tagged generator source at [5.4.0](https://github.com/containers/podman/blob/f9f7d48b24b1ca4403f189caaeab1cb8ff4a9aa2/pkg/systemd/quadlet/quadlet.go#L724-L734)
and [6.0.2](https://github.com/containers/podman/blob/b28edb9ad70ce4317dc762ee9ce0a6d081d154e9/pkg/systemd/quadlet/quadlet.go#L795-L805)
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

The promoted fixtures record the following dry-run expectations across the full matrix:

| Fixture group | Required result |
| --- | --- |
| DNS, DNS option, DNS search | Ordered final values after an empty reset |
| ExposeHostPort | Four ordered TCP/UDP-compatible values after reset |
| Annotation | Two final key-sorted assignments after reset |
| AppArmor | Rejected through 5.7.1; one separate option from 5.8.0 |
| NoNewPrivileges and boolean label keys | One option for true; none for false |
| Seccomp and valued label keys | One isolated separate option per value |
| Mask | One final path-list option after reset |
| Unmask | Ordered `ALL` and path-list options after reset |

The model preserves raw physical values and does not emulate the generator's effective lookup,
sorting, reset, or tokenization rules. These fixtures start no workload and establish no resolver,
OCI, profile, SELinux, path, filesystem, host, runtime, or cross-format behavior. Exact source,
commands, and expected fragments remain recorded in the fixture manifests and capability catalogue.

The memory-limit observation keeps introduction, singleton lookup, and generated command text
separate. The upstream
[`quadlet: support Memory=` change](https://github.com/containers/podman/commit/543be25ef35d3127eeea6a34e16e758ad6fd4418)
first ships in Podman 5.5.0 and is absent from the 5.4 tags. Tagged
[5.5.0 command construction](https://github.com/containers/podman/blob/v5.5.0/pkg/systemd/quadlet/quadlet.go#L655-L671)
and [6.0.2 command construction](https://github.com/containers/podman/blob/v6.0.2/pkg/systemd/quadlet/quadlet.go#L690-L704)
map the last effective singleton value to one `--memory` pair; corresponding tagged parser source
records last-assignment lookup. All three 5.4.x generators reject or exclude the unsupported
fixture without emitting `--memory`. All 17 releases from 5.5.0 through 6.0.2 emit exactly one
`--memory 16777216b` argument and no alternate form. The fixture invokes only the dry-run
generator; it does not start a workload or establish cgroup enforcement, page rounding, swap
interaction, host-memory availability, rootless behavior, runtime inspection, or Compose/BoxFerry
equivalence. Pod `Memory` remains outside this evidence.

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
lane tests the first-conversion fixture on all 20 patch releases and the separate Memory
fixture on the same three unsupported 5.4.x boundaries plus all 17 supported 5.5.0-through-6.0.2
patches: 14 digest-pinned official images and six exact source builds. It
belongs in the scheduled/manual GitHub workflow rather than pull-request CI.

## Local requirements

Running generator containers requires either `podman` or `docker`; source-backed releases also
require Git. Go itself runs inside the pinned builder and is not a host requirement. Maintaining
the registry matrix benefits from `skopeo` and `jq`, but the Rust harness does not require them. The
current development machine already has Podman 6.0.2, Git, Skopeo, and jq, so no additional
installation is needed.
