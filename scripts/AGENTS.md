# Script guidance

These instructions supplement the [root contract](../AGENTS.md). For Python, use the locked shared
uv, Ruff and ty tooling through `python3 scripts/shared-ci.py run --task python-quality` and
`./scripts/check-files.sh --check`; see the [development guide](../docs/development-environment.md).
Preserve the Bash and native generator harnesses, their explicit runtime selection, evidence, resource
bounds and cleanup. Consult the [generator matrix](../docs/generator-matrix.md) before changing them.
Shared development tooling does not add a BoxFerry runtime dependency to the library.
