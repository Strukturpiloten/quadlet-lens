# Testing strategy

QuadletLens requires both pure library tests and tests against real released generators. Documentation alone cannot establish parser and generated-command behavior across versions.

## Test layers

### Syntax tests

Cover sections, ordered and repeated keys, comments, blank lines, continuations, quoting, resets, specifiers, malformed lines, Unicode, line endings, and source spans.

### Typed-model tests

Cover every supported unit type, section, key, short/long value form, reference type, unknown entry, and generic systemd section.

### Programmatic-generation tests

Cover typed native-key placement, deterministic section order, repeated entries, singleton
rejection, unsafe physical values, generic systemd directives, parse-back validation, and complete
multi-document relationships for the first conversion subset.

### Round-trip and property tests

Verify that parsing never panics, canonical output is deterministic, preservation edits do not rewrite unrelated entries, and supported typed values survive parse-render-parse cycles.

### Capability-schema tests

Every catalogue entry must:

- conform to the schema
- have a unique stable identifier
- use a coherent version range
- link to evidence or state a documented evidence gap
- identify test coverage
- avoid contradictory native/fallback ranges

### Version-boundary tests

For every introduction, change, deprecation, removal, bug range, or fallback, test the nearest supported version below and above the boundary where available.

### Real-generator tests

Run fixtures through exact Podman system-generator versions from 5.4 through the newest supported release. Capture command, version, environment, exit status, generated service, and diagnostics. Separate syntactic acceptance from successful generated-command behavior.

The implemented container harness records every official immutable Podman patch image from 5.4.0
through 5.8.2 with its manifest digest. For 5.8.3 through current 6.0.2 it builds the standalone
generator from an exact release commit in a digest-pinned Go container. The smoke lane covers the
minimum, the official-image boundary, and current stable; its scheduled/manual full lane covers all
20 patches. It invokes only the dry-run generator. The fixture covers container and pod
relationships, ports, environment-file path forms, repeated container labels, execution identity
and context, networks, image and rootfs workload sources, volume and bind forms, distinct
container/pod user namespaces, health modes, restart behavior, continued raw arguments, and stable
generated dependencies. See the
[matrix documentation](generator-matrix.md).

Systemd-dependent fixtures record the systemd version and rootless/rootful context.

### Real-world fixtures

Fixtures require source provenance, redistribution permission, version assumptions, secret review, and an explanation of the protected behavior.

The implemented opt-in corpus covers 35 immutable unit files from ten public projects across
upstream application, vendor, distribution, platform, organization-example, and community evidence
classes. It verifies Git blobs, required feature markers, syntax validity, byte preservation,
canonical reparsing, typed-model validation, and native document-set references without starting a
generator or workload. See the [real-world corpus](real-world-quadlet-corpus.md).

## Test organization

Cargo-discovered integration tests live in [`../tests/`](../tests/README.md), with private helpers in `tests/support/`. Fixtures live in [`../fixtures/`](../fixtures/README.md) and are validated against the versioned [fixture manifest contract](fixture-format.md). Product suites are added only with implemented behavior and meaningful assertions.

The initial syntax suite protects authored ordering, repeated `After=` entries, `#` comments inside
a continuation, `%h` values, Unicode locations, exact preservation, LF/CRLF and final-line-ending
spelling, and structured recovery for malformed headers, entries before a section, empty keys,
invalid lines, and dangling continuations.

The canonical-rendering suite protects structural normalization without value rewriting and
refuses invalid input. A bounded generated corpus combines line endings, comment markers, quoted
values, image tags plus digests, continuations, and specifiers to prove preservation and canonical
idempotence without introducing a random dependency.

The capability suite parses the embedded strict TOML schema, locks the first-conversion surface
from Podman 5.4.0 through current 6.0.2, checks required/repeated/path/reference metadata, evaluates
the lower and rolling upper coverage boundaries, and uses synthetic evidence to exercise fallback
and known-bug precedence. Capabilities outside the generator fixture retain explicit evidence gaps.

The typed-model suite protects the initial `.container`, `.pod`, `.network`, and `.volume` surface.
It checks native key classification, repeated container/pod `AddHost` and container `Label`/`Secret`
entries, singleton pod `UserNS`, generic systemd
sections, repeated and unknown entries, continuation segments, `%h` and unit-relative paths, native
unit references, explicit supported suffixes, required fields, singleton diagnostics, foreign
native sections, and source labels. Its document-set cases protect exact basename resolution,
resolved dependency edges, retained missing and ambiguous references, duplicate names, unique
source identities, and filename/type matching.

The generation suite constructs `.container`, `.network`, and `.volume` documents, verifies exact
deterministic output, reparses every result, and resolves the generated cross-file graph. It also
protects repeated AddHost, environment, label, secret, and systemd entries and all document-builder rejection
paths.

## Regression rule

Every compatibility fix adds a version-specific regression fixture. Catalogue corrections include a test demonstrating why the earlier claim was wrong or an explicit note explaining why automation is not currently possible.

## Canonical commands

The crate uses Rust 2024 with an MSRV of 1.85.0. `rust-toolchain.toml` pins the normal development toolchain; the explicit MSRV command prevents that pin from hiding accidental use of newer language or library features.

```shell
cargo fmt --all -- --check
cargo ci-check
cargo ci-catalogue
cargo ci-generators
cargo ci-model
cargo ci-policy
cargo ci-real-world-quadlet
cargo ci-clippy
cargo ci-test
cargo ci-doctest
RUSTDOCFLAGS="-D warnings" cargo ci-doc
cargo +1.85.0 ci-check
cargo +1.85.0 ci-policy
cargo deny check
```

The `ci-*` aliases use `--locked`, all workspace features, and all targets where the Cargo command
supports them. CI also runs markdownlint and lychee over the documentation. Add exact property/fuzz
and real-generator matrix commands here before those harnesses become required checks.
