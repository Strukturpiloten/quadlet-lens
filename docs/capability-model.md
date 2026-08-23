# Capability model

The capability catalogue answers whether a Quadlet representation is evidenced for an explicit
Podman target range. It does not decide whether source syntax can be parsed, and it does not predict
runtime behavior.

The checked source is
[`catalogue/v1/podman-supported-range.toml`](../catalogue/v1/podman-supported-range.toml).

## Evaluate a target

```rust
use quadlet_lens::capability::{
    CapabilityCatalogue, PodmanTarget, PodmanVersion, SupportClassification,
};

let catalogue = CapabilityCatalogue::supported_range()?;
let target = PodmanTarget::new(
    PodmanVersion::new(5, 4, 0),
    Some(PodmanVersion::new(6, 1, 0)),
)?;
let result = catalogue.evaluate("quadlet.container.image", target);

assert_eq!(result.classification(), SupportClassification::Native);
# Ok::<(), Box<dyn std::error::Error>>(())
```

A range succeeds only when one representation covers the entire range. An omitted maximum means
“through the newest catalogue evidence,” not “all future releases.”

## Classifications

| Classification | Meaning                                                                  |
| -------------- | ------------------------------------------------------------------------ |
| `native`       | The target directly supports the capability.                             |
| `fallback`     | A documented alternate representation covers the target.                 |
| `deprecated`   | The target accepts the capability but discourages it.                    |
| `removed`      | A formerly supported capability is no longer available.                  |
| `unsupported`  | Evidence shows that no supported representation exists.                  |
| `unknown`      | Retained evidence cannot establish the requested claim.                  |
| `broken`       | The target advertises or accepts the capability but behaves incorrectly. |

`unknown` is a deliberate fail-closed result. It must not be promoted to support because a newer
version resembles an evidenced one.

## Record contents

A capability record identifies its semantics, applicable units and sections, finite support
ranges, value forms, repetition or reset behavior, fallbacks, known broken ranges, evidence, tests,
and explicit gaps.

`value_forms` describes caller representations covered by evidence. It does not add key-specific
parsing or normalization to the typed model or `EntryValue`.

Catalogue validation rejects unknown fields, duplicate identifiers, incoherent or uncovered
ranges, contradictory support, missing evidence, and documentation-only claims without a named
gap.

## Evidence levels

Keep these claims separate:

1. **Documentation evidence** records what an exact manual, release note, or tagged source says.
2. **Generator evidence** records dry-run output from every exact Podman release in a claimed range.
3. **Runtime evidence** records behavior from an explicitly described installed environment.

Generator output proves command or unit construction only. It does not prove image pulls, mounts,
network isolation, systemd activation, rootless behavior, or a running workload.

Systemd requirements use their own evidence records and optional caller-supplied
`SystemdVersion`. QuadletLens never probes the host or infers distribution backports.

## Update a capability

1. Identify the semantic claim and its smallest useful range.
2. Compare exact tagged manuals, release notes, and relevant tagged source.
3. Add or amend evidence with a claim, test, and remaining gap.
4. Add boundary tests immediately below and above each change where releases exist.
5. Add exact generator coverage when claiming generator behavior.
6. Run catalogue, model, policy, and relevant generator checks.
7. Review the public compatibility wording if the support contract changed.

Generated key diffs can reveal work but cannot establish semantics. Exact evidence details belong in
the catalogue and fixtures, not in this guide.

## Fallbacks and overrides

A fallback describes a semantic representation and its own support range, never a preassembled
shell command. Rendering remains responsible for safe target syntax.

Distribution overrides are intentionally absent. A future override requires a concrete supported
backport case, explicit caller selection, visible evidence, and an architectural decision.

The version policy is recorded in
[ADR 0006](decisions/0006-rolling-support-window-and-generator-evidence.md). The
[generator guide](generator-matrix.md) explains exact execution.
