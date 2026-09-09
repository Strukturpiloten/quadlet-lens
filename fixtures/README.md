# QuadletLens fixtures

Fixtures are stored as `fixtures/<suite>/<id>/`. Every fixture directory contains a `fixture.toml` manifest and all files listed by that manifest.

The common manifest contract is documented in [Fixture format](../docs/fixture-format.md). Executable test entry points live in [`tests/`](../tests/README.md).

Do not add credentials, unreviewed external content, or files with unclear redistribution rights.

The `boxferry-*-application` generator fixtures are bounded exceptions to the usual reduced,
authored generator inputs: they preserve complete application unit sets at immutable revisions so
the test-only generator runner can validate exact structured semantics. Their manifests record
every source hash and distinguish copied units from QuadletLens-authored expectations.

The real-world suite is a catalogue rather than a vendored fixture directory. See the
[real-world Quadlet corpus](../docs/real-world-quadlet-corpus.md) for evidence classes, immutable
download checks, and refresh policy.
