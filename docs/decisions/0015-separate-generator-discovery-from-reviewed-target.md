# ADR 0015: Separate generator discovery from the reviewed target

Status: accepted

Date: 2026-09-21

Supersedes the rolling-target interpretation in ADR 0006.

## Context

The generator matrix previously used `tracked_current` both for Renovate's newest-upstream signal and for the exact Podman version selected by application-generator contracts. A one-field automated update could therefore claim a target for which the source commit, review date, and independent Nextcloud and Forgejo expectations had not been revalidated.

## Decision

`latest_upstream` is the sole Renovate-owned discovery field. It may advance when Renovate observes a newer stable Podman release, but it is not generator or compatibility evidence.

`tracked_current` remains the reviewed generator target. Advancing it requires one reviewed change that records the exact source evidence, updates `checked_on`, updates every application contract target, and passes the generator and application-generator suites. Tests reject a missing discovery signal, a discovery older than the reviewed target, and application contracts that do not match the reviewed target.

## Consequences

Renovate can safely surface upstream releases without silently widening supported behavior. A maintained discovery signal can be newer than the reviewed target, making outstanding validation work explicit. The supported target advances more deliberately, with independently authored application expectations and immutable evidence kept coherent.
