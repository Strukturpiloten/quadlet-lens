# Cross-repository implementation plan

This plan gives BoxFerry, ComposeLens, and QuadletLens one stable task numbering scheme. Repository roadmaps describe internal phases; this document describes delivery order across repositories.

Last synchronized: 2026-08-05.

## Status convention

- `planned` — scoped but not started
- `in progress` — implementation is currently active
- `completed` — exit criteria are met and validation is documented
- `blocked` — progress requires a named external decision or capability

The repository that owns a task is authoritative for its detailed status. Update the summary copies in the other two repositories whenever a task changes state.

## Program status

| Task | Owner                                     | Status      | Deliverable                                              |
| ---- | ----------------------------------------- | ----------- | -------------------------------------------------------- |
| T1   | All repositories                          | completed   | Executable testing and fixture foundations               |
| T2   | ComposeLens                               | completed   | Loss-aware YAML syntax and diagnostic kernel             |
| T3   | QuadletLens                               | completed   | Ordered Quadlet syntax and rendering kernel              |
| T4   | BoxFerry                                  | completed   | Independent neutral model and conversion engine          |
| T5   | All repositories                          | in progress | Minimum native typed subsets for the first conversion    |
| T6   | BoxFerry, integrating both Lens libraries | in progress | First Compose-to-Quadlet vertical slice                  |
| T7   | All repositories                          | in progress | Expanded conformance, runtime, and release testing tiers |
| T8   | BoxFerry, integrating all adapters        | in progress | First N-to-N Docker/Compose/Podman/Quadlet milestone     |

## T1: Testing foundations

Status: completed.

The repositories have Cargo-discovered policy tests, versioned fixture manifests, provenance and secret-review rules, immutable GitHub Action checks, stable/MSRV CI execution, and documented suite ownership. Product suites are created only with meaningful behavior.

## T2: ComposeLens YAML syntax kernel

Status: completed. ComposeLens owns this task.

ComposeLens evaluated loss-aware YAML representations, accepted ADR 0002, implemented source and diagnostic primitives, and proved exact preservation and malformed-input recovery on stable Rust and Rust 1.85.0. Its repository copy contains the detailed evidence.

## T3: QuadletLens ordered syntax kernel

Status: completed. QuadletLens owns this task.

Work:

1. Implement ordered sections and entries without collapsing repeated keys.
2. Preserve comments, blank lines, continuations, unknown keys, and generic systemd sections.
3. Preserve `%h` and other systemd specifiers as source values rather than shell substitutions.
4. Add source identifiers, spans, and structured diagnostics.
5. Implement deterministic canonical rendering plus preservation-oriented round trips.
6. Add malformed-input and parser/renderer fixtures.
7. Define the capability schema, establish Podman 5.4 as the minimum, and expand evidence toward rolling current.

Exit criteria:

- Ordered and repeated source constructs survive parsing and rendering.
- Invalid input returns source-spanned diagnostics without panicking.
- Capability entries express minimum/maximum versions, evidence, fallbacks, and known-bug ranges.
- Podman 5.4 minimum claims have tests or explicit evidence gaps, and later targets fail closed until verified.
- Public syntax and diagnostic primitives compile on Rust 1.85.0.

Completion evidence: accepted ADRs 0002–0004; exact preservation and conservative canonical
rendering; source locations and structured recovery; lexical path/specifier classification; a
bounded generated property corpus; the strict versioned TOML catalogue schema; and Podman 5.4.0 as
the support floor. The generator harness now verifies the first-conversion subset on all 22 patch
releases through current 6.1.0, using 14 digest-pinned official images and eight exact source builds.
Capability and boundary tests fail closed below 5.4 and above verified coverage, exercise fallback
and known-bug precedence synthetically, and retain explicit gaps for unverified capabilities.

## T4: BoxFerry independent conversion core

Status: completed. BoxFerry owns this task.

Implement neutral application, service, volume, network, port, environment, and tolerant image-reference models; provenance and redacted diagnostics; exact, approximate, unsupported, and invalid outcomes; target version ranges; adapter contracts; and an in-memory adapter. This task does not depend on unfinished Lens APIs.

BoxFerry has implemented the public library facade, neutral application graph, provenance,
protected values, redacted structured diagnostics, version-bounded target profiles, validated
fidelity outcomes, explicit loss authorization, adapter contracts, and in-memory adapter. Its
stable and Rust 1.85.0 tests exercise the same orchestration available to external Rust projects.
The component crates remain unpublished until their release contract is finalized.

