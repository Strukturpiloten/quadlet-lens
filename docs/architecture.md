# Software architecture

## Purpose

QuadletLens provides a native Quadlet representation and target-aware validation without invoking Podman or systemd during normal parsing and rendering.

## Layers

```text
source files
     │
     ▼
unit syntax documents ──▶ typed Quadlet documents ──▶ document set and graph
     │                            │                              │
     ├──▶ comments/order          ├──▶ native value types        ├──▶ references
     ├──▶ repeated keys           ├──▶ unknown fields            ├──▶ dependencies
     ├──▶ source spans            │                              │
     │                            │                              │
     ├────────────────────────────┼──────────────────────────────┼──▶ renderer
     │                            │                              │
     └────────────────────────────┴──────────────────────────────┤
target profile ──▶ capability catalogue ─────────────────────────┴──▶ validation report
```

### Unit syntax

The syntax layer represents sections and ordered entries rather than flattening them into maps. It owns comments, blank lines, continuations, quoting, repeated keys, reset behavior, and source spans.

The initial grammar covers the systemd-style syntax used by Quadlet. Supporting every systemd file type and every systemd parser extension is not required.

### Typed Quadlet model

Typed documents represent native Quadlet unit types, including container, pod, network, volume, image, build, kube, and artifact units as supported by target versions.

Generic systemd sections and unknown Quadlet entries remain attached to the document. Typed conversion cannot be destructive.

### Document set and dependency graph

A Quadlet application commonly spans multiple files. A document set resolves references between units, detects missing or ambiguous targets, and exposes dependencies without requiring installation.

### Capability catalogue

The catalogue describes when a unit type, section, key, value form, or fallback is available. It is data-driven and independently validated. It may describe known broken patch ranges separately from feature introduction.

### Target validator

Validation combines documents, a target profile, and catalogue data. It reports syntax validity, native-model constraints, cross-file references, target support, deprecation, known issues, and fallback availability.

### Renderer

The renderer supports:

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
