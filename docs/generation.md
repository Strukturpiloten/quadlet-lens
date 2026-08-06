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

`ContainerKey::RunInit` is a singleton that carries Quadlet's exact boolean text. Setting it to
`true` asks Podman to run its minimal signal-forwarding and child-reaping init process.

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

`EntryValue` is exact native semantic text on one physical line. It rejects NUL bytes and line
endings, but deliberately does not quote or normalize its contents. A caller that writes
`AddHost=`, `Environment=`, `Label=`, `Secret=`, `Entrypoint=`, `RunInit=`, `Exec=`, an identity/context key, a health-check
or readiness key, a systemd unit dependency, `PublishPort=`, or `Volume=` must select the appropriate native
systemd/Podman spelling.

This is an explicit boundary, not a claim that all value forms are interchangeable. Future
key-specific constructors can add stronger guarantees once exact Podman-version behavior and
systemd escaping rules are covered by evidence.

## Example

```rust
use quadlet_lens::{
    model::{ContainerKey, QuadletUnitType},
    render::{EntryValue, QuadletDocumentBuilder},
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
        "Environment=APP_ENV=production\n",
        "Label=org.example.application=example\n",
    ),
);
# Ok(())
# }
```
