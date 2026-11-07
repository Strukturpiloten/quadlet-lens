# Development environment

The supported VS Code setup is the repository Dev Container. Open the checkout and run
**Dev Containers: Reopen in Container**. The default container is unprivileged and supports normal
library, documentation, and repository-policy work.

## Sources of truth

| Concern                               | Source                                   |
| ------------------------------------- | ---------------------------------------- |
| Current Rust toolchain and components | `rust-toolchain.toml`                    |
| Minimum supported Rust                | workspace `rust-version` in `Cargo.toml` |
| Rust dependencies and package version | `Cargo.toml` and `Cargo.lock`            |
| Base image and installed tools        | `.devcontainer/Dockerfile`               |
| Dev Container features                | `.devcontainer/devcontainer-lock.json`   |
| Editor behavior and tasks             | `.vscode/`                               |
| Cargo command definitions             | `.cargo/config.toml`                     |

Do not repeat tool versions in prose. Renovate proposes updates to the checked configuration.

## Daily workflow

Run the complete deterministic gate with:

```console
./scripts/check-all.sh
```

The script formats repository-owned files before checking them. Use focused commands while editing:

```console
./scripts/check-files.sh --check
cargo fmt --all -- --check
cargo ci-check
cargo ci-model
cargo ci-catalogue
cargo ci-policy
cargo ci-test
cargo ci-doc
```

The VS Code complete task runs this gate. Planning and change-aware tasks delegate through
`scripts/shared-ci.py`; they are development feedback. The shared manifest and repository policy
retain one implementation for local and hosted selection, typed Python checks and tool ownership.

SemVer preflight verifies the canonical compiler, exact binary and isolated Cargo home/target
before formatting or tests. Local API validation uses trusted development history. Release uses
the published crate and its own purpose-bound receipt. Shared automation selects the locked
Python tooling for these checks.

Use VS Code Dev Containers with the existing Podman setup and storage; prefer rootless Podman.
Ordinary local validation does not build or qualify an image and requires no new privileged
setup. The recipe selects the candidate Rust toolchain and retains canonical shared Python tools
for verification. Its manifest follows the same immutable automation source as CI; changes to
copied inputs invalidate previous image evidence. Existing nonroot workspace and volume mounts
and the official feature lock remain intact.

CI now qualifies selected candidates on native amd64 and arm64 hosted Podman lanes. A missing
selector runs qualification during the approved trusted-policy transition; the aggregate requires
a fresh exact-candidate receipt. This adds no local image task. Direct image, feature and
Dockerfile tool-ARG proposals remain held pending the real native and App evidence in
[#563](https://github.com/Strukturpiloten/boxferry/issues/563). Source checks and mocked probes
do not establish image readiness, and no updater schedule or publication is activated. Normal
package, compiler and shared-tool metadata retain their canonical automatic owners.

Generator containers and the downloaded real-world corpus remain explicit opt-in tiers:

```console
cargo ci-generators
cargo ci-real-world-quadlet
```

See [Testing](testing.md) before selecting a tier.

## Issue-to-PR contribution workflow

Coding agents follow the issue-to-PR procedure in [`AGENTS.md`](../AGENTS.md). Human contributors
use the same essential gate: start from synchronized `main`, keep one focused change, run
`./scripts/check-all.sh`, review the staged diff, and submit a linked pull request.

All steps must pass before the change is committed, pushed, or submitted as a pull request. A file
change after a successful complete gate requires running the gate again.

The primary agent uses `gpt-6-sol` with `xhigh` reasoning and owns integration, Git and GitHub
writes, and the final complete gate. Worker agents may perform bounded implementation, research,
review, or non-mutating verification, but they never perform Git or GitHub writes. Formatting can
change repository files, so the complete check remains the primary agent's final responsibility.

## Update the container

Direct image and feature proposals remain held by the final rules in
[Renovate configuration](../.github/renovate.json). Rebuilding or changing those definitions needs
the fresh CI qualification tracked in #563. Package locks, the compiler and the existing shared
manifest remain ordinary inputs with one canonical automatic owner; their updates do not prove
image readiness.

## Runtime boundary

The Dev Container does not provide an installed systemd deployment environment and normal tests do
not need one. Exact Podman generators run in isolated containers described by the
[generator matrix](generator-matrix.md).

Add a privileged or host-integrated environment only for a named runtime claim that dry-run
generator output cannot establish. Document its engine, versions, privilege mode, isolation, inputs,
and cleanup before treating its result as evidence.

## Agent-assisted verification

Repository model defaults and role overrides live in [`.codex/`](../.codex/); permissions and
workflow ownership remain defined in [`AGENTS.md`](../AGENTS.md). Reload or start a new trusted
project session after updating configuration; the primary manager must use `gpt-6-sol` with
`xhigh` reasoning. Keep any explicit primary-session override aligned with Sol/xhigh.

Use `./scripts/check-all.sh --check` to run the complete gate without formatting repository-owned
files. The default command (or `--fix`) still formats first. Both modes run the same validation;
ignored caches and build artifacts may change. Verifiers report failures without fixing files,
and the primary agent owns the final complete gate and task-related merges within the standing
authorization in [`AGENTS.md`](../AGENTS.md).

The shell-runner regression tests target the Linux Dev Container gate. Agent-configuration checks
remain platform-independent; macOS client compatibility is intended but has no CI validation lane.
