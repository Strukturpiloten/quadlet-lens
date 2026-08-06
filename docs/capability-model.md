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
- evidenced and supported caller value forms
- repetition/reset semantics
- native support classification
- fallback kind and fallback range
- known broken patch ranges
- documentation and source evidence
- automated-test evidence and known gaps

Catalogue files are validated against a strict versioned TOML schema. Unknown fields, duplicate
identifiers, inverted or uncovered ranges, missing evidence, and documentation-only claims without
an explicit evidence gap are rejected.

`value_forms` describes the caller representations for which a support claim has evidence. It does
not imply that the syntax parser, typed model, or shared `EntryValue` builder validates that
key-specific grammar. Those layers preserve authored values and enforce their own structural or
physical-line boundaries unless a dedicated value type is documented separately.

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

The product support policy and catalogue evidence coverage are different ranges. Podman 5.4.0 is
the fixed minimum; the upper product target follows the newest stable Podman release. The finite
catalogue range expands only as documentation and generator evidence are reviewed. See the
[generator matrix](generator-matrix.md) and [ADR 0006](decisions/0006-rolling-support-window-and-generator-evidence.md).

A requested range contains:

```text
podmanMinimumVersion
podmanMaximumVersion  # optional
```

Validation succeeds only if the selected representation works throughout the range. A capability introduced after the minimum cannot be selected unless an earlier-compatible fallback covers the rest of the range.

When the maximum is omitted, evaluation extends through the newest catalogue version and reports
that later releases are untested assumptions. The built-in supported-range catalogue currently has
finite evidence coverage from Podman 5.4.0 through current Podman 6.0.2. Generator-proven
first-conversion capabilities span that range; capabilities not protected by the fixture remain
`unknown` above their narrower evidence boundary. A newer upstream release becomes a tracked target
before it becomes catalogue evidence.

The container stop-lifecycle capabilities use the conservative supported caller forms
`signal-token-or-number` and `non-negative-integer-seconds`. Their generator evidence records named
and numeric signals, a positive timeout, and zero. These catalogue forms are not parser or builder
validators, and the evidence does not establish a broader signal grammar, an undocumented timeout
maximum, runtime timing, whether zero sends a signal, or cross-format equivalence.

The container `RunInit` capability similarly limits its supported caller form to literal `true` or
`false`. That form does not add boolean parsing to the model or builder: omission stays absent and
all authored one-line values remain raw. The generator evidence covers only the observed CLI
difference—one `--init` for `true` and one `--init=false` for `false`—not runtime init behavior.

The container `Pull` capability limits its supported caller forms to `always`, `missing`, `never`,
and `newer`. The shared model and builder preserve any authored one-line value without semantic
validation. Generator evidence establishes matching `--pull` arguments, not registry or local
image-storage runtime behavior.

The container `PidsLimit` capability covers `-1` for unlimited and positive integers. The focused
construction helper accepts nonzero ASCII-decimal spelling and rejects empty, nondecimal, or
all-zero text. It preserves leading zeros and arbitrary-precision digits without parsing or
silently normalizing them. The supported-window evidence does not establish one portable numeric
maximum; a future target-specific bound belongs in target validation. This helper does not validate
parsed or raw `EntryValue` text: omission, authored zero, and noncanonical one-line values remain
preserved. Generator evidence covers one positive value and `-1` only; zero and runtime pids-cgroup
enforcement are not capability-evidenced.

The container `HostName` capability records the native hostname spelling without adding an
RFC-1123 or other semantic validator to the model or builder. The Quadlet documentation establishes
the `HostName`-to-`--hostname` mapping; the separate Podman run documentation requires a private UTS
namespace and records the private-container/shared-pod UTS defaults. The isolated generator fixture
relies on Podman's default private UTS namespace and proves exactly one emitted logical
`--hostname app.example` argument only. If a container joins a pod with the default shared UTS
namespace, the pod hostname wins; runtime hostname inspection and that precedence remain outside
generator evidence.

The separate container and pod `ShmSize` capabilities cover a non-negative ASCII-decimal amount
with either unitless bytes or one lowercase `b`, `k`, `m`, or `g` suffix. The focused constructor
retains leading zeros and arbitrary-precision spelling without integer parsing, while parsed and
raw `EntryValue` text remains opaque. Podman documents a `64m` default when the option is omitted,
zero as unlimited IPC memory, and a conflict with host IPC. Pods share IPC among their containers
by default, so pod `ShmSize` owns that shared context. Generator evidence establishes exactly one
matching explicit CLI argument in each isolated container or pod unit and no duplicate in the
joined container; it does not establish omission defaults, runtime enforcement, rootless behavior,
host-IPC conflict handling, namespace state, or `/dev/shm` contents.

The container `DropCapability` capability covers the documented repeatable, space-separated native
list and lowercase `all`. Parsed and programmatically constructed `EntryValue` text remains opaque:
QuadletLens does not split lists, deduplicate entries, lowercase authored values, or validate a
capability whitelist. The isolated generator fixture observes four ordered lowercase `--cap-drop`
arguments from three authored entries and no `--cap-add` argument across the finite supported range.
Tagged Podman 5.4.0 source separately shows drops being added before additions; neither that source
ordering nor generated command text proves rootless/rootful privilege behavior, an effective
bounding set, user-namespace effects, SELinux/seccomp interaction, or runtime privilege outcomes.

