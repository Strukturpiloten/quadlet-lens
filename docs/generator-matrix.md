# Podman generator matrix

The generator matrix verifies selected Quadlet files against exact released Podman generators. It
is stronger than documentation evidence and narrower than runtime testing.

Exact releases, immutable image references, source commits, the pinned builder, and smoke-lane
selection live in [`tools/generator-matrix.toml`](../tools/generator-matrix.toml).

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
5. compares stable generated fragments with the fixture contract.

It does not install units, invoke `systemctl`, execute generated Podman commands, pull application
images, or start workloads. Runtime, privilege, cgroup, network, storage, and SELinux behavior
require a separate test with its own environment contract.

## Run a lane

```console
cargo ci-generators
QUADLET_LENS_GENERATOR_LANE=full cargo ci-generators
QUADLET_LENS_GENERATOR_VERSION=5.6.2 cargo ci-generators
```

The default smoke lane uses releases marked `smoke = true` in the matrix. The full lane runs every
recorded release. A single-version run is useful while developing a boundary fixture.

Podman is the default outer engine. Docker can run the harness when Podman is unavailable:

```console
QUADLET_LENS_CONTAINER_ENGINE=docker cargo ci-generators
```

Source-backed releases also require Git. Go runs inside the pinned builder.

## Add a release

1. Confirm the newest stable upstream release and its exact tag.
2. Prefer an immutable official image and record its manifest digest.
3. Otherwise record the exact release commit and keep the builder pinned.
4. Verify the generator reports the expected version.
5. Run the smoke and full lanes.
6. Expand capability ranges only after reviewing results and evidence gaps.
7. Update Renovate metadata and the checked date in the matrix.

A new upstream version is a tracked target before it is catalogue evidence.

## Add a fixture

Create the smallest fixture that distinguishes the behavior. Its manifest owns provenance,
environment, version selection, and expected fragments. Include rejection boundaries, reset or
repetition behavior, and exact source references where relevant.

Do not duplicate every expected fragment in prose. The fixture manifest and Rust assertion are the
reviewable contract.

Pull requests keep generator execution opt-in because container availability and source builds are
environment-dependent. The scheduled/manual workflow runs the complete recorded matrix.

See [Testing](testing.md) for tier selection and [Capability model](capability-model.md) for the
claim admitted from generator evidence.
