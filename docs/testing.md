# Testing

Tests are organized by the claim they protect. Start with the smallest deterministic layer and add
external generator or runtime evidence only when a lower layer cannot establish the contract.

## Test layers

| Layer             | Protects                                                          | Normal pull request        |
| ----------------- | ----------------------------------------------------------------- | -------------------------- |
| Syntax            | Physical parsing, recovery, preservation, and canonical rendering | yes                        |
| Typed model       | Native keys, cardinality, values, diagnostics, and document sets  | yes                        |
| Generation        | Builder validation, deterministic output, and parse-back          | yes                        |
| Capability        | Schema, ranges, evidence, and boundary evaluation                 | yes                        |
| Repository policy | Fixtures, workflows, documentation, and release contracts         | yes                        |
| Generator         | Exact Podman dry-run output                                       | opt-in or scheduled        |
| Real-world corpus | Immutable external source ingestion                               | opt-in                     |
| Runtime           | A named installed environment and behavior                        | only for an explicit claim |

Coverage floors are regression alarms, not substitutes for assertions. The project does not require
100% coverage, a fuzzing program, or every host and privilege combination.

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
cargo ci-real-world-quadlet
```

`check-all.sh` formats repository-owned files before running deterministic Rust, coverage, MSRV,
dependency, package, local-link, workflow, and SemVer checks. Any later edit invalidates the run.

See [Generator matrix](generator-matrix.md) and
[Real-world corpus](real-world-quadlet-corpus.md) before changing those harnesses.
