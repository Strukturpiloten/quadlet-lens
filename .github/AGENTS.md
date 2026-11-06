# GitHub automation guidance

These instructions supplement the [root contract](../AGENTS.md). Use the
[development guide](../docs/development-environment.md) and its pinned automation manifest for shared
tooling and trusted revisions. The shared `docs/automation.md` guide belongs to that manifest revision.
Keep workflow pins and Renovate ownership aligned. The
[generator matrix](../docs/generator-matrix.md) owns native evidence, full-matrix selection, resource
bounds and cleanup; dry-run generator evidence does not qualify runtime behavior. Publication, tags and
deployment require separate approval.
