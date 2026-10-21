# Podman generator matrix

The `latest_upstream` field and each `latest_<major>_<minor>` field in the
[generator matrix](../tools/generator-matrix.toml) are independent Renovate discovery signals
for new Podman minor lines and newer patches on maintained lines. They never imply support.
`tracked_current`, exact source evidence, the `checked_on` date, and the Nextcloud
and Forgejo application contracts advance together only after review and generator validation.

The generator matrix verifies selected Quadlet files against exact released Podman generators. It
is stronger than documentation evidence and narrower than runtime testing.

Exact active releases, immutable image references, source commits, the pinned builder, and
smoke-lane selection live in [`tools/generator-matrix.toml`](../tools/generator-matrix.toml).
Retired patch results and the pre-admission 5.8.7 and 6.1.2 probes live in the separate
[historical evidence ledger](../tools/generator-history.toml). A patch leaves the active full and
Release lanes only after its own durable passing generator result is recorded. [ADR 0018](decisions/0018-active-generator-patches-and-historical-evidence.md)
defines when historical evidence supports a finite capability range.

## How releases run

Where an immutable official Podman image exists, the harness verifies its digest and reported
version before invoking the bundled system generator.

Where an exact image is unavailable, the harness checks out the recorded release commit, verifies
it, and builds only the standalone Quadlet generator in the digest-pinned Go builder. The same
fixture and output checks then run against that binary.

This source-build path verifies the commit, not a cryptographic release-tag signature.

## What a generator test proves

For each selected release, the harness:

1. mounts one authored fixture directory read-only;
2. sets `QUADLET_UNIT_DIRS` to that directory;
3. runs the system generator in dry-run mode;
4. checks success or expected rejection; and
5. compares fixture-defined generated fragments and, for application contracts, exact structured
   dependency, image, command, and Podman argument values.

The harness intentionally does not snapshot incidental whitespace or the complete generated output.

Each outer test container receives a unique run label, including version probes and source builds.
Normal completion and Rust error unwinding remove only containers whose full IDs and exact labels
match that run. The reusable workflow also performs bounded, exact-label cleanup in an
`always()` step after failed or cancelled generator steps. A force-killed runner cannot execute
that step; the printed run label supports explicit, reviewed manual recovery. No global prune or
unrelated image, volume, network, or container cleanup is authorized.

Rust cleanup bounds engine `ps` to 20 seconds and each `inspect` or `rm` to 10 seconds, with a
two-second forced-kill grace. The hosted 45-minute native job allocates two minutes for prerequisite
setup, 33 for full generation, five for application generation, two for `always()` cleanup and
three for checkout/toolchain/evidence handling. A runner killed without a final step still needs
exact-label inspection.

It does not install units, invoke `systemctl`, execute generated Podman commands, pull application
images, or start workloads. Runtime, privilege, cgroup, network, storage, and SELinux behavior
require a separate test with its own environment contract.

## Run a lane

```console
cargo ci-generators
QUADLET_LENS_GENERATOR_LANE=full cargo ci-generators
QUADLET_LENS_GENERATOR_VERSION=5.6.2 cargo ci-generators
cargo ci-application-generators
```

The hash-locked manifests under `fixtures/application-prospective/podman-6.1.2/` retain the
independent Nextcloud and Forgejo expectations authored before 6.1.2 admission. The normal
application lane now validates both reviewed 6.1.2 contracts. Requesting the former prospective
6.1.2 lane is rejected because it is the reviewed target. The exact admission run is recorded in
the [application evidence](../tools/generator-evidence/application-podman-6.1.2.txt).

The default smoke lane uses releases marked `smoke = true` in the active matrix. The full lane
runs every active release, not the historical ledger. A single-version run is useful while
developing a boundary fixture. To rerun one historical pin, set both
`QUADLET_LENS_GENERATOR_HISTORY_VERSION` and `QUADLET_LENS_GENERATOR_VERSION` to its exact
version if it is outside the active matrix; an active patch uses
`QUADLET_LENS_GENERATOR_VERSION` alone. Neither focused selection changes scheduled or Release
selection.

