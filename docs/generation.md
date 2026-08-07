# Programmatic generation

QuadletLens 0.1.1 adds a validated construction boundary for tools that generate Quadlet files.
[ADR 0009](decisions/0009-validated-programmatic-generation.md) records its scope.

## Document construction

`QuadletDocumentBuilder` is created for one `QuadletUnitType`. Typed `push_container`, `push_pod`,
`push_network`, and `push_volume` methods prevent native keys from being written into the wrong
section. `push_systemd` adds open-ended directives to `[Unit]`, `[Service]`, or `[Install]`.
`push_systemd_unit` provides typed `Requires`, `Wants`, and `After` spellings for the dependency
subset protected by capability and real-generator evidence.

Repeated native keys retain insertion order. Native keys classified as singletons are rejected
when repeated. Generic systemd directives may repeat because their list and reset semantics are
directive-specific.

A generated container requires exactly one workload source. `ContainerKey::Image` selects an image
or native image/build reference; `ContainerKey::Rootfs` selects a Podman root filesystem. Building
a document with neither or both returns structured typed-model diagnostics.

`ContainerKey::ContainerName` optionally selects the exact Podman runtime name. It is a singleton
and remains separate from the Quadlet filename and generated service identity.

`ContainerKey::Entrypoint` optionally overrides the image entrypoint and is a singleton distinct
from `ContainerKey::Exec`. Multiple entrypoint arguments use the documented JSON command-array
text; `EntryValue` retains that spelling exactly.

`ContainerKey::RunInit` is a singleton that carries exact authored one-line text. Leaving the key
out, writing `true`, and writing `false` remain three distinct builder results; raw noncanonical
text is preserved rather than interpreted as a boolean. Across the evidenced Podman
5.4.0-through-6.0.2 generator range, authored `true` emits exactly one `--init` argument and
authored `false` emits exactly one `--init=false` argument. This generator observation does not
inspect the init binary or establish runtime signal-forwarding and child-reaping behavior.

`ContainerKey::StopSignal` and `ContainerKey::StopTimeout` are singletons. Named/numeric signals and
non-negative integer seconds are the caller forms backed by current capability and generator
evidence; in particular, the builder preserves and emits `StopTimeout=0` without substituting a
default. The keys still use the shared raw-value boundary rather than dedicated semantic
validators.

`ContainerKey::Pull` is a singleton carrying exact authored one-line text. Omission stays absent;
the evidenced caller forms are `always`, `missing`, `never`, and `newer`, but the shared builder
does not reject other raw values or claim Podman accepts them.

`ContainerKey::PidsLimit` is a singleton carrying exact authored one-line text. Raw `EntryValue`
construction preserves omission, zero, overflow-sized text, and noncanonical values without
claiming they are accepted. `PidsLimit::unlimited()` and `PidsLimit::finite` are the stronger
additive construction path: they emit `-1` or exact nonzero ASCII-decimal spelling, rejecting
empty, nondecimal, and all-zero text. Finite values retain leading zeros and arbitrary-precision
digits without parsing into a machine integer, so this layer cannot overflow or silently normalize
the caller's value. The supported-window evidence does not establish a portable target maximum;
current generator evidence covers positive `127` and unlimited `-1`, not zero or runtime cgroup
enforcement.

`ContainerKey::HostName` is a singleton carrying exact authored one-line text. Omission stays
absent, and the shared `EntryValue` boundary rejects only NUL bytes and physical line endings; it
does not apply Compose RFC-1123 validation or normalize native values. For an isolated container,
Podman's default private UTS namespace lets the hostname apply. When a container joins a pod with
the default shared UTS namespace, the pod hostname wins. The generator fixture proves one emitted
logical `--hostname app.example` argument only, not runtime hostname behavior or pod precedence.

`ContainerKey::ShmSize` and `PodKey::ShmSize` are distinct singletons carrying exact authored
one-line text. The stronger `ShmSize::new` path accepts a non-negative ASCII-decimal amount with no
unit or one lowercase `b`, `k`, `m`, or `g`, preserving leading zeros and arbitrary precision
without parsing. `ShmSize::unlimited()` emits explicit `0`, distinct from omission and the
documented `64m` default. The builder does not select IPC mode or inspect `/dev/shm`; pod values
belong to the pod's shared-IPC context.

