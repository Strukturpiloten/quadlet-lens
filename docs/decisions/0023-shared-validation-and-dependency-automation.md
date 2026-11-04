# ADR 0023: Shared validation and dependency automation

- Status: accepted
- Date: 2026-11-03

## Context

A candidate CI reader cannot trust a dependency eligibility service introduced only by that same
pull request. The trusted workflow must already exist on the default branch before reader adoption.

## Decision

Adopt shared automation in two reviewed stages. This prerequisite introduces the trusted
metadata service, complete merge-queue validation, and the shared loader and checksum-verified
file-tool bootstrap used by CI and the Dev Container. Existing product gates, native generator
selection, age guard, release preparation and publication remain in force. The next stage adopts
shared planning, automatic dependency policy and the read-only receipt waiter after the service is
available on main; this prerequisite does not claim those changes are delivered.

The service reads authoritative GitHub metadata and lockfiles without executing candidate code.
Its repository-scoped token may publish its own eligibility status and rerun an eligible unchanged
read-only candidate workflow. It cannot synthesize `Required CI`; native evidence and full audits
remain separate requirements. Scheduled rechecks preserve first-seen evidence.

Shared workflow references require full lowercase 40-character commit SHAs. The initial
infrastructure uses the literal `automation-v1 interface v1` comment for an exact shared workflow
allowlist; it claims no release tag. The immutable SHA guard remains unconditional, and tests reject
branches, tags, malformed pins and unrelated paths. Two temporary exact-file Renovate managers
separately own the service reference/input and manifest revision, with one owner per pin, until
the common preset replaces them.

## Consequences

The first adoption pull request can complete existing checks before a trusted receipt exists.
Merge queues run every deterministic lane at the tested queue revision. Native and publication
boundaries are unchanged. Subsequent adoption must pin the same service implementation in its
reader, remove the temporary managers, run the complete gate and record independent hosted evidence.

## Alternatives

A wait-only first pull request has no trusted service and cannot pass. Reading candidate-controlled
actor fields or treating an absent receipt as eligibility would weaken the first-party exception.
