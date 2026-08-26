# QuadletLens test suites

Executable integration-test entry points live directly in this directory so Cargo discovers them. Shared test-only helpers live in `support/` and must not become part of the public library API.

Suites are introduced with the behavior they verify:

- `repository-policy` — fixture metadata and repository security invariants
- `public-api` — supported external Rust consumer path for the current release line
- `syntax` — systemd-style source syntax, malformed input, spans, and recovery
- `typed-model` — native Quadlet unit types, sections, keys, value forms, and document-set graphs
- `environment` — caller-authorized environment-file/secret values and safe generated ordering
- `render` — validated programmatic construction, deterministic output, and rejection paths
- `roundtrip` — preservation and deterministic canonical rendering
- `capabilities` — catalogue schema and coherent version evidence
- `version-boundaries` — behavior immediately around supported version changes
- `generators` — exact released Podman system-generator behavior
- `real-world` — opt-in immutable downloads from the licensed external-project catalogue

Do not add an empty test binary merely to reserve a suite name. Add the entry point, its fixtures, and meaningful assertions together.
