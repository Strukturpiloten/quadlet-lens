# ADR 0017: Change-aware pull-request validation and Linux evidence

- Status: accepted
- Date: 2026-09-26

## Context

The previous CI repeated the complete deterministic Rust, coverage, dependency, API, and file
checks for every pull request, including changes confined to public prose. A second macOS job
repeated Rust checks even though the maintained local environment and native generator evidence
are Linux-based. Release validation already requires exact-candidate deterministic CI and the
independent pinned native generator matrix.

## Decision

Use a repository-owned, standard-library classifier and policy to select a bounded validation
profile for pull requests. The classifier and policy used for a pull request come from its trusted
base revision. A missing classifier, unexpected path, malformed comparison, or uncertain impact
selects the complete suite. Public prose without executable content may use the file-quality and
offline-link lane plus the lockfile age lane. Documentation with examples retains the complete
suite until an independent focused executable-documentation contract exists. The stable aggregate
gate verifies the exact candidate and every selected job, treating missing, failed, or cancelled
results as failures. Main pushes, manual runs, and workflow calls always run every deterministic
job. Releases additionally run the existing full native generator and application-generator
conformance at the same candidate SHA.

Linux is the verified CI platform. macOS client compatibility remains an intended API property,
not a tested claim; no macOS runner or additional hardware is required. This decision does not
add a Windows support claim. Local complete validation remains the pre-publication gate; narrow
local tasks are development feedback, not substitutes for it.

## Consequences

Prose-only pull requests avoid duplicate Rust and coverage jobs after the trusted classifier has
landed on main. The first rollout pull request runs the complete suite. Any later policy edit
must update the classifier's regression tests and preserve the fail-closed release boundary.

## Alternatives

A path-only `docs/**` skip was rejected because examples, fixtures, policy and machine-readable
documentation can change behavior. A dynamic remote classifier or shared mutable state was
rejected because PR selection must be based on reviewed base code without new network trust.
