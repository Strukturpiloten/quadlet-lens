# Capability model

## Purpose

Quadlet evolves with Podman and relies on systemd behavior. The catalogue gives the parser, validator, renderer, and downstream tools a shared evidence-backed answer about target compatibility.

It does not attempt to catalogue every Podman or systemd feature. Its scope is the Quadlet document contract and documented fallbacks.

## Capability identity

Capabilities use stable, namespaced identifiers, for example:

```text
quadlet.unit-type.container
quadlet.container.example-key
quadlet.container.example-key.value-form
quadlet.reference.template-instance
systemd.specifier.home-directory
```

Identifiers describe semantics rather than Rust type names.

## Capability record

A record can include:

- stable identifier and description
- applicable unit types and sections
- introduced version
- changed versions
- deprecated version
- removed version
- accepted value forms
- repetition/reset semantics
- native support classification
- fallback kind and fallback range
- known broken patch ranges
- documentation and source evidence
- automated-test evidence and known gaps

Catalogue files are validated against a versioned schema.

## Support classifications

| Classification | Meaning                                                                  |
| -------------- | ------------------------------------------------------------------------ |
| `native`       | Target directly supports the capability.                                 |
| `fallback`     | A documented compatible representation exists, such as Podman arguments. |
| `deprecated`   | Accepted but discouraged for the target.                                 |
| `removed`      | Previously supported but unavailable in the target.                      |
| `unsupported`  | No supported representation exists.                                      |
| `unknown`      | Available evidence cannot establish behavior.                            |
| `broken`       | Advertised or accepted, but a known target range behaves incorrectly.    |

## Version ranges

A requested range contains:

```text
podmanMinimumVersion
podmanMaximumVersion  # optional
```

Validation succeeds only if the selected representation works throughout the range. A capability introduced after the minimum cannot be selected unless an earlier-compatible fallback covers the rest of the range.

When the maximum is omitted, evaluation extends through the newest catalogue version and reports that later releases are untested assumptions.

Exact runtime detection can narrow validation to one version, but generated project files should normally declare their intended portable range.

## Patch releases and distribution backports

Feature introduction is usually tracked by Podman minor version, while known bugs may require patch-level ranges. Distribution packages may backport fixes or features without changing the upstream minor version.

Target profiles therefore support explicit enable/disable overrides. Overrides are visible in validation reports and never silently modify the catalogue.

## Evidence workflow

1. Compare tagged Podman documentation and release notes.
2. Inspect relevant tagged implementation behavior without copying implementation code.
3. Produce a candidate catalogue change.
4. Run fixtures against exact Podman generator versions.
5. Review semantic behavior, not only whether a key is accepted.
6. Record evidence, test result, and remaining uncertainty.

Generated diffs may assist this process, but a generated list of keys is not sufficient evidence of correct semantics.

## Fallbacks

Fallback records describe a semantic option, not a preassembled shell string. Rendering is responsible for safe argument construction and target syntax. A fallback must state which Podman versions support the underlying command behavior and what semantic differences remain.
