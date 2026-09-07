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

The VS Code task **QuadletLens: Format, lint, and test all** runs the complete gate. Other tasks
match the focused Cargo aliases.

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

The primary agent uses high reasoning effort and owns integration, Git and GitHub writes, and
the final complete gate. Worker agents may perform bounded implementation, research, review, or
non-mutating verification, but they never perform Git or GitHub writes. Formatting can change
repository files, so the complete check remains the primary agent's final responsibility.

## Update the container

Review Renovate changes to their original source: image digests, feature integrity values, Rust
toolchain, Node lockfile, native tool checksums, and workflow pins. Regenerate the Dev Container
lock with the CLI version already pinned by the repository, rebuild the container, run
`.devcontainer/verify-tools.sh`, and then run the complete gate.

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
project session after updating configuration; an explicit session override can take precedence.

Use `./scripts/check-all.sh --check` to run the complete gate without formatting repository-owned
files. The default command (or `--fix`) still formats first. Both modes run the same validation;
ignored caches and build artifacts may change. Verifiers report failures without fixing files,
and the primary agent owns the final complete gate and any explicitly authorized merge.

The shell-runner regression tests target the Linux Dev Container gate. Agent-configuration checks
remain platform-independent; the macOS portability lane does not require Linux validation tools.
