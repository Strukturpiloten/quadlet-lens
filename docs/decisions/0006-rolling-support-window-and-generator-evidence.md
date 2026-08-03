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
`PodmanArgs` fragments. It also verifies container user/group, user namespaces, repeated
supplementary groups, working directories, and read-only root filesystems.

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
