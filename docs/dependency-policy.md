# Dependency policy

Dependencies are part of the library contract. Prefer the standard library and focused crates with
active maintenance, clear licensing, and APIs that preserve Quadlet's ordered and source-aware
representation.

## Rust dependencies

- Use explicit compatible requirements; wildcard requirements are denied.
- Use crates.io releases unless an approved exception requires another source.
- Understand default features before enabling them.
- Avoid overlapping crates for the same concern.
- Commit `Cargo.lock` and use locked resolution in automation.
- Record a dependency that shapes public representation, catalogue data, or version policy in an
  ADR.

The capability catalogue uses Serde for its closed schema and TOML for decoding. They do not
participate in Quadlet syntax parsing or native value interpretation. Exact versions belong in
`Cargo.toml` and `Cargo.lock`.

## Licenses and advisories

[`deny.toml`](../deny.toml) is the source of truth for allowed licenses, advisories, bans, and
sources. Adding a license or source is a distribution decision and requires review of its
obligations.

Do not silence an advisory, allow a Git source, clarify a license, or skip a duplicate merely to
make CI pass. An exception must be narrow, versioned, justified in `deny.toml`, and explained in
the change. Use an ADR for a lasting exception.

Run:

```console
cargo deny --all-features check
```

## Repository tools

Formatting, linting, link checking, workflow analysis, coverage, API compatibility, and release
preparation are development tools rather than published Rust dependencies.

Their exact sources are:

- `package-lock.json` for Node tools;
- the canonical `automation/tool-artifacts.json` inventory at the manifest revision for native tools;
- `.devcontainer/Dockerfile` for container tooling;
- full commit pins and release comments in GitHub workflows; and
- `release-plz.toml` plus the release workflows for release preparation.

The Dev Container and CI run the same repository file checks. Renovate may propose version changes,
but every update receives the normal tests and review.

The manifest-selected common and Rust presets own automatic dependency policy, grouping and
approval behavior. Ordinary third-party releases require 72 hours. Exact maintained first-party
crate/source releases have narrow exceptions; an eligible Lens release waives elapsed age for
its whole update only after the trusted dependency closure check. Prefix lookalikes, other
ecosystems and unrelated sources do not inherit an exception. Future or malformed publication
evidence fails closed. Synthetic lock maintenance still requires the transitive-age guard.

Shared workflow, action-input, manifest and preset revisions advance atomically. The required
eligibility reader does not grant write authority to candidate code. Local custom managers own
native Podman discovery and local image/tool declarations, with one extraction owner per pin.
Discovery stays automatic and grouped separately from fixed reviewed conformance targets.

Final exact file/package rules disable direct image, feature and remaining Dockerfile tool-ARG
proposals until fresh GitHub CI qualification in #563. The cargo-llvm-cov Dockerfile ARG is the
canonical local/CI/image coverage version and retains automatic updates with the ordinary
72-hour age rule; it is the only tool removed from that hold. Its custom-regex update stays
ungrouped: the shared Rust dependency group applies only to Cargo package dependencies. Existing extraction owners remain unique; this is an
`enabled: false` hold, not a dashboard or manual-review path. Package dependencies, compiler and
shared-tool metadata retain their canonical automatic owners even when an existing image COPY
uses them. Advancing those ordinary inputs invalidates previous image evidence without adding
a second authority.

The selected CI caller now requires native image qualification and a fresh
candidate receipt where selected, including the approved old-policy bootstrap. Source wiring
alone supplies no image result and removes no proposal hold. App smoke, updater scheduling and
publication remain separate. Consumer tests exercise merged effective rules and independent
revision drift rather than duplicating shared managers.

## Review checklist

### Hosted outer Podman prerequisite exception

The hosted generator workflow uses Ubuntu 24.04's signed distribution Podman package, installing
it only when absent. Its package revision is not independently pinned or checksum-downloaded:
Ubuntu APT's signed metadata/package hashes and runner maintenance own that host prerequisite.
The launcher fails unless actual engine facts are local/rootful and records the actual package,
engine and OCI runtime versions plus the actual root timeout self-control result in current-run
generator evidence. The self-control uses repository-owned Python standard-library code, not
a downloaded tool or host configuration change. This exception never replaces
the eight reviewed inner image/source pins or the digest-pinned Go builder.

Renovate assessment for #157: no version/digest declaration changes. Existing GitHub Actions
ownership covers unchanged action pins; existing custom managers continue to own discovery and
the builder in `tools/generator-matrix.toml`. The new prerequisite script has no manager-extractable
version pin: Ubuntu package refresh is explicitly manual/runner-owned, not a duplicate Podman
discovery manager. No approval, grouping or automerge rule changes. Runtime behavior still needs
fresh admitted generator evidence after this source change.

The canonical reusable workflow is consumed by scheduled/manual validation and Release, including
`validation_only`; all inherit the same launcher, setup, full matrix, application lane and cleanup.
BoxFerry's downstream application-generator interface remains unchanged; it still supplies exact
reviewed units/contracts to QuadletLens rather than acquiring a BoxFerry dependency. ComposeLens
provider checks and the website do not consume this launcher. DockerLens and PodmanLens own their
separate runtime harnesses and coordinated engine-policy changes, not this reusable workflow.
No other repository's passing gate establishes this change's completion.

### Review a dependency change

Before merging a dependency change:

1. confirm the package and feature set are necessary;
2. inspect licenses, source, maintenance, and security history;
3. review the lockfile and transitive changes;
4. update an ADR if representation or public policy changes; and
5. run the complete repository gate.

The interface-1 loader and thin installer select one checksum-verified shared tool inventory.
CI and Dev Container builds retain actionlint, Lychee, uv and locked Ruff/ty ownership there.
Native Rust tool declarations and the manifest remain image build inputs. Changing a source
or image/tool pin requires fresh complete and native image qualification.
