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

| Section/unit                 | Current keys | Typed keys | Syntax-preserved only |
| ---------------------------- | -----------: | ---------: | --------------------: |
| `[Container]` / `.container` |           90 |         63 |                    27 |
| `[Pod]` / `.pod`             |           25 |         10 |                    15 |
| `[Network]` / `.network`     |           18 |         10 |                     8 |
| `[Volume]` / `.volume`       |           16 |         16 |                     0 |
| `[Build]` / `.build`         |           28 |         28 |                     0 |
| `[Image]` / `.image`         |           18 |         12 |                     6 |
| `[Kube]` / `.kube`           |           14 |          0 |                    14 |
| `[Artifact]` / `.artifact`   |           13 |          0 |                    13 |
| `[Quadlet]`                  |            1 |          0 |                     1 |

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
`ReadOnlyTmpfs`, `Retry`, `RetryDelay`,
`ServiceName`, `StartWithPod`,
`SubGIDMap`, `SubUIDMap`, `Timezone`, and `UIDMap`.

The 63 typed keys are `AddHost`, `ContainerName`, `Image`, `Rootfs`, `Entrypoint`, `RunInit`,
`StopSignal`, `StopTimeout`, `Pull`, `PidsLimit`, `HostName`, `ShmSize`, `ReloadCmd`, `ReloadSignal`, `DropCapability`,
`AddCapability`, `Tmpfs`, `Sysctl`, `Ulimit`, `AddDevice`, `Memory`, `LogDriver`, `LogOpt`, `IP`,
`IP6`, `NetworkAlias`, `DNS`, `DNSOption`, `DNSSearch`, `ExposeHostPort`, `Annotation`, `AppArmor`,
`NoNewPrivileges`, `SeccompProfile`, `SecurityLabelDisable`, `SecurityLabelFileType`,
`SecurityLabelLevel`, `SecurityLabelNested`, `SecurityLabelType`, `Mask`, `Unmask`, `Exec`,
`Environment`, `EnvironmentFile`, `Label`, `Secret`, `User`, `Group`,
`UserNS`, `GroupAdd`, `WorkingDir`, `ReadOnly`, `PublishPort`, `Volume`, `Network`, `Pod`,
`HealthCmd`, `Notify`, `HealthInterval`, `HealthRetries`, `HealthStartPeriod`, `HealthTimeout`, and
`PodmanArgs`.

### Missing `[Pod]` keys

The following 15 current keys are syntax-preserved but not typed:

`ContainersConfModule`, `DNS`, `DNSOption`, `DNSSearch`, `GIDMap`, `GlobalArgs`,
`HostName`, `IP`, `IP6`, `Label`, `NetworkAlias`, `PodmanArgs`,
`SubGIDMap`, `SubUIDMap`, and `UIDMap`.

The typed pod keys are `AddHost`, `PodName`, `PublishPort`, `Network`, `Volume`, `UserNS`,
`ShmSize`, `ExitPolicy`, `StopTimeout`, and `ServiceName`.

### Missing `[Network]` keys

`NetworkName`, `Driver`, `Options`, `Label`, `Internal`, `IPv6`, `IPAMDriver`, `Subnet`, `Gateway`, and
`IPRange` are typed. The following eight current keys are
syntax-preserved but not typed:

`ContainersConfModule`, `DisableDNS`, `DNS`, `GlobalArgs`, `InterfaceName`, `NetworkDeleteOnStop`,
`PodmanArgs`, and `ServiceName`.

### Missing `[Volume]` keys

`VolumeName`, `Driver`, `Options`, `Label`, `Device`, `Type`, `Copy`, `ContainersConfModule`, `GlobalArgs`, `PodmanArgs`, `User`, `Group`, `UID`, `GID`, `ServiceName`, and `Image` are typed. No current Volume keys remain syntax-preserved only.

### Unit sections with remaining untyped keys

The syntax layer preserves these unit files, but their native unit type, section, keys, builders,
relationships, capability records, and generator fixtures are open.

