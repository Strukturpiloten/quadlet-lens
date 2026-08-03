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

`build` emits deterministic text, reparses it through the normal syntax and typed-model pipeline,
and fails if that result contains an error. Successful output exposes the source text, typed
document, and complete parse result.

## Value boundary

`EntryValue` is exact native semantic text on one physical line. It rejects NUL bytes and line
endings, but deliberately does not quote or normalize its contents. A caller that writes
`AddHost=`, `Environment=`, `Exec=`, an identity/context key, a health-check or readiness key, a
systemd unit dependency, `PublishPort=`, or `Volume=` must select the appropriate native
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
    ContainerKey::AddHost,
    EntryValue::new("host.docker.internal:host-gateway")?,
)?;
builder.push_container(
    ContainerKey::Image,
    EntryValue::new("example.invalid/application:1")?,
)?;
builder.push_container(
    ContainerKey::Environment,
    EntryValue::new("APP_ENV=production")?,
)?;

let generated = builder.build(SourceId::new(1))?;
assert_eq!(
    generated.text(),
    concat!(
        "[Container]\n",
        "AddHost=host.docker.internal:host-gateway\n",
        "Image=example.invalid/application:1\n",
        "Environment=APP_ENV=production\n",
    ),
);
# Ok(())
# }
```
