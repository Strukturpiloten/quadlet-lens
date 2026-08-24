# Roadmap

The roadmap records current direction, not release history. Completed releases belong in the
[changelog](../CHANGELOG.md), exact feature coverage belongs in checked data, and cross-repository
conversion planning belongs to BoxFerry.

## Current foundation

- [x] Loss-aware ordered syntax with recovery and source spans
- [x] Preservation and deterministic canonical rendering
- [x] Typed documents for the audited Quadlet unit and key surface
- [x] Validated programmatic construction with parse-back
- [x] Exact-name document sets and dependency graphs
- [x] Versioned capability evidence from the fixed Podman support floor through the reviewed
      catalogue ceiling
- [x] Exact dry-run generator coverage for every release recorded in the generator matrix
- [x] Immutable, license-reviewed real-world parser corpus
- [x] Published 0.2 API contract with MSRV and SemVer checks

The [manual inventory](../fixtures/specification-drift/quadlet-manual-current.toml),
[capability catalogue](../catalogue/v1/podman-supported-range.toml), and
[generator matrix](../tools/generator-matrix.toml) provide the exact current counts and versions.

## Next work

- [ ] Admit richer value semantics only when BoxFerry or another concrete consumer needs them and
      exact evidence defines their boundary.
- [ ] Improve diagnostics and redaction where a real input exposes an unclear or unsafe result.
- [ ] Remove pre-1.0 APIs that duplicate the preferred consumer path instead of maintaining
      compatibility aliases.
- [ ] Keep BoxFerry as the primary downstream exercise of parsing, relationships, generation, and
      target support.

These items are demand-driven. They are not a commitment to model every Podman, systemd, or runtime
feature.

## Recurring maintenance

- Review every new stable Podman release in order.
- Refresh the manual inventory when the upstream documented key surface changes.
- Extend capability evidence only after reviewing documentation, source, and required boundaries.
- Add exact generator releases to the matrix and run the complete lane.
- Add a real-world source only when it protects a regression or new surface.
- Review systemd requirements when a Quadlet capability has a direct version boundary.

Recurring maintenance creates a focused issue and updates machine evidence, tests, and relevant
guidance together.

## Deferred boundaries

The following work waits for a concrete supported use case:

- distribution-specific capability overrides;
- host probing or installed-generator discovery;
- rootless or rootful runtime matrices;
- complete systemd value and command grammars;
- filesystem, registry, secret, user, group, device, or network inspection; and
- cross-format mapping policy, which remains BoxFerry-owned.

Generator output is not runtime proof. Add an environment-dependent test only when a stated runtime
claim cannot be established at a deterministic lower tier.

## Toward 1.0

Consider 1.0 when the supported API and diagnostics no longer require normal BoxFerry workarounds,
manual and version drift remain visible, representative evidence covers supported claims, and the
project can state a durable support and deprecation policy.