- `[Build]`: all current keys are typed. `ImageTag` and repeatable
  opaque `Network`/`Label`/`File`/`BuildArg`/`Secret`/`GroupAdd`/`DNS`/`DNSOption`/`DNSSearch`/`Annotation`/`Environment`/`ContainersConfModule`/`GlobalArgs`/`Volume`/`PodmanArgs` are typed with full 5.4.0-through-6.0.2
  generator evidence; singleton `SetWorkingDirectory`/`Target`/`Arch`/`Variant`/`Pull`/`Retry`/`RetryDelay`/`TLSVerify`/`ForceRM`/`AuthFile`/`IgnoreFile`/`ServiceName` values are also typed;
  exact `.network` Build references resolve in document sets, while effective-last `File` selection
  remains target evidence, not Lens normalization. `Label` retains every physical line without
  parsing, unquoting, duplicate-name selection, map collapse or sorting, or validation; the matrix
  covers only `build.label=one` and `empty=`. `BuildArg` is explicitly unsupported through 5.6.2,
  native from 5.7.0 through 6.0.2, and otherwise unknown; it remains opaque with no assignment,
  environment, secret, bare, or null-value interpretation. `Secret` is native from 5.4.0 through
  6.0.2 and unknown outside that range; it remains opaque without comma, argument, environment,
  path, or secret-data interpretation, and its separate fixture uses placeholder paths only. `Arch`
  and `Variant` preserve raw singleton text without platform parsing, host defaults, or effective-last
  normalization; their separate fixture asserts one `--arch arm64` and one `--variant v8` without
  a relative-order claim. `Pull` preserves opaque raw singleton text without policy validation,
  default selection, normalization, effective-last access, Compose mapping, registry, image-pull, or runtime claims; its separate fixture asserts exactly one `--pull=always` form. `Retry` and `RetryDelay` are unsupported in 5.4.0–5.4.2, native in 5.5.0–6.0.2, and otherwise unknown; their opaque fixture asserts one separate `--retry 4` pair and one separate `--retry-delay 7s` pair before final `.` without a relative-order claim between pairs, and makes no parsing, defaults, effective-last, Compose `dockerfile_inline`, registry, retry/timing, build-success, runtime, or conversion claim. `TLSVerify` is opaque and native in 5.4.0–6.0.2, otherwise unknown; its two-unit fixture asserts one bare `--tls-verify` for true and one `--tls-verify=false` for false before final `.`, without TLS/certificate/registry/pull/build-success/security/provenance/runtime/conversion claims. `ForceRM` is opaque and native in 5.4.0–6.0.2, otherwise unknown; its two-unit fixture asserts one bare `--force-rm` for true and one `--force-rm=false` for false before final `.`, without parsing/default/effective-last, cleanup/failure/execution/default/configuration/cache-equivalence, runtime, or conversion claims. `GroupAdd` is repeatable and native in 5.4.0–6.0.2, otherwise unknown; its fixture asserts ordered separate `--group-add 1234` then `--group-add 5678` pairs before final `.`, without group lookup, keep-groups exclusivity, rootless/user-namespace, runtime, build-execution, Compose privilege-equivalence, or conversion claims. `DNS` is repeatable and native in 5.4.0–6.0.2, otherwise unknown; its fixture asserts ordered separate `--dns 9.9.9.9` then `--dns 2001:4860:4860::8888` pairs before final `.`, without resolver, none-compatibility, resolv.conf, host-DNS, build-execution, Compose endpoint-mapping, or conversion claims.
- `[Image]`: `PodmanArgs`,
  `Policy`, `Retry`,
  `RetryDelay`, `TLSVerify`, `Variant`.
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
- [x] Record the exact generic `PodmanArgs=--interactive` escape-hatch form with boundary manuals,
      tagged source, and one isolated all-20-release generator assertion; retain the existing generic
      repeatable API and claim generated command text only, not runtime stdin, attach, or TTY behavior.
- [x] Record the exact generic `PodmanArgs=--tty` escape-hatch form with boundary manuals, tagged
      source, and one isolated all-20-release generator assertion; retain the existing generic
      repeatable API without a `Tty` key or wrapper, and claim generated command text only, not runtime
      TTY, stdout, stderr, or pipe behavior.
- [x] Record the exact generic `PodmanArgs=--privileged` and `PodmanArgs=--privileged=false`
      escape-hatch forms with endpoint Quadlet manuals, tagged command placement, Podman CLI
      boolean/default evidence, and one isolated all-20-release two-unit generator assertion; retain
      the existing generic repeatable API without a `Privileged` key or wrapper, support only the
      finite 5.4.0-through-6.0.2 range, and claim command text only—not runtime privileges, devices,
      LSM, seccomp, rootless, or cross-format equivalence.
