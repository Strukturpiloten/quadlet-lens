# Roadmap

The roadmap is ordered by dependencies rather than dates.

Cross-repository delivery uses the stable task numbers in the [implementation plan](implementation-plan.md). This roadmap remains the detailed internal phase order for QuadletLens.

## Status key

- [x] Completed and validated
- [ ] Open

## Specification coverage ledger

This ledger was audited on 2026-08-07 against the current official
[Podman Quadlet manual](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html).
It records the latest documented surface, not the subset available at the Podman 5.4 minimum.
Each promoted key still needs separate introduction/deprecation/removal evidence over the finite
supported Podman range.

QuadletLens syntax-preserves ordered unknown sections and keys. “Missing” below therefore means
that the key has no native typed enum/builder contract; it does not mean the source text is lost or
that the syntax parser rejects it.

| Section/unit | Current keys | Typed keys | Syntax-preserved only |
| --- | ---: | ---: | ---: |
| `[Container]` / `.container` | 90 | 58 | 32 |
| `[Pod]` / `.pod` | 25 | 7 | 18 |
| `[Network]` / `.network` | 18 | 3 | 15 |
| `[Volume]` / `.volume` | 16 | 4 | 12 |
| `[Build]` / `.build` | 28 | 0 | 28 |
| `[Image]` / `.image` | 18 | 0 | 18 |
| `[Kube]` / `.kube` | 14 | 0 | 14 |
| `[Artifact]` / `.artifact` | 13 | 0 | 13 |
| `[Quadlet]` | 1 | 0 | 1 |

The typed counts describe key recognition and programmatic construction. Capability and generator
evidence are separate layers documented in [Native coverage](coverage.md).

### Missing `[Container]` keys

The following 29 current keys are syntax-preserved but not typed:

`AutoUpdate`, `CgroupsMode`,
`ContainersConfModule`, `EnvironmentHost`,
`GIDMap`, `GlobalArgs`, `HealthLogDestination`,
`HealthMaxLogCount`, `HealthMaxLogSize`, `HealthOnFailure`, `HealthStartupCmd`,
`HealthStartupInterval`, `HealthStartupRetries`, `HealthStartupSuccess`,
`HealthStartupTimeout`, `HttpProxy`, `ImageVolume`, `Mount`,
`ReadOnlyTmpfs`, `ReloadCmd`, `ReloadSignal`, `Retry`, `RetryDelay`,
`ServiceName`, `StartWithPod`,
`SubGIDMap`, `SubUIDMap`, `Timezone`, and `UIDMap`.

The 61 typed keys are `AddHost`, `ContainerName`, `Image`, `Rootfs`, `Entrypoint`, `RunInit`,
`StopSignal`, `StopTimeout`, `Pull`, `PidsLimit`, `HostName`, `ShmSize`, `DropCapability`,
`AddCapability`, `Tmpfs`, `Sysctl`, `Ulimit`, `AddDevice`, `Memory`, `LogDriver`, `LogOpt`, `IP`,
`IP6`, `NetworkAlias`, `DNS`, `DNSOption`, `DNSSearch`, `ExposeHostPort`, `Annotation`, `AppArmor`,
`NoNewPrivileges`, `SeccompProfile`, `SecurityLabelDisable`, `SecurityLabelFileType`,
`SecurityLabelLevel`, `SecurityLabelNested`, `SecurityLabelType`, `Mask`, `Unmask`, `Exec`,
`Environment`, `EnvironmentFile`, `Label`, `Secret`, `User`, `Group`,
`UserNS`, `GroupAdd`, `WorkingDir`, `ReadOnly`, `PublishPort`, `Volume`, `Network`, `Pod`,
`HealthCmd`, `Notify`, `HealthInterval`, `HealthRetries`, `HealthStartPeriod`, `HealthTimeout`, and
`PodmanArgs`.

### Missing `[Pod]` keys

The following 18 current keys are syntax-preserved but not typed:

`ContainersConfModule`, `DNS`, `DNSOption`, `DNSSearch`, `ExitPolicy`, `GIDMap`, `GlobalArgs`,
`HostName`, `IP`, `IP6`, `Label`, `NetworkAlias`, `PodmanArgs`, `ServiceName`,
`StopTimeout`, `SubGIDMap`, `SubUIDMap`, and `UIDMap`.