The container `AddCapability` capability likewise covers a documented repeatable,
space-separated native list of additions beyond Podman's default capability set. The model and
builder preserve omission, empty native reset assignments, duplicates, order, case, and exact raw
text without interpreting them. The Quadlet prose does not document `all`; tagged 5.4.0 and 6.0.2
source plus the complete generator matrix record its special handling, list splitting,
lowercasing, resets, and drop-before-add construction separately. The isolated fixture observes
four ordered lowercase `--cap-add` arguments and no drop, while a combined fixture observes one
drop-all before one specific addition. These remain source and generated-command observations,
not runtime privilege claims. See the [generator evidence](generator-matrix.md#what-the-container-test-does).

The container `Tmpfs` capability covers the Quadlet manual's repeatable
`CONTAINER-DIR[:OPTIONS]` mapping to Podman `--tmpfs`. Parsed and programmatically constructed
values remain opaque: omission, empty native reset assignments, duplicates, order, case, and exact
destination/options text are preserved without splitting, normalization, deduplication,
mount-option validation, or conflation with `Volume`. Separate Podman CLI documentation describes
the Linux default mount-flag option surface and `rw,noexec,nosuid,nodev` when options are omitted;
those target/runtime rules do not become parser validation. Tagged 5.4.0 and 6.0.2 source maps
`Tmpfs` through `LookupAll`, whose empty assignment clears earlier logical entries. Every recorded
generator confirms exactly one final post-reset `--tmpfs /data:mode=755,uid=1009,gid=1009` and no
pre-reset or extra tmpfs form. Source and generated command text do not prove target-only option
acceptance, mount creation, copy-up/default enforcement, rootless behavior, or runtime filesystem
properties.

The container `Sysctl` capability covers the endpoint Quadlet manuals' repeatable,
space-separated `name=value` list passed to Podman `--sysctl`. Parsed and programmatically
constructed entries remain opaque: omission, empty native resets, duplicates, order, case,
whitespace, systemd quoting/specifiers, and exact one-line text are preserved without parsing or
normalization. Tagged 5.4.0 and 6.0.2 source separately records `LookupAllStrv` command
construction, systemd-compatible whitespace/quote tokenization, and empty-assignment reset
behavior. Every recorded generator confirms exactly one final post-reset
`--sysctl net.ipv4.ip_forward=1`, neither pre-reset setting, and no other sysctl form. Podman-run
documentation limits accepted settings by their IPC/network namespace context; source and dry-run
output do not prove runtime namespace state, rootless behavior, kernel acceptance, runtime
equivalence, or actual sysctl effects.

The container `Ulimit` capability covers the endpoint Quadlet manuals' repeatable native key and
the Podman-run manual's `TYPE=SOFT[:HARD]` target grammar. Parsed and programmatically constructed
entries remain opaque: omission, empty native resets, duplicates, order, case, systemd
quoting/specifiers, and each exact one-line value are preserved without splitting, unquoting,
normalization, or validation. Tagged 5.4.0 and 6.0.2 source maps the key through the repeated-string
helper using `LookupAll`, not `LookupAllStrv`, and records empty-assignment reset behavior. Every
recorded generator confirms exactly two ordered final post-reset arguments,
`--ulimit nproc=4096:8192` and `--ulimit stack=-1:-1`, with no pre-reset, duplicate, empty, or
alternate form. The Podman-run documentation's default caveats are evidence context, not a
QuadletLens default claim; runtime enforcement, host inheritance, cgroups, rootless behavior, and
acceptance of unverified resource names remain outside this capability.

The container `AddDevice` capability covers the endpoint Quadlet manuals' repeatable
`HOST-DEVICE[:CONTAINER-DEVICE][:PERMISSIONS]` key and documented conditional leading-`-` form.
Parsed and programmatically constructed entries remain opaque: omission, empty native resets,
duplicates, order, case, systemd quoting/specifiers, whitespace-containing lines, leading `-`, and
each exact physical value are preserved without splitting, unquoting, parsing, normalization, or
validation. Tagged 5.4.0 and 6.0.2 source records `LookupAllStrv` tokenization, reset, conditional
leading-minus handling, and `--device` command construction separately. Every recorded generator
confirms exactly two ordered final post-reset arguments, `--device /dev/null:/dev/final-null:r`
and `--device /dev/zero:/dev/final-zero:w`, with no pre-reset, duplicate, empty, or alternate form.
The fixture deliberately avoids leading `-` and invokes no workload. Podman-run caveats bound the
evidence rather than establish QuadletLens claims: CDI, runtime access, rootless, SELinux, cgroup,
host-device existence, and symlink behavior remain outside this capability.

The container `Memory` capability begins at Podman 5.5.0, where the upstream introduction change
registers the singleton key and maps its last effective value to one `--memory` argument. Parsed
and raw builder values remain opaque, including empty, quoted, specifier-bearing, zero, and
noncanonical spellings. The focused constructor accepts only a positive ASCII-decimal amount with
no unit or one lowercase `b`, `k`, `m`, or `g`, preserving leading zeros and arbitrary precision
without integer parsing. Every recorded 5.5.0-through-6.0.2 generator emits exactly one final
`--memory 16777216b` argument for the separate fixture; the 5.4.x line rejects or excludes that
unsupported key and emits no memory argument. This establishes native recognition and dry-run
command construction only, not cgroup enforcement, page rounding, swap interaction, host-memory
availability, rootless behavior, runtime inspection, or Compose/BoxFerry equivalence.

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

Evidence records declare either `documentation` or `generator` verification and a finite exact
version or range. A generator range is valid only when every patch in it is executed.
Documentation-only records must name the missing generator evidence. A support result can therefore
be native according to primary documentation while still exposing the exact execution gap to
callers and maintainers.

## Fallbacks

Fallback records describe a semantic option, not a preassembled shell string. Rendering is responsible for safe argument construction and target syntax. A fallback must state which Podman versions support the underlying command behavior and what semantic differences remain.
