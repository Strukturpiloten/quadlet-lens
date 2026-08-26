# Environment and secret values

QuadletLens separates authored syntax from external values. Parsing never reads an environment
file, process environment, Podman secret, filesystem path, or secret store.

## Inspect sources

Call `QuadletDocument::container_environment_sources()` to obtain:

- the ordered inline `Environment=` view;
- source-located `EnvironmentFile=` references;
- source-located `Secret=...,type=env` references;
- value-free diagnostics for malformed or systemd-specifier-dependent references.

Literal, deferred, and unmodeled references remain distinct. The original `AuthoredValue` and
syntax document remain the source of truth for exact spelling, continuations, quoting, and spans.
Mounted secrets without `type=env` are not environment values and are not returned as such.

## Authorize external values

`AuthorizedContainerEnvironment` accepts values already obtained and decoded by the caller. This
is the authorization boundary: QuadletLens does not open a referenced file or contact Podman.

```rust
use quadlet_lens::model::{
    AuthorizedContainerEnvironment, AuthorizedEnvironmentAssignment,
    SensitiveEnvironmentValue,
};

let mut authorized = AuthorizedContainerEnvironment::new();
authorized.authorize_environment_file(
    "./application.env",
    [AuthorizedEnvironmentAssignment::new(
        "APPLICATION_MODE",
        SensitiveEnvironmentValue::new("production")?,
    )?],
)?;
authorized.authorize_secret(
    "application-token",
    SensitiveEnvironmentValue::new("protected payload")?,
)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Pass that value to `ContainerEnvironmentSources::resolve`. Matching is exact and only literal
references resolve. A missing authorization remains `None`; an authorized empty payload remains
`Some("")`. File assignment order and duplicate names remain explicit.

QuadletLens deliberately does not parse environment-file bytes. The caller owns file-format and
encoding policy and supplies decoded assignments. This prevents an apparently pure parse from
acquiring host-specific or protected state.

## Protected values

External values use `SensitiveEnvironmentValue`. Its `Debug` representation is always redacted and
it has no `Display` implementation. Payload access requires the deliberately named
`expose_secret()` method. Raw source remains explicitly accessible through the ordinary syntax
model, so applications must also avoid printing source excerpts containing inline values.

## Ordering generated output

Parsed and canonical rendering preserve authored order. Generation plans also preserve insertion
order by default.

Call `ContainerEnvironmentPlan::sorted_by_name()` only for caller-owned literal assignments when a
stable human-readable order is wanted. It sorts stably within explicit reset boundaries, keeps
duplicate assignments in their original last-wins order, and expands assignment groups into
individual directives. It never reorders parsed source.

## BoxFerry boundary

BoxFerry may use the source view to report unresolved inputs and may provide authorized values only
after its own CLI policy permits acquisition. QuadletLens does not decide whether a value should be
promoted into BoxFerry's neutral model, written to Compose, or emitted as Quadlet. BoxFerry must:

1. preserve source identity and resolution state in diagnostics;
2. distinguish missing authorization from an authorized empty value;
3. avoid placing protected payloads in diagnostics or support bundles;
4. apply its own loss and promotion policy before cross-format output;
5. request sorted generation explicitly rather than changing source-preserving rendering.
