# Quality plan

QuadletLens has native typed keys for the complete currently audited Quadlet manual surface. That
completes key recognition, but not every value semantic, Podman/systemd capability claim, or
real-generator behavior.

This plan aims for dependable software that a small project can maintain. The detailed key and
capability ledger remains in the [roadmap](roadmap.md); this document sets priorities and the
quality bar.

## Investment boundary

QuadletLens will use:

- deterministic pull-request checks that finish in a practical amount of time;
- focused positive and negative tests for each changed behavior;
- malformed-input, reset, repetition, document-set, redaction, and public-API tests where relevant;
- representative end-to-end and licensed real-world fixtures; and
- a small required generator smoke lane plus opt-in or scheduled extended evidence runs.

QuadletLens will not require:

- a fuzzing program;
- 100% code coverage;
- every Podman, systemd, operating system, distribution, and privilege combination;
- a large performance or benchmark farm; or
- enterprise-scale release governance.

Coverage floors remain regression alarms, not a goal to execute every line. A useful negative test
is more valuable than extra coverage without a behavioral assertion.

## Planned work

### 1. Prevent manual drift

Completed for the bounded phase-1 key surface: the strict, versioned
[`quadlet-manual-current.toml`](../fixtures/specification-drift/quadlet-manual-current.toml)
inventory records the 223 closed keys in the pinned official aggregate manual, provenance, and a
classification. Offline policy tests reject schema, spelling, ordering, duplicate, section, and
classification drift; scheduled/manual-only automation compares it with upstream and only reports
added or removed rows. It neither runs for pull requests nor writes files or issues.

To update it, download and review the aggregate manual, run
`scripts/check-specification-drift.sh PATH_TO_MANUAL`, then review each difference before
updating the inventory's version, URL, retrieval date, digest, classification, and these coverage
documents. Regenerate deterministic compressed evidence from the exact reviewed manual bytes, and
refresh the corresponding upstream license and any upstream NOTICE material (none exists at the
current pinned tag). A changed prose description or value grammar without a key change is
intentionally not detected by this phase.

### 2. Demand-driven value semantics and diagnostics

Admit a value-form parser, fallback, or cross-field constraint only when a concrete consumer and
immutable evidence define its exact boundary. Preserve raw source when Podman behavior varies.
Diagnostics remain source-aware and distinguish syntax acceptance, generator behavior, and runtime
behavior; this is a maintenance rule, not a standing backlog item.

The completed phase-2 environment slice includes one literal-assignment encoder, a non-empty group
that joins only prevalidated assignments into one physical directive, an explicit blank reset
directive, and a builder-owned ordered plan with per-name effective literal lookup. The plan covers
group order, later-wins duplicates, reset clearing, empty values, opaque effective membership and
cardinality, exact directive emission, and debug redaction. The authored Container view additionally
handles physical order, resets, literal assignments, bare names, documented quote/escape handling,
continuations, and deferred `%` specifiers with recoverable `QLM0023`/`QLM0024` diagnostics.
Recognized Container and Build environment values are redacted from repository-owned debug output,
while explicit raw access and preservation rendering remain available. Environment-file/secret
loading, manager/process/runtime expansion, generic command parsing, and a complete systemd token
grammar remain explicitly deferred until a concrete use case supplies evidence.

### 3. Complete capability evidence

Record introduction, change, deprecation, removal, known patch bugs, and required systemd versions
when new evidence or a consumer changes a supported claim. The optional systemd target context is
complete for the sole reviewed `Upholds=` minimum (249); it is not a generic systemd catalogue,
host probe, or distribution-backport mechanism. Its versioned systemd evidence remains separate
from Podman generator and manual evidence.

### 4. Keep generator conformance focused

Pull requests should run pure tests and a small pinned generator smoke lane covering the minimum,
an important boundary, and the current supported release. The complete recorded generator matrix,
selected rootless/rootful checks, and installed-generator verification remain scheduled or manual.
Runtime tests are added only when dry-run generator output cannot establish the claimed behavior;
installed-generator tooling and distribution overrides remain deferred until a concrete maintainer
use case defines their contract.

### 5. Grow the real-world corpus selectively

Add a licensed, immutable fixture when it exposes a missing behavior or prevents a known
regression. Do not collect units merely to increase corpus size.

### 6. Exercise the public API through BoxFerry

Use BoxFerry as the main downstream contract test. Promote source-aware APIs when conversion needs
them and remove redundant pre-1.0 APIs instead of maintaining compatibility aliases. QuadletLens
must continue to own native parsing, rendering, document relationships, and capability evidence;
cross-format loss policy remains in BoxFerry.

### 7. Stabilize for 1.0

Consider 1.0 when:

- manual drift is automatically detected;
- supported syntax, typed values, document sets, generation, and capability boundaries are clear;
- diagnostics and redaction are stable enough for downstream use;
- representative generator and real-world evidence covers the supported claims; and
- BoxFerry no longer needs internal workarounds for normal Quadlet input and output.

Distribution/backport overrides should be added only after a concrete supported use case defines
their contract; they are not a prerequisite for 1.0 by themselves.

## Test requirement for changes

Every behavior change should normally add one successful case and one relevant rejection or
recovery case. Repeatable or resettable keys also cover ordering, duplicates, and empty resets.
Public API changes add an external-consumer test, and generator claims add a version-boundary or
exact-output assertion. Exceptions should be explained in the change rather than hidden behind a
coverage number.
