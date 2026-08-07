# ADR 0006: Rolling support window and ranged generator evidence

- Status: accepted
- Date: 2026-08-02

## Context

ADR 0004 established a finite, fail-closed Podman 5.4.0 evidence catalogue. That was a useful
bootstrap boundary but could be misread as the product's intended maximum. The actual product goal
is compatibility from a fixed Podman 5.4 minimum through the newest stable upstream release.

A support promise and an evidence claim move at different speeds. Official versioned Podman
containers are available only for part of the target range, and a successful test for one
capability does not prove every Quadlet key or runtime behavior in that release.

## Decision

This ADR amends ADR 0004's initial coverage and exact-single-version evidence clauses while keeping
its strict schema, fail-closed evaluation, capability records, evidence levels, and precedence
rules.

1. Podman 5.4.0 is the minimum support policy, not the maximum.
2. The upper support target follows the newest stable Podman release and records the date on which
   it was checked.
3. Catalogue coverage means finite reviewed evidence coverage. It may temporarily trail the upper
   support target.
4. Generator evidence may cover a finite range only when the harness executes every listed patch
   version in that range.
5. Capability native ranges expand only for the value forms protected by the fixture. Other
   capabilities remain unknown beyond their evidence; they are not automatically unsupported.
6. Official generator images use exact version tags plus manifest digests. Their internal Podman
   binary must report the expected version.
7. The pull-request lane validates the matrix contract without downloading all images. Scheduled
   and manual CI run the full generator matrix.
8. When official images are absent, the project builds only the standalone Quadlet generator from
   the full commit corresponding to an upstream release tag using a version-and-digest-pinned
   builder. The harness verifies the commit and reported version; cryptographic tag-signature
   verification remains a separate supply-chain task.

## Current evidence boundary

