# Podman generator matrix

`latest_upstream` in the [generator matrix](../tools/generator-matrix.toml) is Renovate's discovery signal only. `tracked_current`, its exact source evidence, `checked_on` date, and the Nextcloud and Forgejo application contracts advance together only after review and generator validation; discovery never implies support.

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
5. compares fixture-defined generated fragments and, for application contracts, exact structured
   dependency, image, command, and Podman argument values.

The harness intentionally does not snapshot incidental whitespace or the complete generated output.

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

The default smoke lane uses releases marked `smoke = true` in the matrix. The full lane runs every
recorded release. A single-version run is useful while developing a boundary fixture.

The application lane separately validates the repository-owned Nextcloud and Forgejo contracts
with the exact tracked-current generator. BoxFerry can validate one actual exported unit set
without supplying its own oracle:

```console
QUADLET_LENS_APPLICATION_UNIT_DIR=/absolute/path/to/units \
QUADLET_LENS_APPLICATION_CONTRACT=forgejo \
QUADLET_LENS_GENERATOR_VERSION=6.1.0 \
cargo ci-application-generators
```

The supplied directory must contain only the byte-identical reviewed unit files. Relative paths,
symlinks, extra files, changed hashes, unknown contracts, and generator versions other than the
contract target are rejected. The scheduled matrix and release workflow run the built-in
application contracts; neither dry run is runtime evidence.

Podman is the default outer engine. Docker can run the harness when Podman is unavailable:

```console
QUADLET_LENS_CONTAINER_ENGINE=docker cargo ci-generators
```

On GitHub-hosted Docker runners, the Podman system generator re-executes its
native binary and requires a privileged, explicitly AppArmor-unconfined _outer_ container. The scheduled,
manual, and Release reusable workflow sets
`QUADLET_LENS_DOCKER_PRIVILEGED_GENERATORS=true` for generator dry runs and
the exact image-version probe that invokes Podman in the same image. Both
operations receive the bounded privilege and AppArmor exception because either
may need Podman to re-exec. The setting does not apply to source builds,
source-version probes, or any other non-generator command. The outer generator image may still be pulled and
executed; the dry-run boundary means no generated unit or Podman command is
executed, and no generated application image is pulled or workload started.

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
environment-dependent. The scheduled/manual workflow runs the complete recorded matrix. The same
reusable workflow is the Release native-validation lane: it checks out the exact candidate SHA,
runs both the full pinned generator matrix and tracked-current application contracts, and records
current-run, task-labelled evidence. This remains dry-run generator evidence: it does not install,
enable, start, or otherwise execute systemd units or workloads.

See [Testing](testing.md) for tier selection and [Capability model](capability-model.md) for the
claim admitted from generator evidence.
