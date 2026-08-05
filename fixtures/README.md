# QuadletLens fixtures

Fixtures are stored as `fixtures/<suite>/<id>/`. Every fixture directory contains a `fixture.toml` manifest and all files listed by that manifest.

The common manifest contract is documented in [Fixture format](../docs/fixture-format.md). Executable test entry points live in [`tests/`](../tests/README.md).

Do not add credentials, unreviewed external content, or files with unclear redistribution rights.

The real-world suite is a catalogue rather than a vendored fixture directory. See the
[real-world Quadlet corpus](../docs/real-world-quadlet-corpus.md) for evidence classes, immutable
download checks, and refresh policy.
