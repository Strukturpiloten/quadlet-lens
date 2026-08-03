# Podman generator matrix

## Support policy

QuadletLens has a fixed minimum Podman version and a rolling upper target:

- minimum supported version: Podman 5.4.0;
- current upstream target checked on 2026-08-03: Podman 6.0.2;
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

The first-conversion fixture covers registry images including `name:tag@digest`, commands,
environment and systemd specifiers, absolute and unit-relative environment files, repeatable
container and pod host mappings including `host-gateway`, container and pod membership, the
container user/group, user namespace, supplementary groups, working directory, read-only root
filesystem, supported port spellings, native and external networks, named/anonymous/relative and `.volume`
mounts, SELinux mount-option spelling, health commands including `none`, regular health timings,
`Notify=healthy` readiness, generic systemd `Requires`/`Wants`/`After` dependency ordering and
restart behavior, continued `PodmanArgs`, and generated cross-unit dependencies. These are
generator claims; actual activation, failure propagation, rootless/rootful, and SELinux enforcement
remain runtime evidence.

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
lane tests all 20 patch releases: 14 digest-pinned official images and six exact source builds. It
belongs in the scheduled/manual GitHub workflow rather than pull-request CI.

## Local requirements

Running generator containers requires either `podman` or `docker`; source-backed releases also
require Git. Go itself runs inside the pinned builder and is not a host requirement. Maintaining
the registry matrix benefits from `skopeo` and `jq`, but the Rust harness does not require them. The
current development machine already has Podman 5.8.3, Git, Skopeo, and jq, so no additional
installation is needed.