The typed pod keys are `AddHost`, `PodName`, `PublishPort`, `Network`, `Volume`, `UserNS`, and
`ShmSize`.

### Missing `[Network]` keys

`NetworkName`, `Driver`, `Options`, `Label`, `Internal`, `IPv6`, `IPAMDriver`, `Subnet`, `Gateway`, and
`IPRange` are typed. The following eight current keys are
syntax-preserved but not typed:

`ContainersConfModule`, `DisableDNS`, `DNS`, `GlobalArgs`, `InterfaceName`, `NetworkDeleteOnStop`,
`PodmanArgs`, and `ServiceName`.

### Missing `[Volume]` keys

`VolumeName`, `Driver`, `Options`, `Label`, `Device`, `Type`, and `Copy` are typed. The following 9 current keys are syntax-preserved but not typed:

`ContainersConfModule`, `GID`, `GlobalArgs`, `Group`, `Image`, `PodmanArgs`, `ServiceName`,
`UID`, and `User`.

### Entirely untyped unit sections

The syntax layer preserves these unit files, but their native unit type, section, keys, builders,
relationships, capability records, and generator fixtures are open.

- `[Build]`: `Annotation`, `Arch`, `AuthFile`, `BuildArg`, `ContainersConfModule`, `DNS`,
  `DNSOption`, `DNSSearch`, `Environment`, `File`, `ForceRM`, `GlobalArgs`, `GroupAdd`,
  `IgnoreFile`, `ImageTag`, `Label`, `Network`, `PodmanArgs`, `Pull`, `Retry`, `RetryDelay`,
  `Secret`, `ServiceName`, `SetWorkingDirectory`, `Target`, `TLSVerify`, `Variant`, `Volume`.
- `[Image]`: `AllTags`, `Arch`, `AuthFile`, `CertDir`, `ContainersConfModule`, `Creds`,
  `DecryptionKey`, `GlobalArgs`, `Image`, `ImageTag`, `OS`, `PodmanArgs`, `Policy`, `Retry`,
  `RetryDelay`, `ServiceName`, `TLSVerify`, `Variant`.
- `[Kube]`: `AutoUpdate`, `ConfigMap`, `ContainersConfModule`, `ExitCodePropagation`,
  `GlobalArgs`, `KubeDownForce`, `LogDriver`, `Network`, `PodmanArgs`, `PublishPort`,
  `ServiceName`, `SetWorkingDirectory`, `UserNS`, `Yaml`.
- `[Artifact]` (experimental upstream): `Artifact`, `AuthFile`, `CertDir`,
  `ContainersConfModule`, `Creds`, `DecryptionKey`, `GlobalArgs`, `PodmanArgs`, `Quiet`, `Retry`,
  `RetryDelay`, `ServiceName`, `TLSVerify`.
- `[Quadlet]`: `DefaultDependencies`.

### Generic systemd sections

`[Unit]`, `[Service]`, and `[Install]` remain deliberately open-ended because their complete key
space belongs to systemd, not Quadlet. All directives are syntax-preserved. QuadletLens currently
provides typed construction/evidence for `[Unit]` `Requires`, `Wants`, and `After`, plus
`[Service]` `Restart`.

The current Quadlet manual additionally rewrites Quadlet-to-Quadlet references in `[Unit]`
`Requisite`, `BindsTo`, `PartOf`, `Upholds`, `Conflicts`, and `Before`; those six relationship keys
still need typed construction and graph semantics. Other systemd directives should be promoted
only for a concrete consumer scenario and must retain their native ordering/repetition rules.

## Priority after 0.1.9

### Next 1: lifecycle and process parity

- [x] Type singleton container `Entrypoint` and verify JSON-array argument preservation from Podman
  5.4.0 through 6.0.2.
- [x] Type singleton container `RunInit`, preserve omission and explicit true/false/raw values, and
  verify that true emits one `--init` while false emits one `--init=false` from Podman 5.4.0 through
  6.0.2.
- [x] Type singleton container `StopSignal` and `StopTimeout`, preserve native zero, and verify
  named/numeric signals plus positive/zero timeout generator observations from Podman 5.4.0 through
  6.0.2.
- [x] Type singleton container `Pull`, preserve omission and raw values, and verify isolated
  `always`, `missing`, `never`, and `newer` generator output from Podman 5.4.0 through 6.0.2.
