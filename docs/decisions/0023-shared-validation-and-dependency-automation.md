# ADR 0023: Shared validation and dependency automation

- Status: accepted
- Date: 2026-11-03
- Updated: 2026-11-06

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

Ordinary adoption retains the existing CI inventory and complete deterministic local gate.
Draft image qualification jobs, their planner selectors and the local image leaf are not active.
The direct image, feature, lock and lifecycle definitions retain their prior main-branch form.
QuadletLens already copies the shared manifest into its image: that existing input advances with
the ordinary automation source, so the image closure changes and needs fresh qualification.
There is no second image-only authority and no claim that source adoption qualifies an image.

Exact late Renovate rules disable only direct image, feature and Dockerfile tool-ARG proposals
pending [#563](https://github.com/Strukturpiloten/boxferry/issues/563). Their extraction owners remain
unique; normal package, compiler and shared-tool metadata keep canonical automatic ownership.
Fresh image qualification belongs in GitHub CI using standard engine storage. Developer VS Code
Dev Containers use existing Podman storage, preferring rootless operation; no custom private
store or mandatory privileged local controls are introduced. Source tests and mocked probes
cannot establish image readiness, and no image build, publication or updater activation is
part of this adoption.

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