`ContainerKey::DropCapability` is repeatable. Each `EntryValue` stays one exact authored physical
line, including any space-separated list, and repeated entries retain insertion order. The builder
does not split, deduplicate, lowercase, or validate capability tokens. Podman's generator performs
its own observed lowercase expansion into `--cap-drop` arguments; callers must not treat that
generated-command observation as proof of runtime privilege state.

`ContainerKey::AddCapability` is repeatable with the same raw boundary. Omission stays absent;
empty native reset assignments, duplicates, order, case, and space-separated text are retained
exactly. QuadletLens does not apply the generator's observed splitting and lowercasing or interpret
`all`. Tagged Podman source and generated commands record those behaviors separately from this
builder and from runtime privilege state.

`ContainerKey::Tmpfs` is repeatable and retains each `EntryValue` as one exact physical line.
Omission, empty native reset assignments, duplicates, insertion order, case, destination spelling,
and options remain unchanged. The builder does not split or normalize
`CONTAINER-DIR[:OPTIONS]`, deduplicate destinations, validate target mount options, or route these
values through `ContainerKey::Volume`.

`ContainerKey::Sysctl` is repeatable and retains each `EntryValue` as one exact physical line.
Omission, empty native resets, duplicates, insertion order, case, whitespace, systemd
quoting/specifiers, and authored text remain unchanged. The builder does not parse `name=value`,
split a space-separated list, normalize assignments, validate namespaces, or apply kernel/runtime
rules. `PodKey` intentionally has no `Sysctl` variant.

`ContainerKey::Ulimit` is repeatable and retains each `EntryValue` as one exact physical line.
Omission, empty native resets, duplicates, insertion order, case, systemd quoting/specifiers, and
authored text remain unchanged. The builder does not split, unquote, or validate
`TYPE=SOFT[:HARD]`, infer defaults, or apply runtime resource-limit rules. `PodKey` intentionally
has no `Ulimit` variant.

`ContainerKey::AddDevice` is repeatable and retains each `EntryValue` as one exact physical line.
Omission, empty native resets, duplicates, insertion order, case, systemd quoting/specifiers,
whitespace-token-containing lines, a leading `-`, and authored text remain unchanged. The builder
does not split, unquote, parse host/container paths or permissions, check devices, or apply
conditional inclusion. `PodKey` intentionally has no `AddDevice` variant.

`ContainerKey::Memory` is a singleton with the same raw `EntryValue` preservation boundary.
`Memory::new` provides a stronger additive path for a positive ASCII-decimal amount with no unit
or one lowercase `b`, `k`, `m`, or `g`, retaining leading zeros and arbitrary precision without
parsing. It does not accept zero or make runtime, cgroup, page-size, swap, host-memory, rootless, or
cross-format claims. `PodKey` intentionally has no `Memory` variant.

`DNS`, `DNSOption`, `DNSSearch`, `ExposeHostPort`, `Annotation`, `Mask`, and
`Unmask` are repeatable `ContainerKey` variants. The builder preserves each physical-line-safe
`EntryValue` in insertion order, including duplicates and reset assignments.

`AppArmor`, `NoNewPrivileges`, `SeccompProfile`, and the five `SecurityLabel*` variants are
singletons. Their values remain opaque and a second assignment is rejected.

The builder performs no address, port, OCI, boolean, profile, SELinux, path, filesystem, host, or
runtime validation. Unsupported scopes expose no typed construction key.

`ContainerKey::Secret` is repeatable. Its exact value may select mounted-file or environment
exposure and carry target, UID, GID, and mode options; the builder preserves those options without
reading the referenced Podman secret.

`ContainerKey::Label` is repeatable. Each value remains an exact native `key=value` assignment;
the builder preserves insertion order and does not enforce reverse-DNS naming recommendations or
merge duplicate label names.

`PodKey::UserNS` configures the namespace shared by pod members and is a singleton. It is distinct
from `ContainerKey::UserNS`: Podman ignores container-level namespace selection after a container
joins a pod.

`build` emits deterministic text, reparses it through the normal syntax and typed-model pipeline,
and fails if that result contains an error. Successful output exposes the source text, typed
document, and complete parse result.

## Value boundary

