# Repository guidance for coding agents

This file applies to the entire QuadletLens repository.

## Read before changing code

1. Read [`README.md`](README.md) and the task map in [`docs/README.md`](docs/README.md).
2. Read only the guide selected for the task: architecture, typed model, generation, capabilities,
   generators, testing, dependencies, or releases.
3. Read [`docs/decisions/README.md`](docs/decisions/README.md) and only the ADRs relevant to an
   architectural boundary being changed.

Architectural changes require documentation and an ADR update in the same change.

## Scope

QuadletLens owns native Quadlet syntax, typed models, document-set relationships, rendering,
diagnostics, and capability evidence for supported Podman and systemd targets.

It does not own Compose, Kubernetes semantics, runtime inspection, BoxFerry's neutral model, or
cross-format conversion decisions. It must not depend on BoxFerry.

## Origin policy

QuadletLens is implemented from scratch. Do not copy or mechanically translate source from Podlet,
Podman, systemd, or another parser. Public documentation, released source behavior, and commands
from identified versions may inform independent implementation and differential tests.

External fixtures and behavior-oracle results record source, version, command, environment, license,
and expected result.

## Non-negotiable behavior

- Unknown keys and generic systemd sections are never silently discarded.
- Repeated keys and order remain explicit; a map is not a syntax representation.
- Systemd specifiers remain distinct from shell or Compose substitutions.
- Source syntax, typed values, and target validation remain separate layers.
- A capability claim has evidence and a test or an explicit test gap.
- Invalid input returns structured diagnostics and never panics.
- Rendering is deterministic for the same document, target, and options.
- Parsing and rendering never install, enable, start, or otherwise mutate units.

## Development rules

- Use the data-driven catalogue instead of a closed Podman version enum in application logic.
- Model Quadlet features needed by the library contract, not every Podman feature.
- Keep target evaluation independent of the parser.
- Add parser, renderer, capability-boundary, and generator tests where behavior changes.
- Record exact upstream source and observed behavior for capability updates.
- Update compatibility guidance with the implementation.
- Start repository-owned complete YAML documents with `---`.
- Keep release notes concise and link to canonical technical documentation.
- Pin GitHub Actions to full commit SHAs with exact release-tag comments.

## Canonical commands

```console
./scripts/check-all.sh
./scripts/check-files.sh --check
cargo ci-check
cargo ci-catalogue
cargo ci-generators
cargo ci-model
cargo ci-policy
cargo ci-clippy
cargo ci-test
cargo ci-doctest
RUSTDOCFLAGS="-D warnings" cargo ci-doc
cargo +1.85.0 ci-check
cargo +1.85.0 ci-policy
cargo deny check
```

The `ci-*` aliases use locked resolution and all workspace features and targets where applicable.
Generator execution and the real-world corpus are opt-in tiers.

## GitHub issue-to-PR workflow

When the user authorizes issue, branch, commit, push, and pull-request writes:

1. Inspect status and the complete diff; preserve unrelated changes.
2. Search for a duplicate issue and create one focused issue if needed.
3. Fetch `origin/main`, verify local `main`, and create `TheRealBecks/issue<NUMBER>`.
4. Implement and review without staging unrelated paths.
5. Run `./scripts/check-all.sh`. This is a hard gate against commit, push, or pull-request
   creation. Any later source, test, configuration, or documentation edit invalidates the run.
6. Stage explicit paths, run `git diff --cached --check`, review the staged diff, and create one
   intentional commit.
7. Push and open a ready pull request containing `Closes #<NUMBER>`.
8. Use release-worthy Conventional Commit types only for shipped behavior. Use `docs`, `test`,
   `ci`, `build`, `style`, or `chore` for maintenance so release-plz ignores it.
9. Read the pull request back and report the issue, branch, commit, validation, URL, and checks.

The primary Sol agent runs this workflow with high reasoning effort. Sol owns Git and GitHub
writes, integration review, the final complete gate, staging, and pull-request readback.

Terra subagents never execute the Git or GitHub write steps. They may perform bounded research,
implementation, review, or non-mutating verification. The final formatting and complete gate
remains Sol's responsibility. Subagents never commit, push, publish, tag, or release.

## Multi-agent coordination

- Delegate only bounded tasks with independently verifiable results.
- Never run two writing agents in this checkout concurrently.
- Run read-only review or verification after writing finishes.
- The primary agent reviews every diff and owns cross-repository API decisions.
