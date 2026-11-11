# ADR 0023: Shared validation and dependency automation

- Status: accepted
- Date: 2026-11-03
- Updated: 2026-11-11

## Context

A candidate cannot introduce its own trusted dependency eligibility service. The bootstrap stage
landed the read-only metadata boundary, merge-queue validation and checksum-verified tool loader
before this consumer adoption. Repeated planning, Rust, API, release and dependency policy now
need one reviewed implementation while QuadletLens retains its native evidence and product gates.

## Decision

The [automation manifest](../../.github/automation.json) selects one immutable BoxFerry automation
revision. Shared workflow references, checkout inputs and common/Rust Renovate presets advance
with it in one adoption. Interface-1 references use full lowercase commit SHAs and the exact
`automation-v1 interface v1` comment; this interface marker makes no release-tag claim.

The stable loader delegates local planning, task execution, Cargo tool verification and file/Python
tool installation. Hosted callers use trusted base policy and the exact tested candidate. Missing,
malformed, unknown and first-rollout policy retains the complete suite. Every original Rust,
coverage, API, dependency, documentation and native generator contract remains required for its
selected context. The aggregate rejects missing, failed, cancelled, stale and unplanned skipped jobs.

Ordinary local and PR/main API checks use trusted development Git history. Release and its
`validation_only` path select the canonical published-crate baseline, derive the compatibility
requirement from manifest versions and require a distinct exact-candidate published API receipt.
A generic successful CI receipt cannot replace that evidence.

The typed release policy retains deterministic CI, the complete pinned generator matrix, persisted
native evidence and strict metadata as separate prerequisites. Shared read-only crate preparation
runs before the protected common publisher. Existing publication approval, credentials, tag,
attestation and immutable GitHub release boundaries remain authoritative.

Common presets own automatic updates, grouping and age policy. Exact maintained first-party
releases may waive elapsed age for their whole eligible update after the trusted closure check;
third-party updates retain 72 hours. Operational tool timestamps require their narrow reviewed
source/path contract. Future or malformed publication evidence fails closed. Local managers own
only QuadletLens-native discovery and local image/tool declarations; discovery never admits a
new conformance target.

Ordinary adoption preserved the existing CI inventory and complete deterministic local gate.
The CI qualification stage now appends a native Dev Container job using the selected immutable
source, tested candidate and independently trusted policy. A missing selector runs the job during
the approved old-policy transition; the original trusted plan stays unchanged. Once the image job
is in trusted policy, only an explicit false selector permits a skip. The aggregate requires a
fresh tested-candidate receipt for every successful image job and preserves all prior job checks.

The recipe derives Rust from the candidate toolchain before installing Cargo tools and retains
the selected shared Python tools and caches for verification. The existing feature graph and
conventional nonroot VS Code mounts stay unchanged. There is no second image-only source.

The Rust quality and coverage jobs use typed shared tasks with their existing 30-minute budgets.
Rust quality, coverage and MSRV require explicitly configured trusted tasks before Rust setup;
missing task definitions fail closed rather than reconstructing legacy aliases or defaults.
The explicit MSRV task retains its previous two-command order and 900-second internal budget.
Coverage verifies the candidate Dockerfile cargo-llvm-cov declaration and exact Rust channel
before any local build. Both local coverage commands retain the actual Cargo home and isolated
coverage target, using that verified channel. Missing or invalid metadata or executables fail closed.
The manual feature-qualification caller retains only contents, checks and statuses read so the
immutable reader can inspect existing evidence. Its main-only guard and App secret binding remain
unchanged; this does not dispatch or activate an updater.

Exact late Renovate rules disable only direct image, feature and remaining Dockerfile tool-ARG proposals
pending [#563](https://github.com/Strukturpiloten/boxferry/issues/563). Their extraction owners remain
unique; cargo-llvm-cov now has one canonical Dockerfile ARG for local/CI/image coverage, outside
that hold with ordinary 72-hour eligibility and unchanged ungrouped custom-regex updates.
The shared Rust dependency group remains limited to Cargo package dependencies. Other tool, image, feature
and fixed-target holds remain. Normal package, compiler and shared-tool metadata keep canonical
automatic ownership.
Native amd64 and arm64 qualification runs in GitHub CI with standard hosted Podman storage.
Developer VS Code Dev Containers keep existing Podman storage, preferring rootless operation;
the complete local gate adds no image executor or privileged setup. Source tests and mocked
probes do not establish image readiness. Actual native qualification, App smoke and any later
updater activation require their separate evidence; no updater schedule or publication is enabled.

## Consequences

Libraries have no BoxFerry product dependency. QuadletLens owns fixtures, coverage thresholds,
generator/source/image pins, budgets, cleanup and native tests; the shared project owns policy
implementation and its behavioral regressions. Consumer tests exercise effective presets,
independent drift, failure propagation and exact receipts. Complete repository and native gates
remain necessary before submission or publication.

## Alternatives

Trusting candidate actor claims, accepting absent eligibility evidence, skipping first-rollout jobs,
duplicating shared managers or replacing fixed native pins with discovery values would weaken the
reviewed boundaries.