- [x] Type repeatable Build `PodmanArgs` as opaque physical-line text and record exact
      `--build-context extra=container-image://alpine:3.15` command placement through the all-20
      generator matrix, without Compose context lowering, resolution, CLI, build, runtime, or cross-format claims.
- [x] Record exact Build `PodmanArgs=--ssh=default` command placement through the all-20 generator
      matrix with endpoint manuals and tagged source; this non-secret fixture claims no keys, sockets,
      agent, PEM, path, environment, mount, build, runtime, or Compose lowering behavior.
- [x] Record exact Build `PodmanArgs=--shm-size=32m` command placement through the all-20 generator
      matrix with endpoint Quadlet/build manuals and tagged source; it adds no native Build `ShmSize`
      key and claims no Compose/unit equivalence, zero/default, IPC, host/cgroup/memory, build/runtime,
      or conversion behavior.
- [x] Record exact Build `PodmanArgs=--add-host=buildhost:192.0.2.10` command placement through
      the all-20 generator matrix with endpoint Quadlet/build manuals and tagged source; it does not
      lower Compose list/map hosts, establish IPv6/host-gateway/DNS/`/etc/hosts` semantics, resolve
      conflicts/defaults, or claim build, runtime, or conversion behavior.
- [x] Record exact Build `PodmanArgs=--cap-add=CAP_SYS_ADMIN` command placement through the all-20
      generator matrix with endpoint Quadlet/build manuals and tagged source; it does not establish
      Compose entitlement equivalence/conversion, actual capability grants, build execution, or
      LSM/seccomp/rootless/runtime effects.
- [x] Type singleton Build `Retry` and `RetryDelay` as opaque values; record the 5.4.x rejection
      boundary and 5.5.0–6.0.2 generator output without retry/timing, runtime, or conversion claims.
- [x] Type singleton Build `TLSVerify` as an opaque value and verify isolated true/false command
      construction from Podman 5.4.0 through 6.0.2 without TLS, registry, build-success, security,
      runtime, or conversion claims.
- [x] Type singleton Build `ForceRM` as an opaque value and verify isolated true/false command
      construction from Podman 5.4.0 through 6.0.2 without cleanup, failure, execution, default,
      configuration, cache-equivalence, runtime, or conversion claims.
- [x] Type repeatable Build `GroupAdd` as opaque physical-line values and verify ordered separate
      group arguments from Podman 5.4.0 through 6.0.2 without group lookup, keep-groups exclusivity,
      rootless/user-namespace, runtime, build-execution, Compose privilege-equivalence, or conversion claims.
- [x] Type repeatable Build `DNS` as opaque physical-line values and verify ordered separate DNS
      arguments from Podman 5.4.0 through 6.0.2 without resolver, none-compatibility, resolv.conf,
      host-DNS, build-execution, Compose endpoint-mapping, or conversion claims.
- [x] Type repeatable Build `DNSSearch` as opaque physical-line values and verify ordered separate
      post-reset `--dns-search corp.example` then `--dns-search .` arguments from Podman 5.4.0 through
      6.0.2, without model reset or dot semantics, domain removal, DNS/resolver work, build execution,
      Compose mapping, or conversion claims.
- [x] Type singleton Build `AuthFile` as opaque physical-line text and verify one separate path,
      generator-effective-last repeated output, and final-empty omission from Podman 5.4.0 through
      6.0.2, without model normalization, path reads or validation, credential parsing, sensitivity
      classification, authentication, build success, runtime, Compose mapping, or conversion claims.
- [x] Type singleton Build `IgnoreFile` as opaque physical-line text; record 5.4.0–5.6.2
      rejection/exclusion and 5.7.0–6.0.2 one-path, generator-effective-last, and final-empty
      command construction without model normalization, path or ignore-rule interpretation, build
      success, runtime, Compose mapping, or conversion claims.
- [x] Type repeatable Build `Annotation` as opaque physical-line text and record all-20-release
      target reset, tokenization/unquoting/C-unescaping, duplicate-key-collapse, sorting, and 5.6.0
      bare/malformed-token behavior without Lens normalization, OCI/image-metadata, build, runtime,
      Compose mapping, or conversion claims.
