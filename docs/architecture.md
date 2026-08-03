# Software architecture

## Purpose

QuadletLens provides a native Quadlet representation and target-aware validation without invoking Podman or systemd during normal parsing and rendering.

## Layers

```text
source files ───────────────────────────────────────────────────┐
                                                               ▼
native generated values ──▶ document builder ──▶ unit syntax documents ──▶ typed Quadlet documents ──▶ document set and graph
                                │                     │                            │                              │
                                ├──▶ typed keys       ├──▶ comments/order          ├──▶ native value types        ├──▶ references
                                ├──▶ exact values     ├──▶ repeated keys           ├──▶ unknown fields            ├──▶ dependencies
                                └──▶ parse-back       ├──▶ source spans            │                              │
                                                      │                            │                              │
                                                      ├────────────────────────────┼──────────────────────────────┼──▶ renderer
                                                      │                            │                              │
                                                      └────────────────────────────┴──────────────────────────────┤
target profile ──▶ capability catalogue ─────────────────────────────────────────────────────────────────────────┴──▶ validation report
```

### Unit syntax

The syntax layer represents sections and ordered entries rather than flattening them into maps. It owns comments, blank lines, continuations, quoting, repeated keys, reset behavior, and source spans.

The initial dependency-free grammar stores immutable source text and classifies every physical line
as blank, comment, section, entry, continuation, or recoverable invalid input. It retains repeated
keys, comment markers, continuation context, line endings, source spans, and specifier-shaped text
without decoding values. [ADR 0002](decisions/0002-loss-aware-systemd-syntax.md) defines this
boundary. Supporting every systemd file type and every systemd parser extension is not required.

Valid parse results can also render a conservative canonical form. It normalizes structural
indentation, assignment spacing, and line endings while retaining order, repetition, comments,
continuations, raw value spelling, and specifiers. It refuses invalid syntax and does not perform
typed normalization. [ADR 0003](decisions/0003-conservative-canonical-syntax-rendering.md) defines
that boundary.

### Typed Quadlet model

Typed documents represent native Quadlet unit types, including container, pod, network, volume, image, build, kube, and artifact units as supported by target versions.

Generic systemd sections and unknown Quadlet entries remain attached to the document. Typed conversion cannot be destructive.

The first implemented subset covers `.container`, `.pod`, `.network`, and `.volume`. It classifies the
native keys needed by the first conversion, keeps repeated section occurrences and entries in
source order, and owns the authored text plus source span for every typed name and value segment.
That boundary includes execution identity and container context without resolving host users,
groups, paths, or namespaces.
Generic `[Unit]`, `[Service]`, and `[Install]` entries remain open-ended. Unknown sections and keys
remain explicit entries rather than validation losses.

Value interpretation is deliberately conservative. Environment-file and mount sources receive
lexical path classifications, and `.image`, `.build`, `.pod`, `.network`, and `.volume` references
receive explicit reference kinds. Systemd command lines, environment assignments, ports, health
commands, and raw Podman arguments remain opaque until their behavior is protected by focused
parsers and exact generator evidence. [ADR 0005](decisions/0005-source-aware-native-typed-model.md)
defines this boundary.

### Document set and dependency graph

A Quadlet application commonly spans multiple files. The implemented document set pairs every
typed document with a validated unit-file basename, requires unique source identities, and resolves
native references by exact basename without consulting the filesystem. The graph retains resolved,
missing, and ambiguous references; resolved relationships become deterministic dependency edges.
Source-labelled diagnostics report missing targets, ambiguous targets, and duplicate basenames.

### Capability catalogue

The catalogue describes when a unit type, section, key, value form, or fallback is available. It is
strict versioned TOML, data-driven, and independently validated. Coverage ranges are separate from
the rolling product support window and upstream introduction claims, known patch bugs override
native support, and documentation evidence remains distinguishable from exact generator execution.
The built-in catalogue starts at Podman 5.4.0 and currently verifies the first-conversion subset
through current Podman 6.0.2. [ADR 0004](decisions/0004-versioned-capability-catalogue.md) defines the core schema;
[ADR 0006](decisions/0006-rolling-support-window-and-generator-evidence.md) defines ranged evidence
and the rolling upper target.

### Target validator

Validation combines documents, a target profile, and catalogue data. It reports syntax validity, native-model constraints, cross-file references, target support, deprecation, known issues, and fallback availability.

### Renderer

The implemented programmatic renderer accepts typed native keys, exact one-line native values, and
open-ended generic systemd directives. It produces deterministic section order and reparses every
generated document before returning it. It deliberately does not invent quoting or normalization
rules for opaque systemd and Podman value grammars. [ADR 0009](decisions/0009-validated-programmatic-generation.md)
defines this boundary.

The broader renderer direction supports:

- preservation-oriented edits based on syntax documents
- deterministic canonical files based on typed documents
- target-aware rendering when value syntax differs by version

Rendering never installs or executes the result.

## Target profile

A target profile may include:

- Podman minimum and optional maximum version
- exact detected Podman version
- systemd minimum or detected version
- rootless or rootful mode
- allowed fallback policy
- explicit capability overrides for distribution backports

Output claimed compatible with a range must validate across the entire range.

## Dependency rules

- Syntax does not depend on typed Quadlet models.
- Typed models may carry source references but not parser internals.
- Capability data does not depend on BoxFerry.
- Validation depends on models and capability interfaces.
- External generator verification is an optional test/tooling layer, not parser behavior.
- QuadletLens never depends on BoxFerry.
