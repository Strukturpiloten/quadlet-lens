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
Local generator execution and the real-world corpus are opt-in tiers. Every Release automatically
and fail-closed runs the reusable full pinned generator matrix plus the tracked-current
application-generator lane; `validation_only` runs the identical gate. This is dry-run generator
evidence only: it never installs, enables, starts, or otherwise executes systemd units or
workloads.

## Workspace scope and standing GitHub authorization

The maintainer grants standing authorization for task-related Git and GitHub work only in these
workspace repositories:

- `Strukturpiloten/boxferry`
- `Strukturpiloten/compose-lens`
- `Strukturpiloten/podman-lens`
- `Strukturpiloten/quadlet-lens`
- `Strukturpiloten/boxferry-website`
- `Strukturpiloten/docker-lens`

Do not work on or modify any repository outside this explicit allowlist, including its issues,
pull requests, branches, settings, or workflows. An upstream documentation reference is not
permission to operate on that upstream repository. A newly discovered checkout is not implicitly
in scope.

For user-requested work within this scope, the primary agent may create issues, branches, commits,
pushes, and pull requests and merge verified task-related pull requests without asking for renewed
approval. This permission does not authorize unrelated backlog work, implementation of
discussion-only proposals, or expansion of the requested product scope. A later user instruction
may narrow or revoke this permission.

Immediately before merging, read back the exact head commit and verify that the pull request is
ready, mergeable, independently reviewed, and has every required check successful. Use the normal
merge method with an exact-head safeguard; never bypass branch protection or use an administrator
override. Read back the merged state and merge commit, synchronize local `main` with `origin/main`,
and remove the task's recorded worktrees and verified merged local branches while preserving
unrelated work.

This standing permission does not authorize releases, publication, deployment operations, or
merging release/publication/deployment pull requests; those require a separate explicit request.
The primary agent owns all Git and GitHub writes. Subagents remain within their assigned task and
checkout and must not perform those writes.

## GitHub issue-to-PR workflow

For user-requested work within the authorized workspace scope:

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

The primary agent owns Git and GitHub writes, integration review, the final complete gate, staging,
and pull-request readback.

Worker subagents never execute the Git or GitHub write steps. They may perform bounded research,
implementation, review, or non-mutating verification. The final formatting and complete gate
remains the primary agent's responsibility. Subagents never commit, push, publish, tag, or release.

## Multi-agent coordination

- Delegate only bounded tasks with independently verifiable results.
- Never run two writing agents in this checkout concurrently.
- Run read-only review or verification after writing finishes.
- The primary agent reviews every diff and owns cross-repository API decisions.

## Agent roles and verification

Model defaults belong in [`.codex/config.toml`](.codex/config.toml); task-specific models and
reasoning belong in [`.codex/agents/`](.codex/agents/). The primary manager always uses
`gpt-6-astra` with `xhigh` reasoning. Implementation, specification research, and independent review
use `gpt-6-sol` with `high` reasoning; check-only verification uses `gpt-6-luna` with `high`
reasoning. Use Luna for bounded read-only exploration and Sol for difficult failure diagnosis.
These model settings do not expand the workspace scope or grant additional permissions.

- Delegate bounded tasks when independent work can usefully proceed in parallel. Define the shared
  contract and explicit repository, checkout, and file ownership before delegation.
- Use up to nine concurrent subagents plus the primary manager, subject to the session's actual
  runtime limit. Nine is a ceiling, not a target or nine distinct roles: several subagents may use
  the same role for independent tasks. Do not create nested agents to evade the limit.
- Never run two writers in one checkout. Use separate assigned repositories or worktrees for
  concurrent implementation. Research and review remain read-only.
- The reviewer checks the original requirements and independent expected results, not just agreement
  between the implementation and its tests.
- After writing finishes, the verifier runs `./scripts/check-all.sh --check`. It reports failures
  without formatting or editing tracked files; ignored build artifacts and caches are allowed.
- Run at most one complete gate or heavy runtime suite at a time across this workspace. Agent
  concurrency is not permission for competing builds. The primary owns integration, the final
  complete gate, and every authorized Git or GitHub write.

The default `./scripts/check-all.sh` still formats before checking. `--check` runs the same
complete gate without source formatting; it is not a reduced test tier. A later edit invalidates
either result. Neither mode grants release, publication, or deployment authority.

## Cross-repository workflow version policy

- Keep equivalent local development tasks, GitHub PR, main, and release workflow definitions aligned across BoxFerry, ComposeLens, PodmanLens, QuadletLens, DockerLens, and the website where responsibilities match. Before a change, identify the canonical definition and every affected consumer; coordinate updates and document justified repository-specific differences.
- Reuse common scripts, actions, and workflows without making Lens product libraries depend on BoxFerry. Preserve native conformance, least privilege, exact-candidate evidence, resource budgets, and cleanup. One repository passing does not establish that a shared rollout is complete.
- Every added or changed software dependency or operational tool/runtime pin needs an explicit version and immutable integrity information where the ecosystem supports it:
  - Container images: readable version tag plus immutable digest.
  - GitHub Actions and reusable workflows: full commit SHA plus exact release-tag comment.
  - Downloaded tools: version plus verified checksum for the selected artifact.
  - Package dependencies: policy-compliant version declarations, lockfiles, and integrity records.
    Document justified exceptions when integrity metadata is unavailable; never invent a checksum, replace a reviewed pin with a floating reference, or weaken existing admission controls.
- Whenever a pin or definition is added, changed, moved, or removed, review Renovate in the same change: canonical ownership, manager paths, extraction, grouping, approvals, and regression coverage. Update configuration and affected consumers together. If no configuration edit is needed, record verified extraction evidence and the reason in the issue or PR. Avoid duplicate managers for the same operational pin; keep historical evidence and intentional fixtures outside automatic update streams.
- These rules do not change current validation gates or grant release, publication, deployment, or out-of-workspace authority. Agent model choices remain maintainer-owned routing policy, not automatically updated software dependencies.

## Code discovery

For code discovery, use an available codebase-memory graph first; otherwise use CodeGraph only if
the repository already has a usable index. Do not create an index without user authorization.
If neither graph is available or a query cannot answer the question, use `rg` and targeted reads.
For string literals, configuration, scripts, and documentation, start with `rg` directly.

## After an authorized merge

Read back the merged state and exact merge commit, then synchronize the primary checkout with
`origin/main`. Preserve unrelated files. Remove only the recorded task worktree with
`git worktree remove <recorded-path>`, delete the verified merged local issue branch with
`git branch --delete --force TheRealBecks/issue<NUMBER>`, and run
`git worktree prune --verbose`. Read back `git worktree list --porcelain` and
`git status --short --branch`; do not leave stale task worktree registrations.