## T5: Minimum native typed subsets

Status: in progress. Each repository owns its native types; BoxFerry owns mappings.

- ComposeLens (completed): services, images, commands, environment, extra hosts, ports, volumes,
  networks, profiles, configs, and secrets.
- QuadletLens (completed): `.container`, `.pod`, `.volume`, `.network`, required generic systemd
  sections, repeatable container/pod host mappings, and exact document-set relationships.
- BoxFerry (in progress): Compose-to-neutral mappings, the first Quadlet exporter, path policies,
  pod grouping, and end-to-end host mappings are implemented; broader value encoders remain.

ComposeLens has completed its T5 native subset with source-aware typed resources, tolerant image
references, deferred values, and representation-preserving command, environment, port, volume,
network, profile, config, secret, and label forms. QuadletLens has completed its first native subset
with ordered source-aware `.container`, `.pod`, `.network`, and `.volume` documents, generic systemd
and unknown entry preservation, native key enums, conservative path/reference forms, separate
syntax/model diagnostics, and exact document-set dependency resolution. BoxFerry consumes
ComposeLens 0.1.2's native merged-project view through its independent `boxferry-compose` crate and
QuadletLens 0.1.2 through `boxferry-quadlet`. Its Compose adapter maps images, commands,
environment, extra hosts, single ports, named volumes, bind mounts, networks, explicit profiles,
provenance, and short/long SELinux relabel intent. Its Quadlet adapter implements deterministic
native documents, explicit pod grouping, caller-supplied bind-source mappings, and native
container/pod host mappings. Source omissions are structured outcomes governed by `LossPolicy`,
not warning-only side effects.

ComposeLens 0.1.2 and QuadletLens 0.1.2 are published on crates.io with documented pre-1.0
compatibility contracts. BoxFerry consumes released Lens crates through compatible crates.io
requirements and commits its application lockfile. Commit-pinned Git dependencies remain an
emergency-only fallback.

Explicit host mappings are complete across merged Compose `extra_hosts` with full provenance, the
BoxFerry neutral model, and repeatable Quadlet `AddHost` keys with exact Podman
5.4.0-through-6.1.0 generator evidence. Separate containers retain service scope; single-pod
grouping requires identical ordered mappings and moves them to pod scope.

### Coverage guardrail, completed health slice, and dependency release gate

