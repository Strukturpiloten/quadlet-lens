# Testing

Tests are organized by the claim they protect. Start with the smallest deterministic layer and add
external generator or runtime evidence only when a lower layer cannot establish the contract.

## Test layers

| Layer             | Protects                                                          | Normal pull request        |
| ----------------- | ----------------------------------------------------------------- | -------------------------- |
| Syntax            | Physical parsing, recovery, preservation, and canonical rendering | full or executable docs    |
| Typed model       | Native keys, cardinality, values, diagnostics, and document sets  | full or executable docs    |
| Generation        | Builder validation, deterministic output, and parse-back          | full or executable docs    |
| Capability        | Schema, ranges, evidence, and boundary evaluation                 | full or executable docs    |
| Repository policy | Fixtures, workflows, documentation, and release contracts         | selected by trusted plan   |
| Generator         | Version-pinned Podman dry-run semantics                           | opt-in or scheduled        |
| Real-world corpus | Immutable external source ingestion                               | opt-in                     |
| Runtime           | A named installed environment and behavior                        | only for an explicit claim |

`application-conformance` is an ordinary deterministic layer for immutable application document
sets and independent native semantic expectations. Actual Podman generation remains in the
separate opt-in generator layer.

Pull-request CI uses the immutable shared implementation selected by the automation manifest and
trusted base policy in `scripts/validation-policy.json` to classify the exact candidate. Public
prose may select documentation and dependency eligibility; examples, unknown paths, malformed
comparisons and first-rollout policy retain complete validation. Main, manual and reusable calls
select every deterministic job. The stable `Required CI` verifies selected jobs and planned skips.
Linux is the validated platform; macOS compatibility has no CI evidence lane.

Ordinary adoption retains the existing deterministic CI inventory and complete local gate.
Image qualification and updater activation remain separate work in
[#563](https://github.com/Strukturpiloten/boxferry/issues/563), using fresh GitHub CI evidence with
standard engine storage. Developer VS Code Dev Containers use existing Podman storage, preferably
rootless; this gate adds no privileged local setup. Restored direct definitions and mocked source
checks do not qualify images rebuilt with a changed shared manifest input.

Local and ordinary CI API checks use trusted development Git history. Release, including
`validation_only`, checks the canonical published crate and requires the purpose-bound exact
candidate receipt. The [shared contract suite](../scripts/test-automation-contract.py) covers
policy selection, automatic dependency rules, scoped image proposal holds, age exceptions and failed
or stale receipts. Native launcher tests remain independently required.

Coverage floors are regression alarms, not substitutes for assertions. The project does not require
100% coverage, a fuzzing program, or every host and privilege combination.

Every Release automatically and fail-closed invokes the reusable full pinned generator matrix and
the tracked-current application-generator lane for its exact candidate SHA. `validation_only`
runs the identical gate without publication. This remains dry-run evidence: it does not install,
enable, start, or otherwise execute systemd units, generated Podman commands, or workloads.

## Where tests live

Cargo integration entry points live in [`tests/`](../tests/README.md). Private helpers remain in
`tests/support/`. Fixture suites live below [`fixtures/`](../fixtures/README.md) and follow the
[manifest contract](fixture-format.md).

The current manual-key inventory and capability catalogue have dedicated offline policy tests.
Upstream drift reporting is scheduled or manual and never rewrites reviewed data automatically.

## Test a change

Behavior changes normally include:

1. one successful case;
2. one relevant rejection or recovery case;
3. reset, repetition, ordering, or duplicate coverage where applicable;
4. an external-consumer test for public API changes;
5. a boundary test for version claims; and
6. exact generator evidence when generated command behavior is claimed.

A compatibility correction must show why the earlier claim was wrong or name the evidence that
cannot yet be automated.

## Fixture rules

Every fixture records its purpose, source, version assumptions, expected result, and secret review.
External material additionally records an immutable revision, license, and redistribution decision.

Keep fixtures minimal. A large real-world input can reveal a defect, but the required regression
should normally be reduced to a small authored fixture.

Never add credentials or values that resemble usable credentials. Sensitive-path tests use obvious
canaries and verify repository-owned debug redaction.

## Commands

Run the complete deterministic gate before commit:

```console
./scripts/check-all.sh
```

Useful focused checks are:

```console
./scripts/check-files.sh --check
cargo fmt --all -- --check
cargo ci-check
cargo ci-catalogue
cargo ci-model
cargo ci-application
cargo ci-policy
cargo ci-clippy
cargo ci-test
cargo ci-doctest
RUSTDOCFLAGS="-D warnings" cargo ci-doc
cargo deny check
```

Opt-in evidence tiers are:

```console
cargo ci-generators
QUADLET_LENS_GENERATOR_LANE=full cargo ci-generators
cargo ci-application-generators
cargo ci-real-world-quadlet
```

`check-all.sh` formats repository-owned files before running deterministic Rust, coverage, MSRV,
dependency, package, local-link, workflow, and SemVer checks. Any later edit invalidates the run.

See [Generator matrix](generator-matrix.md) and
[Real-world corpus](real-world-quadlet-corpus.md) before changing those harnesses.
