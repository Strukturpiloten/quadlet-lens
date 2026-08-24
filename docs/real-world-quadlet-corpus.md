# Real-world Quadlet corpus

The real-world corpus checks that QuadletLens can ingest representative public Quadlet deployments
without pretending that a successful parse proves runtime portability.

The authoritative project list, immutable revisions, file digests, licenses, feature markers, and
expectations live in [`fixtures/real-world/corpus.toml`](../fixtures/real-world/corpus.toml).

## Evidence classes

The catalogue labels each source so readers do not treat unlike evidence as equivalent:

| Class                                                        | Meaning                                         |
| ------------------------------------------------------------ | ----------------------------------------------- |
| `upstream-project`                                           | Maintained with the application itself          |
| `vendor-project`, `platform-project`, `distribution-project` | Official integration material                   |
| `vendor-example`, `organization-example`                     | Authored example rather than a production claim |
| `community-deployment`                                       | Third-party deployment or migration evidence    |

## What the test checks

The opt-in test:

- downloads only immutable revisions;
- verifies every selected Git blob;
- checks required feature markers;
- parses every unit and retains byte-exact source;
- canonicalizes and reparses valid syntax;
- builds typed documents and native document-set relationships; and
- reports source-labelled failures.

It does not vendor upstream units, run the Podman generator, install units, pull images, start
containers, load environment files or secrets, or inspect runtime state.

Run it with:

```console
cargo ci-real-world-quadlet
```

Normal pull-request tests remain offline.

## Add or refresh a source

1. Choose a file that exposes missing behavior or protects a known regression.
2. Review the repository, exact revision, file license, and redistribution implications.
3. Record the evidence class, immutable commit, Git blob, required markers, and expected counts.
4. Review every selected file for credentials and private material.
5. Run the corpus test and the deterministic repository gate.
6. Reduce newly discovered behavior to a small repository-owned fixture when practical.

Do not add projects merely to increase corpus size. The corpus is evidence for parser pressure, not
a popularity list or a claim that each application runs unchanged on every supported host.

The shared provenance and secret-review fields are documented in
[Fixture format](fixture-format.md).
