# QuadletLens model

QuadletLens separates source syntax, typed document meaning, relationships, and target compatibility. Parsing one file does not silently inspect other files or the machine where the application runs.

| Stage             | Main type                     | Use it for                                                      |
| ----------------- | ----------------------------- | --------------------------------------------------------------- |
| Parsed file       | `model::QuadletDocument`      | Read typed keys while retaining source syntax.                  |
| Named file        | `model::NamedQuadletDocument` | Associate a validated filename with a document.                 |
| File set          | `model::QuadletDocumentSet`   | Validate names and build cross-file relationships.              |
| Capability target | `capability::PodmanTarget`    | Select an explicit minimum and optional maximum Podman version. |

The model covers `.container`, `.volume`, `.network`, `.pod`, `.image`, `.build`, `.kube`, and `.artifact` units plus the supported systemd sections. Unknown or malformed entries remain available as syntax evidence where recovery is possible.

## Environment and secrets

`QuadletDocument::container_environment_sources()` keeps inline assignments, environment-file
references, environment-exposing secret references, and source locations separate. QuadletLens
does not read those external sources. A caller may explicitly provide already-decoded values through
`AuthorizedContainerEnvironment`; absent authorization stays unresolved and protected values remain
redacted in `Debug` output.

Generated literal environment plans can opt into stable key order with
`ContainerEnvironmentPlan::sorted_by_name()`. Parsed and canonical rendering always preserve
authored order, groups, duplicates, resets, quoting, and specifiers.

## Direct use

```rust
use quadlet_lens::{model::{QuadletDocument, QuadletUnitType}, source::SourceId};

let source = "[Container]\nImage=example.invalid/web:1\n";
let parsed = QuadletDocument::parse(
    QuadletUnitType::Container,
    SourceId::new(1),
    source,
).expect("valid Quadlet input");
assert!(parsed.is_valid());
```

Parsing and model inspection are in-memory operations with no runtime side effects.