- [x] Type repeatable Build `Environment` as opaque physical-line text and record all-20-release
      target reset, tokenization/unquoting/C-unescaping, duplicate-name selection, sorting, and 5.6.0
      bare/malformed-token representation behavior without Lens normalization, host lookup, build,
      runtime, Compose mapping, or conversion claims.
- [x] Type repeatable Build `ContainersConfModule` as opaque physical-line text and record
      all-20-release target reset plus ordered post-reset `--module` command construction without Lens
      path parsing, module reads, configuration inspection, deduplication, normalization, build,
      runtime, Compose mapping, or conversion claims.
- [x] Type repeatable Build `GlobalArgs` as opaque physical-line text and record all-20-release
      target reset, tokenization/unquoting/C-unescaping, malformed-line omission, and ordered placement
      between `podman` and `build` without Lens option validation, security, runtime, build, Compose,
      or conversion interpretation.
- [x] Type Build `ServiceName` as opaque singleton physical-line text, retain raw duplicate source
      diagnostics, and record all-20-release generated-unit default/override, template, and unmatched-quote
      naming observations without assigning document, dependency, runtime, or conversion identity semantics.
- [x] Type repeatable Build `Volume` as opaque physical-line text with only exact source-prefix
      `.volume` references, and record all-20-release reset/continuation, relative-source, and native
      volume substitution/dependency observations without mount, filesystem, runtime, or conversion claims.
- [x] Type repeatable Volume `ContainersConfModule` as opaque physical-line text and record
      all-20-release target reset, continuation presentation, and ordered post-reset `--module` command
      construction before `volume create` without Lens path parsing, module reads, configuration
      inspection, deduplication, normalization, volume creation, lifecycle, filesystem, runtime,
      Compose mapping, or conversion claims.
- [x] Type repeatable Volume `GlobalArgs` as opaque physical-line text and record all-20-release
      target reset, tokenization/unquoting/C-unescaping, malformed-line omission, and ordered
      post-reset token placement before `volume create` without Lens argument parsing, validation,
      security inference, volume creation, lifecycle, filesystem, runtime, Compose mapping, or
      conversion claims.
- [x] Type repeatable Volume `PodmanArgs` as opaque physical-line text and record all-20-release
      target reset, tokenization/unquoting/C-unescaping, malformed-line omission, and ordered terminal
      placement before the volume name without Lens CLI parsing, dedicated-key semantics, security
      inference, volume creation, lifecycle, filesystem, systemd, runtime, Compose mapping, or
      conversion claims.
- [x] Type opaque singleton `ReloadCmd` and `ReloadSignal`, reject their generated mutual-exclusion
      pair, and record the 5.4.x rejection plus 5.5.0–6.0.2 dry-run `ExecReload` boundary without
      command parsing, signal validation, resource-name derivation, or runtime execution.
- [x] Type opaque singleton pod `ExitPolicy` with its explicit 5.4.0–5.5.2 rejection and
      5.6.0–6.0.2 generator boundaries, without policy or restart interpretation.
- [x] Type opaque singleton pod `StopTimeout` with its explicit 5.4.0–5.6.2 rejection and
      5.7.0–6.0.2 generator boundaries, without timeout or restart interpretation.
- [x] Type opaque singleton pod `ServiceName`, preserve source values and duplicate diagnostics, and
      record generated-unit naming observations without assigning restart, identity, or runtime semantics.

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
- `[Image]`: `Image` is an opaque required singleton, while `ImageTag`, `ServiceName`, `AllTags`, `Arch`, `AuthFile`, `CertDir`, `Creds`, `DecryptionKey`, and `OS` are opaque
  singletons and `ContainersConfModule`/`GlobalArgs` are repeatable with 5.4.0-through-6.0.2 generator evidence. `Creds` and `DecryptionKey` are redacted only from repository-owned debug output; all preserve raw source physical entries without target identity, boolean, platform, operating-system, path, certificate, credential, key, authentication, module, configuration, argument, or pull behavior. The remaining six current keys remain
  syntax-preserved only.
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

- [x] Complete `[Volume]` typing and capability evidence.
- [ ] Add remaining `.image` keys beyond the opaque required `Image` source; extend the typed
      `.build` surface only when its untyped future surface expands.
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
