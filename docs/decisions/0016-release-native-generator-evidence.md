# ADR 0016: Release-native generator evidence

Status: accepted

Date: 2026-09-21

## Context

The scheduled generator workflow ran the full pinned Podman generator matrix and application
contracts, while Release validated only one application-generator lane. A release could therefore
be published without fresh complete native generator evidence for its exact candidate revision.
Duplicating the matrix in Release would make scheduled and release behavior drift. A successful
older workflow, partial retry, or an artifact from another run is not release evidence.

## Decision

`generator-matrix.yml` is the one reusable native-validation workflow. Scheduled and manual
diagnostics invoke it directly; Release invokes the same definition with its exact candidate SHA
and a stable release task. The workflow runs `cargo ci-generators` with the full pinned matrix and
`cargo ci-application-generators`, then uploads an overwrite-safe artifact named from the current
run ID and task. Release reads that artifact in the same run and requires schema, candidate SHA,
run ID, task, full-lane, application-lane, and success fields before its fail-closed gate passes.

The deterministic CI workflow is reusable too. Release runs both workflows before the gate. A
failure, timeout, cancellation, missing worker, or unexpected skip produces a non-success need or
missing evidence and blocks publication. `validation_only` runs precisely these validations but
does not schedule the publication job. The final gated job alone has tag, release, attestation, or
trusted-publishing permissions.

## Runner isolation

The GitHub-hosted Docker runner explicitly opts in to a privileged, AppArmor-unconfined outer
container while invoking a Podman generator dry run or exact image-version probe. Podman re-executes
its native binary during either operation; the bounded outer runtime settings supply the isolation
that re-exec requires. The exact image-version probe invokes Podman in that
same image and receives the same bounded outer runtime settings. Neither
exception is available to source builds, source-version probes, or other
non-generator commands. The outer generator image may still be pulled and
executed. The privilege does not authorize installation, systemd activation,
generated-command execution, generated application-image pulls, or application
runtime execution.

## Consequences

Release validation is slower than ordinary CI by design, but it records complete dry-run generator
conformance for the candidate rather than relying on ambient or historical evidence. It does not
claim systemd activation, Podman command execution, container execution, or application runtime
behavior. Artifact overwrite semantics permit GitHub partial retries without stale same-run task
evidence. Workflow action pins remain covered by the existing Renovate workflow manager; no new
version extraction rule is needed.

## Alternatives considered

### Repeat commands in Release

Rejected because scheduled and release native validation would have two independently changing
definitions.

### Treat a successful scheduled run as release evidence

Rejected because its revision, run, and task may differ from the release candidate.

### Give validation jobs release credentials

Rejected because validation-only must be incapable of publication and release permissions should
be confined to the final gated publication job.