`EntryValue` is exact authored text on one physical line. It rejects NUL bytes and line endings
but does not validate, quote, or normalize key-specific grammar. Callers select the appropriate
native spelling for the typed keys in the
[specification coverage ledger](roadmap.md#specification-coverage-ledger).

This is an explicit boundary, not a claim that all value forms are interchangeable. Future
key-specific constructors can add stronger guarantees once exact Podman-version behavior and
systemd escaping rules are covered by evidence. In particular, raw noncanonical `RunInit` text and
raw one-line negative, fractional, or overflow-sized timeout text is preserved rather than rejected
by `EntryValue`; that preservation is not a claim that Podman accepts or gives useful runtime
meaning to those values.

## Example

```rust
use quadlet_lens::{
    model::{ContainerKey, QuadletUnitType},
    render::{EntryValue, Memory, PidsLimit, QuadletDocumentBuilder, ShmSize},
    source::SourceId,
};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let mut builder = QuadletDocumentBuilder::new(QuadletUnitType::Container);
builder.push_container(
    ContainerKey::ContainerName,
    EntryValue::new("example-application")?,
)?;
builder.push_container(
    ContainerKey::AddHost,
    EntryValue::new("host.docker.internal:host-gateway")?,
)?;
builder.push_container(
    ContainerKey::Image,
    EntryValue::new("example.invalid/application:1")?,
)?;
builder.push_container(
    ContainerKey::Entrypoint,
    EntryValue::new(r#"["/usr/bin/env","php"]"#)?,
)?;
builder.push_container(ContainerKey::RunInit, EntryValue::new("true")?)?;
builder.push_container(ContainerKey::StopSignal, EntryValue::new("SIGUSR1")?)?;
builder.push_container(ContainerKey::StopTimeout, EntryValue::new("0")?)?;
builder.push_container(ContainerKey::Pull, EntryValue::new("newer")?)?;
builder.push_container(ContainerKey::PidsLimit, PidsLimit::finite("127")?.into())?;
builder.push_container(ContainerKey::HostName, EntryValue::new("app.example")?)?;
builder.push_container(ContainerKey::ShmSize, ShmSize::new("128m")?.into())?;
builder.push_container(ContainerKey::Memory, Memory::new("268435456b")?.into())?;
builder.push_container(ContainerKey::DropCapability, EntryValue::new("CAP_NET_ADMIN")?)?;
builder.push_container(
    ContainerKey::DropCapability,
    EntryValue::new("CAP_DAC_OVERRIDE CAP_IPC_OWNER")?,
)?;
builder.push_container(ContainerKey::AddCapability, EntryValue::new("CAP_NET_BIND_SERVICE")?)?;
builder.push_container(ContainerKey::Tmpfs, EntryValue::new("/data:mode=755,uid=1009,gid=1009")?)?;
builder.push_container(ContainerKey::Sysctl, EntryValue::new("net.ipv4.ip_forward=1")?)?;
builder.push_container(ContainerKey::Ulimit, EntryValue::new("nproc=4096:8192")?)?;
builder.push_container(
    ContainerKey::AddDevice,
    EntryValue::new("/dev/null:/dev/null:r")?,
)?;
builder.push_container(
    ContainerKey::Mask,
    EntryValue::new("/proc/acpi:/sys/firmware")?,
)?;
builder.push_container(ContainerKey::Unmask, EntryValue::new("ALL")?)?;
builder.push_container(
    ContainerKey::Environment,
    EntryValue::new("APP_ENV=production")?,
)?;
builder.push_container(
    ContainerKey::Label,
    EntryValue::new("org.example.application=example")?,
)?;

let generated = builder.build(SourceId::new(1))?;
assert_eq!(
    generated.text(),
    concat!(
        "[Container]\n",
        "ContainerName=example-application\n",
        "AddHost=host.docker.internal:host-gateway\n",
        "Image=example.invalid/application:1\n",
        "Entrypoint=[\"/usr/bin/env\",\"php\"]\n",
        "RunInit=true\n",
        "StopSignal=SIGUSR1\n",
        "StopTimeout=0\n",
        "Pull=newer\n",
        "PidsLimit=127\n",
        "HostName=app.example\n",
        "ShmSize=128m\n",
        "Memory=268435456b\n",
        "DropCapability=CAP_NET_ADMIN\n",
        "DropCapability=CAP_DAC_OVERRIDE CAP_IPC_OWNER\n",
        "AddCapability=CAP_NET_BIND_SERVICE\n",
        "Tmpfs=/data:mode=755,uid=1009,gid=1009\n",
        "Sysctl=net.ipv4.ip_forward=1\n",
        "Ulimit=nproc=4096:8192\n",
        "AddDevice=/dev/null:/dev/null:r\n",
        "Environment=APP_ENV=production\n",
        "Label=org.example.application=example\n",
    ),
);
# Ok(())
# }
```