- [x] Type singleton container `PidsLimit`, preserve omission/zero/raw values, add safe typed
  `-1`/nonzero ASCII-decimal construction without parsing, and verify isolated positive/unlimited
  generator output from Podman 5.4.0 through 6.0.2 without claiming runtime cgroup behavior.
- [ ] Type container `Retry` and `RetryDelay` with Podman 5.4-to-current evidence.
- [ ] Type `ServiceName`, `ReloadCmd`, and `ReloadSignal` without confusing Podman resource names,
  Quadlet basenames, and generated systemd unit names.
- [ ] Type pod `ExitPolicy`, `StopTimeout`, and `ServiceName` with explicit restart interactions.

### Next 2: networking and metadata parity

- [x] Type singleton container `HostName`, preserve omission/raw values without Compose
  validation, document private/default/pod-shared UTS behavior, and verify one isolated hostname
  argument from Podman 5.4.0 through 6.0.2 without claiming runtime behavior.
- [ ] Type shared DNS, pod hostname, IP, network-alias, label, and module/global-argument concepts for
  container and pod units where their value grammars actually agree.
- [ ] Complete the `[Network]` key surface, beginning with DNS and delete-on-stop lifecycle.
- [ ] Keep repeatability and cross-field constraints explicit; do not reduce them to raw maps.

### Next 3: security, resources, health, and storage

- [x] Type singleton container and pod `ShmSize`, preserve omission/raw values, add exact native
  non-negative decimal construction with optional `b`/`k`/`m`/`g`, and verify positive, zero, and
  pod-owned generator arguments from Podman 5.4.0 through 6.0.2 without runtime claims.
- [x] Type repeatable container `DropCapability` as opaque ordered native values and verify exact
  lowercase generator arguments from Podman 5.4.0 through 6.0.2 without runtime privilege claims.
- [x] Type repeatable container `AddCapability`, including raw empty resets, and verify isolated
  additions plus drop-all/add-one ordering from Podman 5.4.0 through 6.0.2 without runtime claims.
- [x] Type repeatable container `Tmpfs`, preserve raw empty resets and opaque destination/options
  separately from `Volume`, and verify exact post-reset generator output from Podman 5.4.0 through
  6.0.2 without target-option, rootless, mount, or runtime claims.
- [x] Type repeatable container `Sysctl`, preserve raw empty resets and exact opaque one-line
  entries, and verify endpoint manuals, tagged `LookupAllStrv` construction/tokenization/reset,
  plus exact post-reset generator output from Podman 5.4.0 through 6.0.2 without namespace,
  rootless, kernel, or runtime-effect claims.
- [x] Type repeatable container `Ulimit`, preserve raw empty resets and exact opaque one-line
  entries, and verify endpoint manuals, Podman-run grammar/default caveats, tagged `LookupAll`
  command/reset construction, plus exactly two ordered post-reset generator arguments from Podman
  5.4.0 through 6.0.2 without runtime, host-inheritance, default, cgroup, rootless, or
  unknown-resource-name claims.
- [x] Type repeatable container `AddDevice`, preserve raw empty resets, duplicates, exact physical
  values, whitespace, quotes/specifiers, and leading `-`, and verify endpoint manuals, Podman-run
  caveats, tagged `LookupAllStrv`/conditional/reset construction, plus exactly two ordered final
  post-reset generator arguments from Podman 5.4.0 through 6.0.2 without CDI, runtime-access,
  rootless, SELinux, cgroup, existence, or symlink claims.
- [x] Type singleton container `Memory`, preserve raw values and duplicate diagnostics, add positive
  arbitrary-precision decimal construction, prove 5.4.x rejection/exclusion, and verify exactly one
  explicit-byte argument across all 17 Podman 5.5.0-through-6.0.2 patches without runtime claims.
- [x] Type singleton container `LogDriver` and repeatable/resettable `LogOpt` as opaque physical
  values, and verify one driver plus ordered post-reset options across Podman 5.4.0 through 6.0.2
  without validation, default, runtime, or cross-format claims.
- [x] Type singleton container `IP` and `IP6` plus repeatable/resettable `NetworkAlias` as opaque
  values, and verify address flags plus ordered final aliases with one selected network across
  Podman 5.4.0 through 6.0.2 without address, IPAM, DNS, runtime, or cross-format claims.
