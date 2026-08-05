# Podlet issue-derived Quadlet regression map

- Reviewed: 2026-08-01
- Source: open and closed [`containers/podlet` issues](https://github.com/containers/podlet/issues)
- QuadletLens scope: native syntax, native models, document sets, rendering, and target capabilities

## Purpose

The complete tracker review covered 139 Podlet issues: 37 open and 102 closed at the review date.
Pull requests were excluded. This document records the subset that should shape QuadletLens.
Compose parsing and cross-format policy remain outside this repository.

An issue identifies a question or a historical failure. It does not establish correct Quadlet,
Podman, or systemd behavior. Capability entries still require tagged documentation/source and
exact-generator evidence. Runtime claims additionally require isolated runtime tests.

## Syntax and rendering requirements

### Ordered repeated entries

[Podlet #216](https://github.com/containers/podlet/issues/216) requested separate `Label=` entries
instead of one joined option. Named volumes in [#191](https://github.com/containers/podlet/issues/191)
also use multiple labels and empty values.

The syntax tree must preserve repetition, order, empty values, and whether entries were authored
separately. Canonical rendering may choose a documented native presentation but must not flatten
the source model into a map.

### Quoting and escaping are semantic

The following reports should become minimal parser/renderer plus real-generator fixtures:

- JSON-like label values containing quotes and whitespace:
  [Podlet #202](https://github.com/containers/podlet/issues/202);
- multiline `Environment=` values: [Podlet #32](https://github.com/containers/podlet/issues/32);
- scalar command quoting: [Podlet #36](https://github.com/containers/podlet/issues/36);
- command arguments that look like Podman options:
  [Podlet #97](https://github.com/containers/podlet/issues/97);
- multi-argument entrypoints: [Podlet #119](https://github.com/containers/podlet/issues/119); and
- security-option values containing colons:
  [Podlet #120](https://github.com/containers/podlet/issues/120).

Exact source round trips are necessary but insufficient. The real-generator suite must also inspect
the generated command so a syntactically accepted value that loses quotes, newlines, or argument
boundaries cannot be classified as supported.

### Systemd specifiers are not shell expansion

[Podlet #166](https://github.com/containers/podlet/issues/166) demonstrates that `~` can survive
into a generated service as an invalid relative path while `%h` produces the intended user-home
path. [#53](https://github.com/containers/podlet/issues/53) discusses broader systemd specifier
mapping, and [#52](https://github.com/containers/podlet/issues/52),
[#102](https://github.com/containers/podlet/issues/102), and
[#140](https://github.com/containers/podlet/issues/140) cover relative-to-absolute path policy.

QuadletLens must preserve `%h` and other specifiers as native value components. It should diagnose
shell-only forms such as literal `~` where the exact Quadlet key requires an absolute or
specifier-expanded path. Deciding whether a Compose path should become an absolute path or `%h`
belongs to BoxFerry.

## Native document-set requirements

### Resource files and references

- Generate/reference `.volume` files when top-level volume configuration carries lifecycle or
  labels: [Podlet #191](https://github.com/containers/podlet/issues/191).
- Represent Compose-like implicit network isolation through explicit `.network` resources where
  the selected plan requires it: [Podlet #190](https://github.com/containers/podlet/issues/190).
- Distinguish application-owned and external resource names:
  [Podlet #158](https://github.com/containers/podlet/issues/158) and
  [#95](https://github.com/containers/podlet/issues/95).
- Preserve the difference between a native file reference such as `name.network` and a runtime
  resource name: [Podlet #48](https://github.com/containers/podlet/issues/48) and
  [#90](https://github.com/containers/podlet/issues/90).

Document-set validation needs typed reference kinds, missing/ambiguous target diagnostics, and the
systemd dependencies implied by each native reference.

### Pod membership and networking

[Podlet #184](https://github.com/containers/podlet/issues/184) requests native `Pod=` references
from container Quadlets. [#92](https://github.com/containers/podlet/issues/92),
[#137](https://github.com/containers/podlet/issues/137), and the service-port ownership request in
[#225](https://github.com/containers/podlet/issues/225) show that accepted syntax does not establish
correct pod networking.

QuadletLens should model `.pod` membership and port placement exactly and validate them against the
target catalogue. Whether several Compose services should share one pod is BoxFerry policy. Real
runtime behavior must remain separate from generator acceptance.

### Health and service dependencies

[Podlet #160](https://github.com/containers/podlet/issues/160) distinguishes `CMD`/`CMD-SHELL`
health command forms. [#145](https://github.com/containers/podlet/issues/145) and
[#164](https://github.com/containers/podlet/issues/164) ask how Compose `service_healthy` maps into
Quadlet/systemd ordering and readiness.

The native model needs the exact health-command form, `Notify=` capability where available, and
generic `[Unit]` dependency entries. QuadletLens can validate a document graph; it must not claim
that an ordering edge reproduces health readiness unless the selected Podman/systemd versions and
generated units provide that semantic guarantee.

## Capability-catalogue requirements

### Podman version boundaries and removed arguments

Podlet reports repeatedly demonstrate version-specific generator and fallback behavior:

- requests for older-version output: [#45](https://github.com/containers/podlet/issues/45) and
  [#94](https://github.com/containers/podlet/issues/94);
- rejected `--infra-conmon-pidfile`: [#142](https://github.com/containers/podlet/issues/142);
- rejected `--infra=false`: [#162](https://github.com/containers/podlet/issues/162);
- pod naming argument differences: [#136](https://github.com/containers/podlet/issues/136) and
  [#141](https://github.com/containers/podlet/issues/141); and
- explicit user demand for version gating:
  [#200](https://github.com/containers/podlet/issues/200).

The catalogue must be able to express native key availability, fallback argument availability,
changed/removed arguments, known broken patch ranges, and distribution overrides. Podman 5.4 is the
initial floor, but exact behavior must be verified rather than inferred from a newer manual.

### Implementation extensions and pending capabilities

Useful future entries include:

- mount `chown`: [Podlet #157](https://github.com/containers/podlet/issues/157);
- `host-gateway` in host additions: [Podlet #155](https://github.com/containers/podlet/issues/155);
- CDI devices: [Podlet #107](https://github.com/containers/podlet/issues/107);
- restart `on-failure` maximum retries: [Podlet #185](https://github.com/containers/podlet/issues/185);
- `userns_mode` mapping: [Podlet #31](https://github.com/containers/podlet/issues/31); and
- pull policy mapping: [Podlet #61](https://github.com/containers/podlet/issues/61).

These should enter the catalogue only after Quadlet keys or safe Podman-argument fallbacks are
verified across the claimed range.

### Runtime and boot behavior

[Podlet #153](https://github.com/containers/podlet/issues/153) reports `Restart=always` not producing
the expected boot behavior on one distribution, while [#163](https://github.com/containers/podlet/issues/163)
reports a restart value being validated but not honored. These are reminders to separate:

1. Quadlet source validity;
2. generated systemd-unit structure;
3. Podman command semantics; and
4. actual boot/runtime behavior.

A capability record may state that a key is recognized while separately marking generated or
runtime semantics unknown/broken for an exact environment.

## Runtime-inspection boundary

[Podlet #134](https://github.com/containers/podlet/issues/134) shows that inspected containers or
pods may lack `CreateCommand`, particularly when created through APIs, Podman Desktop, or
`podman play kube`. This is not a QuadletLens parsing responsibility. BoxFerry runtime adapters
should reconstruct observations from full inspect data. QuadletLens receives an explicit typed
target document and should not depend on Podman inspection JSON.

## Fixture plan

When the matching implementation phase begins, add independently authored fixtures for:

- repeated `Label=` with empty and quote-bearing values — implemented and generator-verified in
  QuadletLens 0.1.7;
- newline-bearing environment values;
- command/entrypoint argument boundaries;
- literal `~`, `%h`, relative, and absolute paths;
- `.container` references to `.pod`, `.network`, and `.volume` files;
- health command shell/exec forms and dependency graphs;
- every Podman 5.4+ capability boundary used by the first BoxFerry slice; and
- generator output plus runtime effects where acceptance cannot prove behavior.

No Podlet implementation source or generated fixture should be copied. The issue inputs are used to
derive minimal original cases with recorded behavioral provenance.

## Non-product findings

Packaging, release cadence, repository governance, icons, communication channels, container-image
distribution, architecture binaries, and general installation support were reviewed but do not
change QuadletLens's native syntax or capability contract.
