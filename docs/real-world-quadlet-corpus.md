# Real-world Quadlet corpus

The corpus turns public Quadlet deployments into reproducible parser evidence and implementation
goals. Its machine-readable catalogue is
[`fixtures/real-world/corpus.toml`](../fixtures/real-world/corpus.toml). QuadletLens does not vendor
the upstream files: the opt-in test downloads an immutable commit, verifies every Git blob, and
then parses the exact bytes.

Quadlet has a smaller public application ecosystem than Compose. The catalogue therefore records
an evidence class and does not present an official platform unit, an organization example, and an
independent community deployment as equivalent runtime proof.

## Evidence classes

- `upstream-project` is maintained in the application project's own repository.
- `vendor-project`, `platform-project`, and `distribution-project` are official integration units
  but may exercise platform behavior rather than a portable application deployment.
- `vendor-example` and `organization-example` are useful authored examples, not production claims.
- `community-deployment` is third-party migration evidence and may contain assumptions that the
  original application has not reviewed.

## Selected projects

| Tier | Project and evidence class | Why it belongs in the corpus |
| --- | --- | --- |
| Baseline | [Algernon](https://github.com/xyproto/algernon/blob/2d67bdd445cbe4bbd30ae2ab70ca3e9d82ac5574/containers/algernon.container) (`upstream-project`) | Compact container with systemd ordering, installation, mounts, and a published port |
| Baseline | [Bazzite Cockpit](https://github.com/ublue-os/bazzite/blob/1b0f180c9fd4cf6dd9cb74c770b937785aba4eea/system_files/desktop/shared/usr/share/containers/systemd/cockpit-container.container) (`distribution-project`) | Distribution integration using `PodmanArgs`, lifecycle settings, labels, and host mounts |
| Baseline | [IBM HMC agent](https://github.com/IBM/project-pim/blob/9e33beaf34f2788696211dff590d578d0078f962/examples/hmc-agent/hmc_agent.container) (`vendor-example`) | Two related services with environment files, auto-update, networking, and SELinux type settings |
| Baseline | [Red Hat rhproxy](https://github.com/RedHatInsights/rhproxy/blob/5ec1c573b6da1a0430af1a56a3c9caf72f9a236a/config/rhproxy.container) (`vendor-project`) | Packaged service with published/exposed ports, user namespaces, and a pre-start command |
| Application | [Redpanda appstore stack](https://github.com/containers/appstore/blob/6a8826f4bbe4b1a84ff6616c12b5f0a767877b65/quadlet/redpanda/redpanda-server.container) (`organization-example`) | Complete container, network, and volume relationship set with explicit dependency ordering |
| Application | [Immich Podman Quadlets](https://github.com/linux-universe/immich-podman-quadlets/blob/7ea8a1e1f36bd5d5a2f32c1e7fe9ec58c5482b0f/immich-server.container) (`community-deployment`) | Direct Compose-to-Quadlet migration example using a pod, health checks, mounts, and authored variables |
| Application | [Metron](https://github.com/Metron-Project/metron/blob/a97b0bf12a7e35565c5cfcf8420e648c219843ad/.quadlet/metron-web.container) (`upstream-project`) | Full upstream web, proxy, database, cache, network, and volume deployment |
| Stress | [containers/appstore AI stack](https://github.com/containers/appstore/blob/6a8826f4bbe4b1a84ff6616c12b5f0a767877b65/quadlet/ai-stack/ollama.container) (`organization-example`) | Ten-file pod topology with devices, raw arguments, logging, labels, auto-update, and health settings |
| Stress | [containers/qm](https://github.com/containers/qm/blob/bfe94ccd2f87a6a44317d855fa6e716f2c5b7364/qm.container) (`platform-project`) | Large rootfs-backed platform unit with devices, capabilities, security labels, sysctls, and systemd resource controls |
| Stress | [Universal Blue Fedora toolbox](https://github.com/ublue-os/toolboxes/blob/c8809b894cbb6035d5df091db419b7859da9d46d/quadlets/fedora-toolbox/fedora-distrobox-quadlet.container) (`distribution-project`) | Development container with many host mounts, user namespaces, ulimits, annotations, and privileged raw arguments |

Each upstream license is recorded in the catalogue. Downloading a file for a test does not change
its license; copying one into an offline fixture would require a separate redistribution review.

## Verified ingestion result

The pinned corpus passed on 2026-08-05:

| Project | Unit files | Typed-model errors | Native references | Resolved references |
| --- | ---: | ---: | ---: | ---: |
| Algernon | 1 | 0 | 0 | 0 |
| Bazzite Cockpit | 1 | 0 | 0 | 0 |
| IBM HMC agent | 2 | 0 | 0 | 0 |
| Red Hat rhproxy | 1 | 0 | 0 | 0 |
| Redpanda appstore stack | 4 | 0 | 3 | 3 |
| Immich Podman Quadlets | 5 | 0 | 4 | 4 |
| Metron | 9 | 0 | 6 | 6 |
| containers/appstore AI stack | 10 | 0 | 1 | 1 |
| containers/qm | 1 | 0 | 0 | 0 |
| Universal Blue Fedora toolbox | 1 | 0 | 0 | 0 |
| **Total** | **35** | **0** | **14** | **14** |

The 35 files comprise 23 `.container`, two `.pod`, three `.network`, and seven `.volume` units.
Every file is syntactically valid, renders with byte-exact preservation, canonicalizes, and
reparses. The document-set figures cover explicit native references only. Generic systemd service
dependencies and ordinary Podman resource names are retained but are not reinterpreted as native
Quadlet-file edges.

The first run exposed a real typed-model defect: `containers/qm` uses `Rootfs=` as the documented
alternative to `Image=`, while QuadletLens required `Image=` unconditionally. `Rootfs` is now a
typed singleton, empty and conflicting `Image`/`Rootfs` sources produce explicit diagnostics, and
the generator fixture protects its `--rootfs` output from Podman 5.4.0 through 6.0.2.

## Compatibility pressure from the corpus

The corpus confirms the current loss-aware syntax layer is broad enough for these files. It also
makes the next typed promotions concrete:

1. container lifecycle: `AutoUpdate`, `Timezone`, and stop behavior; `Entrypoint`, `RunInit`, and
   explicit `ContainerName` are already typed and generator-verified;
2. logging: `LogDriver` and repeatable `LogOpt`;
3. security and host integration: capabilities, devices, security-label keys, seccomp, sysctls,
   temporary filesystems, shared memory, and ulimits;
4. richer pod/network/volume keys, beginning with pod `ShmSize` and network `Driver`;
5. typed systemd resource controls without pretending QuadletLens can validate host cgroups; and
6. explicit processing rules for authored `${...}` text, environment files, and external runtime
   resource names.

Items in this list are syntax-preserved today. They are not yet all typed, capability-evidenced, or
available to BoxFerry's generator. A successful corpus parse is therefore not a claim that each
upstream deployment will run unchanged on every supported host.

## Test policy

Normal pull-request tests remain offline. A behavior discovered here is reduced to a small authored
fixture before it becomes a required compatibility claim. The opt-in test performs no installation,
generator execution, image pull, container start, environment-file read, or secret access. Run it
with:

```shell
cargo ci-real-world-quadlet
```

An upstream refresh is deliberate: review the new file and license, update the commit and blob
together, adjust the required-section/key inventory, and rerun the offline policy suite and this
network suite.