- [x] Type singleton network `Driver` and repeatable/resettable `Options` as opaque physical
  values, and verify reset, duplicate-key collapse, sorted final options, and the 5.4.0 versus
  6.0.2 bare-token difference without provider validation, runtime, or cross-format claims.
- [x] Type singleton network `Internal` and `IPv6` as opaque physical values, preserving literal
  true/false and invalid text without boolean parsing; verify omission/true/false generator forms
  across 5.4.0 through 6.0.2 without driver, network-creation, or runtime claims.
- [x] Type singleton `IPAMDriver` and repeatable/resettable `Subnet`, `Gateway`, and `IPRange` as
  opaque physical values; verify blank-driver omission and ordered final indexed groups across
  5.4.0 through 6.0.2 without applying target resets/zipping or making runtime/cross-format claims.
- [x] Type singleton volume `Driver` and raw singleton `Options`; preserve physical source values,
  reject generated duplicates, and record the 5.8.2 quote and 6.0.0 Device-prerequisite generator
  boundaries without driver/plugin, mount, rootless, runtime, Compose, or BoxFerry policy claims.
- [x] Type opaque singleton volume `Device` and `Type`; preserve physical source values, reject
  generated duplicates, and record final blank suppression, Type-without-Device rejection, the
  existing 5.8.2 unmatched-quote boundary, and Type=bind dependency-presentation bands without
  source-path, filesystem, mount, runtime, Compose, or BoxFerry equivalence claims.
- [x] Type repeatable/resettable volume `Label`; preserve every physical source value and record
  reset, duplicate collapse, key sorting, quoted-whitespace presentation, and the bare-token
  boundary without importing generator semantics into the model or builder.
- [x] Type and generator-verify container DNS, exposed-port, and annotation keys across the
  reviewed Podman range.
- [x] Type and generator-verify AppArmor, no-new-privileges, seccomp, and SELinux-label keys.
- [x] Type and generator-verify repeatable Mask and Unmask values with reset evidence.
- [ ] Type remaining capability interactions, SELinux label controls, and UID/GID
  maps.
- [ ] Type image-volume and read-only-tmpfs behavior; extend `Tmpfs` only when a concrete
  target-aware option or runtime contract is defined.
- [ ] Type health logging, failure actions, and the separate startup-health family.
- [ ] Type `Mount` independently from `Volume`; retain their different grammars and defaults.

### Next 4: resource and image lifecycle units

- [ ] Complete `[Volume]` typing and capability evidence.
- [ ] Add `.image` and `.build` native units, references, builders, and exact generator matrices.
- [ ] Add `.kube` only after its file-access and Kubernetes-YAML boundary is explicit.
- [ ] Defer `.artifact` typed support until its experimental contract is stable enough to test
  without presenting a moving target as supported.

### Next 5: version and conformance maintenance

- [ ] Add a maintained manual-key manifest and a policy test that fails when the current closed
  Quadlet key inventory changes without a roadmap classification.
- [ ] Record introduction, deprecation, removal, systemd requirements, and known patch bugs for
  every promoted key.
- [ ] Run promoted keys through the exact Podman generator matrix and relevant rootless/rootful
  runtime fixtures before BoxFerry consumes them.

## Phase 0: foundation — completed

- [x] Accept architecture and origin ADR.
- [x] Prototype and accept the systemd-style syntax representation.
- [x] Scaffold the crate, CI, lints, MSRV policy, and fixture metadata.
- [x] Establish the initial capability schema.
- [x] Define source spans and structured diagnostics.

## Phase 1: syntax and rendering — in progress

- [x] Parse ordered physical sections and entries with source locations.
- [x] Preserve repeated keys, comments, continuation context, line endings, and unknown lines.
- [ ] Preserve quote, newline, and command-argument semantics through real-generator fixtures.
- [x] Distinguish literal paths, relative paths, and native systemd specifiers such as `%h`.
- [x] Implement deterministic canonical rendering.
- [x] Provide validated programmatic construction for the supported native document types.
- [x] Establish malformed-input, round-trip, and property tests.

## Phase 2: typed Quadlet documents — in progress