The three repositories now document syntax preservation, native typing, effective project views,
neutral representation, target capabilities, and end-to-end conversion as separate coverage
stages. The authoritative cross-format matrix lives in the
[BoxFerry repository](https://github.com/Strukturpiloten/boxferry), with native details in the
ComposeLens and QuadletLens coverage documents. A field is not complete
merely because one Lens recognizes it.

ComposeLens 0.1.3 and QuadletLens 0.1.3 are published and consumed by BoxFerry. The neutral health
model and adapters preserve regular health-check intent and report Compose `start_interval` as an
unsupported non-equivalent target behavior.

ComposeLens 0.1.4 and QuadletLens 0.1.4 are published and consumed by BoxFerry. Ordered neutral
dependency edges retain condition, requirement, restart, and merge provenance. Required and
optional startup edges map to `Requires`/`Wants` plus `After`; healthy edges select
`Notify=healthy` only for explicit encodable target health commands. Unsupported restart and
completion semantics remain policy-controlled losses, while missing required services and cycles
are invalid. Golden tests cover separate containers and explicitly grouped pods.

QuadletLens 0.1.5 is published and consumed by BoxFerry for execution identity and container
context. Typed container `User`, `Group`, `UserNS`, repeatable `GroupAdd`, `WorkingDir`, and
`ReadOnly` keys have capability records and generated-command evidence on every recorded Podman
patch from 5.4.0 through 6.1.0. QuadletLens 0.1.6 adds the distinct pod-level `UserNS` key required
to preserve a compatible namespace choice when BoxFerry explicitly groups services into a pod,
plus repeatable container `Secret` entries for the next config/secret conversion slice; that release
is published. QuadletLens 0.1.7 adds ordered repeatable native container labels with exact generator
evidence across Podman 5.4.0 through 6.1.0. ComposeLens 0.1.9 and QuadletLens 0.1.8 are published
with real-world parser corrections and the first pinned Quadlet corpus. ComposeLens 0.1.10 and
QuadletLens 0.1.9 are the coordinated candidates for an explicit Compose `container_name` and
native Quadlet `ContainerName=` boundary. BoxFerry will consume both published releases before
promoting the field into its neutral model; sibling path dependencies are not used. Using
`Rootfs` end to end still requires an explicit neutral rootfs workload source rather than
substituting it for an image.

QuadletLens now additionally types native singleton container `StopSignal` and `StopTimeout`,
preserves an authored zero timeout, and records exact named/numeric signal and positive/zero
timeout generator observations across its supported Podman 5.4.0-through-6.1.0 range. This is a
Quadlet-native boundary only; runtime zero behavior, cross-format lifecycle mapping, and
zero/default equivalence remain outside this repository.

QuadletLens additionally types native singleton container `Pull`, preserves omission and raw
one-line values, and records isolated `always`, `missing`, `never`, and `newer` generated
`--pull` forms across Podman 5.4.0 through 6.1.0. Registry and local-image runtime behavior remain
outside this generator-only evidence.

QuadletLens additionally types native singleton container `PidsLimit`, preserves omission, zero,
and raw noncanonical values, and provides safe `-1` unlimited or nonzero ASCII-decimal construction
without parsing or integer overflow. Isolated positive and `-1` generated `--pids-limit` forms are
verified from Podman 5.4.0 through 6.1.0; a portable numeric maximum, zero, and runtime cgroup
behavior remain outside the capability evidence.

QuadletLens additionally types native singleton container `HostName`, preserves omission and raw
one-line values without Compose validation or normalization, and verifies one isolated logical
`--hostname app.example` argument from Podman 5.4.0 through 6.1.0. The key requires a private UTS
namespace; for a container joining a pod with default shared UTS, the pod hostname wins. Runtime
hostname inspection, pod hostname typing, and UTS-mode changes remain outside this slice.

QuadletLens additionally types native singleton container and pod `ShmSize`, preserves omission
and raw values, and provides safe non-negative ASCII-decimal construction with optional lowercase
`b`, `k`, `m`, or `g` without parsing or overflow. Explicit zero remains distinguishable as
Podman's documented unlimited IPC-memory value. Positive container, zero container, and pod-owned
generated `--shm-size` forms are verified from Podman 5.4.0 through 6.1.0, including a joined
container that does not duplicate the pod argument. Runtime enforcement, omission defaults,
rootless behavior, IPC-mode keys, pod lifting, and `/dev/shm` inspection remain outside this slice.
Future BoxFerry exact mapping is limited to a positive explicit-byte Compose value on a separate
private-IPC container; that cross-format policy is not implemented in QuadletLens.

QuadletLens additionally types native repeatable container `DropCapability`, preserving omission,
repetition, insertion order, and exact opaque one-line values without splitting, deduplication,
lowercasing, or capability-name validation. The native capability is evidenced exactly from Podman
5.4.0 through 6.1.0. An isolated full-matrix fixture observes four ordered lowercase `--cap-drop`
forms from three entries and no `--cap-add`; tagged source separately records drops before additions.
Rootless/rootful operation, effective bounding sets, user namespaces, SELinux/seccomp interaction,
and runtime privilege outcomes remain explicit evidence gaps.

QuadletLens additionally types native repeatable container `AddCapability`, preserving omission,
empty native reset assignments, duplicates, insertion order, case, and exact opaque one-line text
without splitting, deduplication, lowercasing, or capability validation. Documentation establishes
repeatable space-separated additions beyond Podman's default set; tagged 5.4.0 and 6.0.2 source,
not the Quadlet prose, records `all`, empty resets, lowercasing, drops-before-adds construction, and
capability merger semantics. Across all 22 patches, an isolated fixture emits exactly four ordered
lowercase additions and no drops, while a combined fixture emits one drop-all before one specific
addition and no other capability arguments. Compose `cap_add`, BoxFerry mapping, rootless/rootful
operation, effective and bounding sets, user namespaces, SELinux/seccomp interaction, and runtime
privilege outcomes remain outside this native generator-only slice.

QuadletLens additionally types native repeatable container `Tmpfs`, preserving omission, empty
native reset assignments, duplicates, insertion order, case, options, and exact opaque one-line
text without splitting, normalization, deduplication, mount-option validation, or conflation with
`Volume`. Quadlet documentation establishes the repeatable `CONTAINER-DIR[:OPTIONS]` mapping;
separate Podman CLI documentation records Linux mount flags and the `rw,noexec,nosuid,nodev`
default. Tagged source and all 22 recorded generators establish `LookupAll` post-reset command
construction: the isolated fixture emits exactly one final
`--tmpfs /data:mode=755,uid=1009,gid=1009`, no pre-reset path, and no extra tmpfs form. Target-only
option acceptance, rootless operation, copy-up, mount creation, default enforcement, runtime
inspection, pods, `Volume` tmpfs, Compose, and BoxFerry remain outside this native generator-only
slice.

QuadletLens additionally types native repeatable container `Sysctl`, preserving omission, empty
native reset assignments, duplicates, insertion order, case, whitespace, systemd quoting and
specifiers, and exact opaque one-line values without parsing `name=value`, splitting lists,
normalization, or namespace/runtime validation. Endpoint manuals, Podman-run namespace limits,
tagged 5.4.0/6.0.2 `LookupAllStrv` construction/tokenization/reset source, and all 22 recorded
generators establish exactly one final post-reset `--sysctl net.ipv4.ip_forward=1`, neither
pre-reset setting, and no other sysctl form. Pod `Sysctl`, Compose/BoxFerry mapping, runtime
namespace state, rootless behavior, kernel acceptance, and actual parameter effects remain outside
this native generator-only slice.

QuadletLens additionally types native repeatable container `Ulimit`, preserving omission, empty
native reset assignments, duplicates, insertion order, case, quotes/specifiers, and exact opaque
one-line values without splitting, unquoting, parsing `TYPE=SOFT[:HARD]`, normalization, or
resource-name validation. Distinct endpoint manuals, Podman-run grammar/default caveats, tagged
5.4.0/6.0.2 command and `LookupAll` reset source, and all 22 recorded generators establish exactly
two ordered final post-reset `--ulimit nproc=4096:8192` and `--ulimit stack=-1:-1` arguments, with
neither pre-reset limit nor duplicate, empty, or alternate form. Pod `Ulimit`, Compose/BoxFerry
mapping, runtime enforcement, host inheritance, defaults, cgroups, rootless behavior, and
acceptance of unverified resource names remain outside this native generator-only slice.

QuadletLens additionally types native repeatable container `AddDevice`, preserving omission, every
physical value, empty native resets, duplicates, insertion order, case, quotes/specifiers,
whitespace-token-containing lines, and leading `-` as exact opaque text without splitting,
unquoting, parsing paths/permissions, normalization, or device validation. Endpoint manuals,
Podman-run caveats, tagged 5.4.0/6.0.2 `LookupAllStrv` tokenization/reset and conditional
leading-minus command source, and all 22 recorded generators establish exactly two ordered final
post-reset `--device /dev/null:/dev/final-null:r` and
`--device /dev/zero:/dev/final-zero:w` arguments and exactly two total, with no pre-reset, empty,
or alternate form. The generator fixture contains no leading `-` and runs no workload. Pod
`AddDevice`, Compose/BoxFerry mapping, CDI, runtime access, rootless behavior, SELinux, cgroups,
device existence, and symlink behavior remain outside this native generator-only slice.

QuadletLens additionally types native singleton container `Memory`, preserving omission,
duplicates, empty assignments, quotes/specifiers, and exact opaque one-line values. A focused
helper constructs positive arbitrary-precision ASCII-decimal amounts with optional lowercase
`b`/`k`/`m`/`g` while retaining leading zeros. The capability begins at Podman 5.5.0: all 19
recorded patches through 6.1.0 emit exactly one final `--memory 16777216b`, while all three 5.4.x
generators reject or exclude the unsupported key and emit no memory argument. Runtime cgroup,
page-size, swap, host-memory, rootless, and cross-format behavior remain outside this native slice.

QuadletLens additionally types opaque singleton container `LogDriver` and opaque repeatable,
resettable `LogOpt` at appended `ContainerKey` ordinals 56–57. Physical values, duplicates, order,
empty option resets, quotes, and systemd specifiers remain source-aware without option-map
parsing, validation, or defaults. Endpoint
manuals, tagged 5.4.0/6.0.2 source, and an isolated all-22 generator fixture establish one driver
argument plus ordered post-reset options only; runtime logging, host policy, log inspection, and
cross-format behavior remain outside this native slice.

QuadletLens additionally types opaque singleton container `IP` and `IP6` plus opaque repeatable,
resettable `NetworkAlias` at appended `ContainerKey` ordinals 58–60. Physical values, duplicate
singleton diagnostics, alias duplicates and order, empty resets, quotes, specifiers, and
continuations remain source-aware. An isolated all-22 generator fixture with one `Network=` proves
the two address flags and ordered final post-reset aliases without asserting map-dependent relative
flag ordering or address, IPAM, DNS, network, runtime, or cross-format semantics.

QuadletLens additionally types native network singleton `Driver` and repeatable/resettable
`Options`. The model preserves every physical value and does not validate driver availability or
provider-specific options. Tagged source and the complete generator matrix establish reset,
duplicate-key collapse, sorted output, and the 5.4.0 bare-token drop versus 6.0.2 bare-token
emission as generator evidence rather than model behavior.

QuadletLens additionally types native network singleton `Internal` and `IPv6` as opaque values.
Their source-aware model preserves literal true/false, invalid text, and duplicates without boolean
parsing or Podman's last-value/invalid-as-false lookup. Complete generator evidence distinguishes
omission, true, and false; `Internal` stays driver-conditional and `IPv6` stays a dual-stack
selection, with no IPv4-enable key inferred.

QuadletLens additionally types native network singleton `IPAMDriver` and repeatable/resettable
`Subnet`, `Gateway`, and `IPRange`. It retains physical source values, empties, duplicates,
quotes, specifiers, continuations, and order without applying the generator's resets or indexed
column zipping. Endpoint manuals and tagged 5.4.0/6.0.2 source plus endpoint generator runs prove
one explicit driver, blank-driver omission, and two ordered final groups. IPAM availability/defaults,
network creation, runtime behavior, Compose equivalence, IPv4-disable inference, automatic IPv6
inference, and BoxFerry-owned prefix-complete mapping policy remain outside QuadletLens.

QuadletLens additionally types native repeatable/resettable volume `Label`, retaining every
physical source value and allowing generated empty resets without OCI parsing or target-effective
normalization. Endpoint manuals, tagged parser/helper source, and all 22 generator releases record
reset, last-key collapse, lexical sorting, explicit empty and embedded-equals values, quoted
whitespace presentation, and the bare-token boundary; volume creation, inspection, runtime, and
BoxFerry policy remain outside this native slice.

QuadletLens now types the native networking, annotation, and security keys added in
`ContainerKey` ordinals 41–55. Repeatable values preserve order, duplicates, and resets; singleton
values use the standard duplicate diagnostic. The 20-patch generator matrix covers Podman
5.4.0–6.1.0, with AppArmor explicitly unsupported through 5.7.1 and native from 5.8.0.

This slice establishes native parsing, construction, capability, and dry-run command evidence only.
It does not validate host policy, inspect runtime state, or define Compose/BoxFerry mappings.

## T6: First end-to-end milestone

Status: in progress. BoxFerry coordinates this task. Compose import and the first Quadlet export
are implemented; explicit host mappings, health checks, and dependencies are also complete. Broader compatibility reporting and
the TYPO3 showcase remain.

Deliver tested Compose-to-Quadlet conversion for images, commands, health checks, dependencies, environment, extra hosts,
ports, named volumes, bind mounts, networks, and explicit Compose profile selection. Every
conversion emits compatibility and manual-action reports. After synthetic scenarios are stable,
use `Strukturpiloten/typo3-container` as the first public real-world showcase and regression corpus.

## T7: Expanded testing tiers

Status: in progress. ComposeLens has delivered its repository tier. QuadletLens has an exact
Podman 5.4-to-current generator matrix, and BoxFerry has its first provenance-reviewed Compose
adapter fixture; broader BoxFerry tiers remain.

- Per pull request: unit, integration, golden, round-trip, and property tests.
- Scheduled: Docker Compose, Podman Compose, and real Quadlet generator conformance.
- Release validation: supported Podman matrices, rootless/rootful contexts, real-world projects, and eventually disposable Kubernetes clusters.

Each harness becomes required only after its command, isolation model, version source, fixture provenance, and failure policy are documented.

## T8: First N-to-N runtime and definition milestone

Status: in progress. BoxFerry coordinates this task. Docker runtime resources, Docker Compose,
Podman runtime resources, and Podman Quadlet must each be available as a source and a target.
Routes compose through the neutral application model rather than pair-specific conversion logic.

Exit criteria:

- All four boundaries have importers and exporters for one documented shared semantic subset.
- The CLI explicitly selects every source and target without owning conversion rules.
- All sixteen source/target combinations have offline golden contract tests.
- Runtime targets produce reviewable plans before any explicit apply operation.
- Incompatible intent always produces structured, policy-controlled outcomes.
