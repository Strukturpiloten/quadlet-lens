# ADR 0020: Podman-only generator outer engine

- Status: accepted
- Date: 2026-10-21
- Supersedes: ADR 0016, Runner isolation only

## Context

The generator harness defaulted to local Podman while its reusable hosted workflow selected
Docker. The maintainer requires Docker tests to use Docker, and Podman/Quadlet tests to use
Podman. A hosted outer engine choice must not be confused with inner generator versions or
evidence that generated workloads ran.

## Decision

Keep local generator execution Podman-only without sudo or fallback. The reusable hosted workflow
uses an explicit rootful local Podman launcher for generation, version probes, source builds and
exact-run cleanup. It verifies the Ubuntu prerequisite and actual rootful/local engine facts,
records actual versions and disables remote mode. Signed Ubuntu APT owns the outer package
integrity as a documented prerequisite exception; exact inner targets and builder pins stay unchanged.

The hosted nested generator and image-version probe opt in to privileged Podman containers so
native re-exec is not blocked by container confinement. Source builds/probes do not receive that
exception. Rootful source builds use the host UID/GID for caller-owned output. Privileged child
timeouts run with the same root authority; cleanup remains exact-label/full-ID and bounded.
Before any prerequisite evidence is written, a closed hosted-only root subreaper exercises the
actual timeout prefix against a fixed TERM-resistant parent/descendant pair. It verifies UID 0,
retained pidfd/start identities, SIGKILL wait statuses and complete reaping before a four-second
observation cutoff, reserving 1.5 seconds for exact cleanup inside the original 5.5-second total.
Failure, cancellation or residual state refuses evidence; cleanup cannot manufacture success.
The fixed worker pair retains the exact controller's pidfd across fork and self-SIGKILL on
controller death, not an elapsed timer, so the outer backstop cannot leave that pair running in
an inner timeout process group. Successful inner-timeout proof precedes cleanup/controller death.
The control has a root
six-second plus two-second kill backstop and never creates Podman/Docker resources or changes
host configuration. Local controls mock this native check; passing mocks are not native evidence.

The 45-minute job allocates setup two minutes, full generation 33, application generation five,
cleanup two and other overhead/reserve three. All eight targets, tracked-current application
semantics, release consumers, exact-candidate evidence and least-privilege workflow permissions
remain required. No native run is inferred from configuration or mocked controls.

## Consequences

All scheduled, manual and release validation shares one outer-engine policy. Docker selection
fails closed rather than silently using a different backend. The generator privilege exception
does not authorize systemd activation, generated-command execution, application image pulls or
workload execution. Fresh hosted Podman generator evidence remains necessary.

## Alternatives

Keeping Docker only in CI contradicts the requested engine policy. An outer-rootless matrix
would add a separate hosted user-namespace prerequisite without expanding generator semantics;
it is not part of this change. Requiring hosted sudo locally would unnecessarily change local
Podman use and was rejected.
