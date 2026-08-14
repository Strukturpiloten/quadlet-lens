# ADR 0010: Consolidated 0.2 public API

- Status: accepted
- Date: 2026-08-14
- Supersedes: [ADR 0008](0008-versioned-public-api-and-release-contract.md)

## Context

The 0.1.x line established the Quadlet syntax, model, document-set, rendering, and capability
boundaries. While adding typed systemd Unit relationships, `SystemdUnitKey` temporarily remained
available from both `model` and `render` solely to preserve its first public path. The duplicate
ownership is inappropriate for the long-term API: the enum classifies parsed model entries and is
only consumed by the renderer.

## Decision

QuadletLens publishes a supported 0.2.x API line.

- `SystemdUnitKey` is owned and exported only by `quadlet_lens::model`.
- `quadlet_lens::render` consumes the model type but does not re-export it.
- Compatibility-only public paths are removed rather than deprecated while the library remains
  pre-1.0.
- Patch releases inside 0.2.x preserve the documented public paths. A later intentional public
  break requires another 0.x minor release and migration guidance.
- CI, release, and local SemVer checks derive the release type from Cargo package versions instead
  of forcing patch semantics.
- The existing side-effect, diagnostic, Rust 1.85, finite-capability, generator-evidence, package,
  and auditable-release contracts remain.

## Consequences

Callers import one authoritative systemd key type from `model`. The 0.2.0 version communicates the
path break, and the renderer has no duplicate public ownership to maintain.

## Alternatives considered

### Keep the render re-export

Rejected because it exists only for 0.1.x source compatibility and obscures the model/render
dependency direction.

### Move the type back into render

Rejected because parsed `EntryKind` values and document-set relationship edges use the same type;
rendering is a consumer, not its owner.