- [x] Implement the first shared and `.container`, `.pod`, `.network`, and `.volume` unit-specific sections.
- [x] Add conservative path and native unit-reference value forms for the first conversion.
- [x] Model `.container` relationships with `.pod`, `.network`, and `.volume` resources.
- [x] Model regular health-command and timing keys separately from startup/readiness behavior.
- [x] Model generic systemd readiness/ordering dependencies for `Requires`, `Wants`, and `After`,
  plus container `Notify=healthy` readiness.
- [x] Build document sets and dependency graphs.
- [x] Preserve generic systemd sections, repeated entries, and unknown Quadlet keys.

## Phase 3: Podman 5.4 minimum through rolling current — in progress

- [x] Establish the initial capability catalogue for Podman 5.4.
- [x] Run the first-conversion fixture against every official image from Podman 5.4.0 through 5.8.2.
- [x] Build and run exact generators for Podman 5.8.3 through current 6.0.2.
- [ ] Validate the remaining native keys, value forms, fallbacks, and known limitations.
- [x] Cover path handling, pod membership, resource references, regular health commands/timings, restart behavior, and fallback arguments.
- [ ] Run the real-generator fixture suite in rootless and rootful contexts where relevant.

## Phase 4: version expansion — open

- [ ] Complete capability evidence for each later Podman minor version in order.
- [ ] Track introductions, changes, deprecations, removals, and known patch bugs.
- [ ] Add systemd capability checks required by supported Quadlet behavior.
- [ ] Support explicit distribution capability overrides.

## Phase 5: ecosystem hardening — in progress

- [x] Establish the first licensed, immutable real-world corpus across official and community evidence classes.
- [ ] Expand the corpus as new unit types and typed keys are promoted.
- [x] Establish the supported 0.1.x public API and versioned catalogue schema.
- [x] Prepare and validate compatibility documentation and the QuadletLens 0.1.0 release candidate.
- [ ] Provide optional verification tooling for installed Podman generators.

## Maintainer-controlled 0.1.0 release operation — completed

- [x] Publish QuadletLens 0.1.0 from the reviewed clean default-branch commit using the documented
  one-time crates.io bootstrap.
- [x] Revoke the bootstrap token, configure trusted publishing, and run the protected release
  workflow from the same commit to create the tag, attestation, and GitHub release.

## Additive 0.1.1 generation boundary — completed

- [x] Add typed native document construction without exposing parser internals.
- [x] Reject wrong-section keys, duplicate singleton keys, line endings, and NUL bytes.
- [x] Preserve repeatable native and generic systemd entries deterministically.
- [x] Parse and validate every generated document before returning it.
- [x] Add consumer, document-set, rejection-path, MSRV, and documentation coverage.
- [x] Publish QuadletLens 0.1.1 through the protected trusted-publishing workflow.

Follow the exact [release process](releasing.md). BoxFerry can consume the released 0.1.1 API
without a sibling path or Git dependency.

## Additive 0.1.2 host-mapping boundary — completed

- [x] Type repeatable `AddHost` entries in `.container` and `.pod` documents.
- [x] Preserve ordinary addresses and the runtime-specific `host-gateway` value without
  normalization.
- [x] Add finite capability entries for container and pod host mappings.
- [x] Verify generated `--add-host` arguments across every Podman patch release from 5.4.0 through
  current 6.0.2.
- [x] Add release notes and validate the Rust 1.85.0 consumer boundary.
- [x] Publish QuadletLens 0.1.2 through the protected trusted-publishing workflow.

## Additive 0.1.3 regular-health boundary — completed

- [x] Type regular health interval, retries, start period, and timeout keys.
- [x] Add capability records from the Podman 5.4.0 floor through current 6.0.2.
- [x] Verify all four keys against every recorded Podman patch generator in that range.
- [x] Keep Compose `start_interval` distinct from Podman's startup-healthcheck mechanism.
- [x] Add release notes and validate the Rust 1.85.0 consumer boundary.
- [x] Publish QuadletLens 0.1.3 through the protected trusted-publishing workflow.

## Additive 0.1.4 dependency-readiness boundary — completed

- [x] Type the container `Notify` key and evidence the `healthy` readiness form.
- [x] Add typed programmatic construction for `[Unit]` `Requires`, `Wants`, and `After`.
- [x] Add separate capability records for strong, weak, and ordering dependencies.
- [x] Verify readiness and dependency fragments against every recorded Podman patch generator from
  5.4.0 through current 6.0.2.