Passed historical records retain the SHA-256 of the executed `tests/generators.rs` harness and
the SHA-256 of the sorted, path-bearing file-hash listing for `fixtures/generators/`. These are
immutable run-time identities, not values to refresh when files later change. Review any later
assertion, fixture, or capability change against affected historical records; rerun the exact
version or narrow the claim instead of rewriting old hashes to match current files. The ledger
records which bridge runs and application contracts remain outstanding before a newer target can
be admitted.
`preliminary-passed` observations retain genuine local probe results but cannot admit a target
without a retrievable reviewed harness snapshot and a final exact-version rerun.

The application lane separately validates the repository-owned Nextcloud and Forgejo contracts
with the exact tracked-current generator. Newer releases remain discovery signals until exact
generator and application evidence is recorded. BoxFerry can validate one actual
exported unit set
without supplying its own oracle:

```console
QUADLET_LENS_APPLICATION_UNIT_DIR=/absolute/path/to/units \
QUADLET_LENS_APPLICATION_CONTRACT=forgejo \
QUADLET_LENS_GENERATOR_VERSION=6.1.2 \
cargo ci-application-generators
```

The supplied directory must contain only the byte-identical reviewed unit files. Relative paths,
symlinks, extra files, changed hashes, unknown contracts, and generator versions other than the
contract target are rejected. The scheduled matrix and release workflow run the built-in
application contracts; neither dry run is runtime evidence.

## Outer Podman prerequisite

Local execution uses Podman only. The default is `podman`; an explicit path to a Podman launcher
is supported, but Docker selection and automatic fallback are rejected. Local checks do not
require sudo or the hosted setup script.

Scheduled, manual and Release runs share the rootful hosted launcher in
[`scripts/hosted-generator-podman.py`](../scripts/hosted-generator-podman.py). It checks the reviewed
Ubuntu runner, installs missing Podman only from its signed distribution APT archive, verifies
local rootful engine facts and records the actual engine, package and OCI runtime versions.
The host package is a documented prerequisite exception, not a pinned inner Podman release;
see [dependency policy](dependency-policy.md). Missing or contradictory prerequisites fail,
never skip or switch to Docker. The same launcher handles exact-label cleanup.

The launcher explicitly disables remote mode. Timeouts run under sudo with the Podman client:
16 seconds for listing, six for inspection/removal, eight for metadata, and two seconds of kill
grace. Listing/removal also reserve two seconds of launcher overhead inside the existing
20/10-second cleanup bounds. Container operations are capped at 30 minutes
in the generator step and four minutes in the application step; overall job/step limits remain
authoritative. Source builds use the calling host UID/GID with rootful outer Podman so their
output and caches remain caller-owned. Local rootless Podman retains its user-namespace mapping.

Nested Podman re-exec during generator dry runs and the exact image-version probe receives
`QUADLET_LENS_PODMAN_PRIVILEGED_GENERATORS=true` in the hosted workflow. Only those operations
receive privileged outer containers; Podman's privileged mode disables AppArmor/seccomp confinement.
Source builds and source-version probes remain unprivileged. This is not a new inner rootful/rootless
runtime matrix. No generated unit or command executes, no generated application image is pulled,
and no workload starts. A successful source/configuration review is not a passing hosted run.

Hosted setup must pass its fixed root timeout self-control before writing evidence; failure,
cancellation or residual state refuses it. Ordinary tests mock sudo. See
[ADR 0020](decisions/0020-podman-only-generator-outer-engine.md) for clock, custody and cleanup.

Source-backed releases also require Git. Go runs inside the pinned builder.

For reviewed version updates and fixture changes, follow the
[generator maintenance guide](generator-maintenance.md). See [Testing](testing.md) for tier
selection and [Capability model](capability-model.md) for the claim admitted from generator
evidence.