The full execution covered all 20 Podman patch releases from 5.4.0 through current 6.0.2: 14
official immutable images through 5.8.2 and six exact source builds thereafter. It verified the
first-conversion container, pod, network, volume, generic systemd, image, command, environment,
environment-file, port, mount, resource-reference, health-command, health timing,
`Notify=healthy` readiness, `Requires`/`Wants`/`After` dependency ordering, restart, and
`PodmanArgs` fragments. It also verifies repeated container labels, container user/group, distinct container and pod user namespaces, repeated
supplementary groups, working directories, read-only root filesystems, and an isolated container
hostname argument. Separate positive and zero container shared-memory fixtures and a pod-owned
shared-memory fixture with a joined container require one matching generated `--shm-size` argument
per owning unit and no duplicate in the joined container. An isolated three-entry
`DropCapability` fixture requires four ordered lowercase `--cap-drop` forms and no `--cap-add` form
in every patch. A separate three-entry `AddCapability` fixture requires four ordered lowercase
`--cap-add` forms and no `--cap-drop`, while a combined fixture requires one `--cap-drop all`
before one `--cap-add cap_net_bind_service` and no other capability argument. Tagged 5.4.0 and
6.0.2 source records empty-list resets, lowercasing, drop-before-add construction, and capability
merger behavior, including `all`; the Quadlet prose documents only repeatable space-separated
additions beyond the default set. These generated-command and source observations are distinct
from runtime privilege behavior. A separate `Tmpfs` fixture contains two pre-reset entries, an
empty reset, and one final `/data:mode=755,uid=1009,gid=1009` entry. Every patch emits exactly one
matching final `--tmpfs` argument, no pre-reset path, and no other tmpfs form. Tagged 5.4.0 and
6.0.2 source maps this key through `LookupAll`, while separate CLI documentation records the Linux
mount-flag option surface and `rw,noexec,nosuid,nodev` omission default. The generator observation
does not validate target-only options, create or inspect a mount, enforce defaults, exercise
copy-up, or establish rootless/runtime behavior.
A separate `Sysctl` fixture contains two pre-reset assignments, an empty reset, and one final
`net.ipv4.ip_forward=1` assignment. Every patch emits exactly one matching final `--sysctl`
argument, neither pre-reset setting, and no other sysctl form. Tagged 5.4.0 and 6.0.2 source records
`LookupAllStrv` command construction, tokenization, and reset behavior; endpoint manuals and
Podman-run documentation record the native list and namespace limitations. The generator does not
execute a container or establish namespace state, rootless behavior, kernel acceptance, runtime
equivalence, or actual sysctl effects.
A separate `Ulimit` fixture contains pre-reset `core=0:0` and `nofile=1024:2048` entries, an empty
reset, then `nproc=4096:8192` and `stack=-1:-1`. Every patch emits exactly those two ordered final
`--ulimit` arguments, with no pre-reset, duplicate, empty, or alternate form. Endpoint manuals and
Podman-run documentation record native repetition, grammar, and default caveats; tagged 5.4.0 and
6.0.2 source maps `Ulimit` through the repeated-string helper using `LookupAll`, not
`LookupAllStrv`, and records empty-assignment resets. The generator does not execute a container or
establish runtime enforcement, host inheritance, defaults, cgroups, rootless behavior, or
acceptance of unverified resource names.
A separate `AddDevice` fixture contains one pre-reset line with two mappings, an empty reset, then
one final line with `/dev/null:/dev/final-null:r` and `/dev/zero:/dev/final-zero:w`. Every patch
emits exactly those two ordered final `--device` arguments and exactly two total, with no pre-reset,
duplicate, empty, or alternate form. Endpoint manuals and Podman-run caveats record native
repetition and target context; tagged 5.4.0 and 6.0.2 source records `LookupAllStrv` tokenization,
empty resets, and conditional leading-minus handling. The fixture deliberately contains no
leading `-`, executes no workload, and establishes no CDI, runtime-access, rootless, SELinux,
cgroup, host-device-existence, or symlink behavior.
A separately mounted `Memory` fixture protects its later introduction without making the existing
Podman 5.4-compatible fixture conditional. The three 5.4.x generators reject or exclude the
unsupported key and emit no memory argument. Every one of the 17 recorded patch releases from
5.5.0 through 6.0.2 applies singleton last-value behavior to earlier duplicate and empty
assignments and emits exactly one final `--memory 16777216b` argument, with no duplicate, equals,
empty, quoted, or alternate form. The fixture invokes no workload and establishes no cgroup,
page-size, swap, host-memory, rootless, runtime-inspection, or cross-format behavior.
The all-20 first-conversion lane now includes DNS, exposed-port, annotation, seccomp,
no-new-privileges, SELinux-label, Mask, and Unmask fixtures. They assert ordered command
construction, reset effects, singleton/boolean handling, and absence of alternate forms. AppArmor
is explicitly rejected through 5.7.1 and accepted from 5.8.0.

The fixtures run only the generator. They do not validate profiles or paths, inspect host state,
start workloads, or establish resolver, OCI, security-policy, filesystem, runtime, or cross-format
behavior.

Referenced `.image`/`.build` units, remaining native keys, runtime, rootless/rootful, and SELinux
semantics retain narrower evidence even inside the generator-covered range.

## Consequences

- Users can distinguish the intended product range from what has actually been verified.
- Common first-conversion capabilities no longer become unknown immediately above Podman 5.4.
- A release-tracking update creates visible work instead of silently broadening capability ranges.
- Historical images consume network and cache space only in scheduled/manual testing.
- The catalogue can express a partially verified release without calling its untested features
  unsupported.

## Alternatives considered

Treating 5.4 as the only supported version was rejected because it contradicts the project goal.
Treating every later version as compatible by default was rejected because known patch regressions
exist. Installing many Podman packages directly on one host was rejected because package archives,
dependencies, configuration, and system state are harder to isolate and reproduce than generator
containers.