- [x] Add release notes and validate the Rust 1.85.0 consumer boundary.
- [x] Publish QuadletLens 0.1.4 through the protected trusted-publishing workflow.

## Additive 0.1.5 execution-identity boundary — completed

- [x] Type container `User`, `Group`, `UserNS`, repeatable `GroupAdd`, `WorkingDir`, and `ReadOnly`.
- [x] Add capability records from the Podman 5.4.0 floor through current 6.0.2.
- [x] Verify the generated Podman arguments against every recorded patch generator in that range.
- [x] Add parser, builder, singleton/repetition, public-consumer, and documentation coverage.
- [x] Add release notes and validate the Rust 1.85.0 consumer boundary.
- [x] Publish QuadletLens 0.1.5 through the protected trusted-publishing workflow.

## Additive 0.1.6 pod user-namespace and secret boundary — completed

- [x] Type singleton pod `UserNS` parsing and programmatic construction.
- [x] Add a separate `quadlet.pod.userns` capability record from Podman 5.4.0 through 6.0.2.
- [x] Verify a pod-specific `--userns auto:size=8192` fragment at the support floor, image
  boundary, and current ceiling.
- [x] Add parser, builder, duplicate-singleton, catalogue, and public-consumer coverage.
- [x] Type repeatable container `Secret` parsing and programmatic construction.
- [x] Add `quadlet.container.secret` capability evidence for mounted-file and environment-variable
  option forms from Podman 5.4.0 through 6.0.2.
- [x] Verify both secret forms in the complete real-generator matrix.
- [x] Run the complete 20-patch generator matrix.
- [x] Add release notes and validate the Rust 1.85.0 consumer boundary.
- [x] Publish QuadletLens 0.1.6 through the protected trusted-publishing workflow.

## Additive 0.1.7 container-label boundary — completed

- [x] Type repeatable container `Label` parsing and programmatic construction.
- [x] Add `quadlet.container.label` capability evidence from Podman 5.4.0 through 6.0.2.
- [x] Verify repeated native label arguments in the complete 20-patch generator matrix.
- [x] Append the public key-enum variant without changing published discriminants.
- [x] Add parser, builder, catalogue, public-consumer, and documentation coverage.
- [x] Add release notes and validate the Rust 1.85.0 consumer boundary.
- [x] Publish QuadletLens 0.1.7 through the protected trusted-publishing workflow.

## Additive 0.1.8 real-world corpus and rootfs boundary — completed

- [x] Establish the first license-reviewed, immutable real-world Quadlet corpus.
- [x] Parse and construct `Rootfs` as the mutually exclusive alternative to `Image`.
- [x] Add explicit missing, empty, and conflicting workload-source diagnostics.
- [x] Add `quadlet.container.rootfs` capability evidence from Podman 5.4.0 through 6.0.2.
- [x] Verify `--rootfs` output in the complete 20-patch generator matrix.
- [x] Preserve every published public key-enum discriminant.
- [x] Validate the patch API, Rust 1.85.0 consumer, package, documentation, dependency-policy,
  complete generator-matrix, and real-world-corpus gates.
- [x] Publish QuadletLens 0.1.8 through the protected trusted-publishing workflow.

## Additive 0.1.9 explicit-container-name boundary — completed

- [x] Type singleton `ContainerName` parsing and programmatic construction.
- [x] Add `quadlet.container.container-name` capability evidence from Podman 5.4.0 through 6.0.2.
- [x] Verify exact `--name` output at the support floor, image boundary, and current ceiling.
- [x] Append the public key-enum variant without changing published discriminants.
- [x] Add parser, builder, catalogue, public-consumer, and documentation coverage.
- [x] Run the complete 20-patch generator matrix, real-world corpus, patch API, Rust 1.85.0,
  package, documentation, and dependency-policy release gates.
- [x] Publish QuadletLens 0.1.9 through the protected trusted-publishing workflow.

## Issue-derived evidence

The dated [Podlet regression map](research/podlet-regressions-2026-08-01.md) records concrete
syntax, document-set, capability, and generator cases behind these tasks. Issue closure is not
compatibility evidence; exact Podman/systemd documentation and observations remain required.
